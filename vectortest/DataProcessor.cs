// C# Data Processing Service
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using System.Threading.Channels;
using System.Text.Json;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Hosting;

namespace DataProcessing.Services
{
    public class DataProcessor : BackgroundService
    {
        private readonly ILogger<DataProcessor> _logger;
        private readonly IDataRepository _repository;
        private readonly IMessageQueue _messageQueue;
        private readonly IMetricsCollector _metrics;
        private readonly Channel<ProcessingRequest> _processingChannel;
        private readonly DataProcessorOptions _options;

        public DataProcessor(
            ILogger<DataProcessor> logger,
            IDataRepository repository,
            IMessageQueue messageQueue,
            IMetricsCollector metrics,
            DataProcessorOptions options)
        {
            _logger = logger;
            _repository = repository;
            _messageQueue = messageQueue;
            _metrics = metrics;
            _options = options;
            
            _processingChannel = Channel.CreateUnbounded<ProcessingRequest>(
                new UnboundedChannelOptions
                {
                    SingleReader = false,
                    SingleWriter = false
                });
        }

        protected override async Task ExecuteAsync(CancellationToken stoppingToken)
        {
            var tasks = new List<Task>();

            // Start multiple processors
            for (int i = 0; i < _options.ConcurrencyLevel; i++)
            {
                tasks.Add(ProcessDataAsync(i, stoppingToken));
            }

            // Start queue listener
            tasks.Add(ListenToQueueAsync(stoppingToken));

            await Task.WhenAll(tasks);
        }

        private async Task ProcessDataAsync(int processorId, CancellationToken cancellationToken)
        {
            _logger.LogInformation($"Data processor {processorId} started");

            await foreach (var request in _processingChannel.Reader.ReadAllAsync(cancellationToken))
            {
                try
                {
                    using var activity = _metrics.StartActivity("ProcessData");
                    
                    _logger.LogDebug($"Processor {processorId} processing request {request.Id}");
                    
                    var result = await ProcessRequestAsync(request, cancellationToken);
                    
                    await _repository.SaveResultAsync(result, cancellationToken);
                    
                    _metrics.RecordSuccess("DataProcessed");
                    
                    // Send completion notification
                    await _messageQueue.PublishAsync(new ProcessingCompleteEvent
                    {
                        RequestId = request.Id,
                        ProcessorId = processorId,
                        CompletedAt = DateTime.UtcNow,
                        ResultId = result.Id
                    }, cancellationToken);
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, $"Error processing request {request.Id}");
                    _metrics.RecordError("ProcessingError", ex);
                    
                    await HandleProcessingError(request, ex, cancellationToken);
                }
            }
        }

        private async Task<ProcessingResult> ProcessRequestAsync(
            ProcessingRequest request, 
            CancellationToken cancellationToken)
        {
            var result = new ProcessingResult
            {
                Id = Guid.NewGuid(),
                RequestId = request.Id,
                StartedAt = DateTime.UtcNow
            };

            try
            {
                // Stage 1: Data validation
                var validationResult = await ValidateDataAsync(request.Data, cancellationToken);
                if (!validationResult.IsValid)
                {
                    result.Status = ProcessingStatus.ValidationFailed;
                    result.Errors = validationResult.Errors;
                    return result;
                }

                // Stage 2: Data transformation
                var transformedData = await TransformDataAsync(request.Data, request.TransformationRules, cancellationToken);
                
                // Stage 3: Business rules processing
                var processedData = await ApplyBusinessRulesAsync(transformedData, cancellationToken);
                
                // Stage 4: Enrichment
                var enrichedData = await EnrichDataAsync(processedData, cancellationToken);
                
                // Stage 5: Output generation
                result.OutputData = await GenerateOutputAsync(enrichedData, request.OutputFormat, cancellationToken);
                result.Status = ProcessingStatus.Completed;
                result.CompletedAt = DateTime.UtcNow;
                
                // Record metrics
                var processingTime = result.CompletedAt.Value - result.StartedAt;
                _metrics.RecordDuration("ProcessingDuration", processingTime);
                
                return result;
            }
            catch (Exception ex)
            {
                result.Status = ProcessingStatus.Failed;
                result.Errors = new[] { new ProcessingError { Code = "PROCESSING_ERROR", Message = ex.Message } };
                result.CompletedAt = DateTime.UtcNow;
                throw;
            }
        }

        private async Task<ValidationResult> ValidateDataAsync(
            Dictionary<string, object> data, 
            CancellationToken cancellationToken)
        {
            var errors = new List<ProcessingError>();

            // Check required fields
            foreach (var field in _options.RequiredFields)
            {
                if (!data.ContainsKey(field) || data[field] == null)
                {
                    errors.Add(new ProcessingError
                    {
                        Code = "MISSING_FIELD",
                        Message = $"Required field '{field}' is missing",
                        Field = field
                    });
                }
            }

            // Validate data types
            foreach (var (field, expectedType) in _options.FieldTypes)
            {
                if (data.TryGetValue(field, out var value) && value != null)
                {
                    if (!IsValidType(value, expectedType))
                    {
                        errors.Add(new ProcessingError
                        {
                            Code = "INVALID_TYPE",
                            Message = $"Field '{field}' has invalid type. Expected: {expectedType}",
                            Field = field
                        });
                    }
                }
            }

            // Custom validation rules
            if (_options.CustomValidators != null)
            {
                foreach (var validator in _options.CustomValidators)
                {
                    var validatorErrors = await validator.ValidateAsync(data, cancellationToken);
                    errors.AddRange(validatorErrors);
                }
            }

            return new ValidationResult
            {
                IsValid = !errors.Any(),
                Errors = errors.ToArray()
            };
        }

        private async Task<Dictionary<string, object>> TransformDataAsync(
            Dictionary<string, object> data,
            List<TransformationRule> rules,
            CancellationToken cancellationToken)
        {
            var transformedData = new Dictionary<string, object>(data);

            foreach (var rule in rules.OrderBy(r => r.Order))
            {
                switch (rule.Type)
                {
                    case TransformationType.MapField:
                        MapField(transformedData, rule);
                        break;
                        
                    case TransformationType.Calculate:
                        await CalculateFieldAsync(transformedData, rule, cancellationToken);
                        break;
                        
                    case TransformationType.Normalize:
                        NormalizeField(transformedData, rule);
                        break;
                        
                    case TransformationType.Filter:
                        FilterData(transformedData, rule);
                        break;
                        
                    case TransformationType.Aggregate:
                        await AggregateDataAsync(transformedData, rule, cancellationToken);
                        break;
                }
            }

            return transformedData;
        }

        private void MapField(Dictionary<string, object> data, TransformationRule rule)
        {
            if (data.TryGetValue(rule.SourceField, out var value))
            {
                data[rule.TargetField] = value;
                
                if (rule.RemoveSource)
                {
                    data.Remove(rule.SourceField);
                }
            }
        }

        private async Task CalculateFieldAsync(
            Dictionary<string, object> data,
            TransformationRule rule,
            CancellationToken cancellationToken)
        {
            var calculator = _options.Calculators[rule.CalculatorName];
            var result = await calculator.CalculateAsync(data, rule.Parameters, cancellationToken);
            data[rule.TargetField] = result;
        }

        private void NormalizeField(Dictionary<string, object> data, TransformationRule rule)
        {
            if (data.TryGetValue(rule.SourceField, out var value))
            {
                var normalizedValue = rule.NormalizationType switch
                {
                    NormalizationType.Uppercase => value?.ToString()?.ToUpper(),
                    NormalizationType.Lowercase => value?.ToString()?.ToLower(),
                    NormalizationType.Trim => value?.ToString()?.Trim(),
                    NormalizationType.RemoveSpaces => value?.ToString()?.Replace(" ", ""),
                    _ => value
                };
                
                data[rule.TargetField ?? rule.SourceField] = normalizedValue;
            }
        }

        private void FilterData(Dictionary<string, object> data, TransformationRule rule)
        {
            var keysToRemove = new List<string>();
            
            foreach (var kvp in data)
            {
                if (!rule.FilterPredicate(kvp.Key, kvp.Value))
                {
                    keysToRemove.Add(kvp.Key);
                }
            }
            
            foreach (var key in keysToRemove)
            {
                data.Remove(key);
            }
        }

        private async Task AggregateDataAsync(
            Dictionary<string, object> data,
            TransformationRule rule,
            CancellationToken cancellationToken)
        {
            if (data.TryGetValue(rule.SourceField, out var value) && value is IEnumerable<object> collection)
            {
                var aggregatedValue = rule.AggregationType switch
                {
                    AggregationType.Sum => collection.OfType<IConvertible>().Sum(Convert.ToDouble),
                    AggregationType.Average => collection.OfType<IConvertible>().Average(Convert.ToDouble),
                    AggregationType.Count => collection.Count(),
                    AggregationType.Min => collection.OfType<IComparable>().Min(),
                    AggregationType.Max => collection.OfType<IComparable>().Max(),
                    _ => null
                };
                
                data[rule.TargetField] = aggregatedValue;
            }
        }

        private async Task<Dictionary<string, object>> ApplyBusinessRulesAsync(
            Dictionary<string, object> data,
            CancellationToken cancellationToken)
        {
            var processedData = new Dictionary<string, object>(data);
            
            foreach (var ruleEngine in _options.BusinessRuleEngines)
            {
                var ruleResults = await ruleEngine.EvaluateAsync(processedData, cancellationToken);
                
                foreach (var result in ruleResults)
                {
                    if (result.ShouldModifyData)
                    {
                        foreach (var modification in result.DataModifications)
                        {
                            processedData[modification.Key] = modification.Value;
                        }
                    }
                    
                    if (result.ShouldAddMetadata)
                    {
                        processedData[$"_rule_{result.RuleName}"] = result.Metadata;
                    }
                }
            }
            
            return processedData;
        }

        private async Task<Dictionary<string, object>> EnrichDataAsync(
            Dictionary<string, object> data,
            CancellationToken cancellationToken)
        {
            var enrichedData = new Dictionary<string, object>(data);
            
            // Parallel enrichment from multiple sources
            var enrichmentTasks = _options.EnrichmentSources.Select(async source =>
            {
                try
                {
                    var enrichmentData = await source.EnrichAsync(data, cancellationToken);
                    return new { Source = source.Name, Data = enrichmentData };
                }
                catch (Exception ex)
                {
                    _logger.LogWarning(ex, $"Failed to enrich from source: {source.Name}");
                    return null;
                }
            });
            
            var enrichmentResults = await Task.WhenAll(enrichmentTasks);
            
            foreach (var result in enrichmentResults.Where(r => r != null))
            {
                foreach (var kvp in result.Data)
                {
                    enrichedData[$"{result.Source}_{kvp.Key}"] = kvp.Value;
                }
            }
            
            return enrichedData;
        }

        private async Task<string> GenerateOutputAsync(
            Dictionary<string, object> data,
            OutputFormat format,
            CancellationToken cancellationToken)
        {
            return format switch
            {
                OutputFormat.Json => JsonSerializer.Serialize(data, new JsonSerializerOptions { WriteIndented = true }),
                OutputFormat.Csv => await GenerateCsvAsync(data, cancellationToken),
                OutputFormat.Xml => await GenerateXmlAsync(data, cancellationToken),
                OutputFormat.Custom => await _options.CustomOutputGenerator.GenerateAsync(data, cancellationToken),
                _ => throw new NotSupportedException($"Output format {format} is not supported")
            };
        }

        private async Task<string> GenerateCsvAsync(Dictionary<string, object> data, CancellationToken cancellationToken)
        {
            // CSV generation logic
            var headers = string.Join(",", data.Keys);
            var values = string.Join(",", data.Values.Select(v => $"\"{v}\""));
            return $"{headers}\n{values}";
        }

        private async Task<string> GenerateXmlAsync(Dictionary<string, object> data, CancellationToken cancellationToken)
        {
            // XML generation logic
            var xml = new System.Text.StringBuilder();
            xml.AppendLine("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
            xml.AppendLine("<data>");
            
            foreach (var kvp in data)
            {
                xml.AppendLine($"  <{kvp.Key}>{System.Security.SecurityElement.Escape(kvp.Value?.ToString())}</{kvp.Key}>");
            }
            
            xml.AppendLine("</data>");
            return xml.ToString();
        }

        private bool IsValidType(object value, Type expectedType)
        {
            if (value == null) return true;
            
            return expectedType.IsAssignableFrom(value.GetType()) ||
                   (expectedType == typeof(string) && value != null) ||
                   (expectedType.IsNumericType() && value is IConvertible);
        }

        private async Task ListenToQueueAsync(CancellationToken cancellationToken)
        {
            await foreach (var message in _messageQueue.ConsumeAsync<ProcessingRequest>(cancellationToken))
            {
                await _processingChannel.Writer.WriteAsync(message, cancellationToken);
            }
        }

        private async Task HandleProcessingError(ProcessingRequest request, Exception exception, CancellationToken cancellationToken)
        {
            var errorEvent = new ProcessingErrorEvent
            {
                RequestId = request.Id,
                ErrorMessage = exception.Message,
                StackTrace = exception.StackTrace,
                OccurredAt = DateTime.UtcNow
            };
            
            await _messageQueue.PublishAsync(errorEvent, cancellationToken);
            
            // Retry logic
            if (request.RetryCount < _options.MaxRetries)
            {
                request.RetryCount++;
                await Task.Delay(TimeSpan.FromSeconds(Math.Pow(2, request.RetryCount)), cancellationToken);
                await _processingChannel.Writer.WriteAsync(request, cancellationToken);
            }
            else
            {
                // Move to dead letter queue
                await _messageQueue.PublishToDeadLetterAsync(request, cancellationToken);
            }
        }
    }

    public class ProcessingRequest
    {
        public Guid Id { get; set; }
        public Dictionary<string, object> Data { get; set; }
        public List<TransformationRule> TransformationRules { get; set; }
        public OutputFormat OutputFormat { get; set; }
        public int RetryCount { get; set; }
    }

    public class ProcessingResult
    {
        public Guid Id { get; set; }
        public Guid RequestId { get; set; }
        public ProcessingStatus Status { get; set; }
        public DateTime StartedAt { get; set; }
        public DateTime? CompletedAt { get; set; }
        public string OutputData { get; set; }
        public ProcessingError[] Errors { get; set; }
    }

    public enum ProcessingStatus
    {
        Pending,
        Processing,
        Completed,
        Failed,
        ValidationFailed
    }

    public class ProcessingError
    {
        public string Code { get; set; }
        public string Message { get; set; }
        public string Field { get; set; }
    }

    public class ValidationResult
    {
        public bool IsValid { get; set; }
        public ProcessingError[] Errors { get; set; }
    }

    public class TransformationRule
    {
        public string Name { get; set; }
        public TransformationType Type { get; set; }
        public int Order { get; set; }
        public string SourceField { get; set; }
        public string TargetField { get; set; }
        public bool RemoveSource { get; set; }
        public string CalculatorName { get; set; }
        public Dictionary<string, object> Parameters { get; set; }
        public NormalizationType NormalizationType { get; set; }
        public AggregationType AggregationType { get; set; }
        public Func<string, object, bool> FilterPredicate { get; set; }
    }

    public enum TransformationType
    {
        MapField,
        Calculate,
        Normalize,
        Filter,
        Aggregate
    }

    public enum NormalizationType
    {
        Uppercase,
        Lowercase,
        Trim,
        RemoveSpaces
    }

    public enum AggregationType
    {
        Sum,
        Average,
        Count,
        Min,
        Max
    }

    public enum OutputFormat
    {
        Json,
        Csv,
        Xml,
        Custom
    }
}