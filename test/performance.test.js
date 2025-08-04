/**
 * Performance Benchmarking Tests for MCP RAG Indexer
 * Tests performance targets, concurrent processing, memory optimization, and regression detection
 */

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

describe('Performance Benchmarking Suite', () => {
    let pythonPath;
    let performanceBenchmarkPath;

    beforeAll(() => {
        // Setup paths
        pythonPath = process.platform === 'win32' ? 
            path.join(__dirname, '..', 'runtime', 'windows-x64', 'python.exe') :
            'python3';
        performanceBenchmarkPath = path.join(__dirname, '..', 'python', 'performance_benchmarks.py');
        
        // Ensure benchmark file exists
        expect(fs.existsSync(performanceBenchmarkPath)).toBe(true);
    });

    describe('Multi-Scale Performance Testing', () => {
        test('should process small scale codebases in <1ms', async () => {
            const result = await runPerformanceBenchmark(['--scale', 'small', '--language', 'rust']);
            expect(result.success).toBe(true);
            
            // Check small scale performance targets
            const smallResults = result.data.small;
            expect(smallResults).toBeDefined();
            
            for (const [testName, benchmarkResult] of Object.entries(smallResults)) {
                if (benchmarkResult.processing_time !== undefined) {
                    expect(benchmarkResult.processing_time).toBeLessThan(0.001); // <1ms
                    expect(benchmarkResult.success).toBe(true);
                }
            }
        }, 30000);

        test('should process medium scale codebases in <100ms', async () => {
            const result = await runPerformanceBenchmark(['--scale', 'medium', '--language', 'rust']);
            expect(result.success).toBe(true);
            
            // Check medium scale performance targets
            const mediumResults = result.data.medium;
            expect(mediumResults).toBeDefined();
            
            for (const [testName, benchmarkResult] of Object.entries(mediumResults)) {
                if (benchmarkResult.processing_time !== undefined) {
                    expect(benchmarkResult.processing_time).toBeLessThan(0.1); // <100ms
                    expect(benchmarkResult.success).toBe(true);
                }
            }
        }, 60000);

        test('should process large scale codebases in <500ms', async () => {
            const result = await runPerformanceBenchmark(['--scale', 'large', '--language', 'rust']);
            expect(result.success).toBe(true);
            
            // Check large scale performance targets
            const largeResults = result.data.large;
            expect(largeResults).toBeDefined();
            
            for (const [testName, benchmarkResult] of Object.entries(largeResults)) {
                if (benchmarkResult.processing_time !== undefined) {
                    expect(benchmarkResult.processing_time).toBeLessThan(0.5); // <500ms
                    expect(benchmarkResult.success).toBe(true);
                }
            }
        }, 120000);

        test('should process enterprise scale codebases in <5s', async () => {
            const result = await runPerformanceBenchmark(['--scale', 'enterprise', '--language', 'rust']);
            expect(result.success).toBe(true);
            
            // Check enterprise scale performance targets
            const enterpriseResults = result.data.enterprise;
            expect(enterpriseResults).toBeDefined();
            
            for (const [testName, benchmarkResult] of Object.entries(enterpriseResults)) {
                if (benchmarkResult.processing_time !== undefined) {
                    expect(benchmarkResult.processing_time).toBeLessThan(5.0); // <5s
                    expect(benchmarkResult.success).toBe(true);
                }
            }
        }, 300000);
    });

    describe('Memory Usage Optimization', () => {
        test('should use <50MB memory for small codebases', async () => {
            const result = await runPerformanceBenchmark(['--scale', 'small', '--memory-test']);
            expect(result.success).toBe(true);
            
            const memoryResults = result.data.memory_optimization;
            expect(memoryResults).toBeDefined();
            expect(memoryResults.small).toBeDefined();
            expect(memoryResults.small.optimized_memory).toBeLessThan(50); // <50MB
        }, 30000);

        test('should use <200MB memory for medium codebases', async () => {
            const result = await runPerformanceBenchmark(['--scale', 'medium', '--memory-test']);
            expect(result.success).toBe(true);
            
            const memoryResults = result.data.memory_optimization;
            expect(memoryResults).toBeDefined();
            expect(memoryResults.medium).toBeDefined();
            expect(memoryResults.medium.optimized_memory).toBeLessThan(200); // <200MB
        }, 60000);

        test('should use <500MB memory for large codebases', async () => {
            const result = await runPerformanceBenchmark(['--scale', 'large', '--memory-test']);
            expect(result.success).toBe(true);
            
            const memoryResults = result.data.memory_optimization;
            expect(memoryResults).toBeDefined();
            expect(memoryResults.large).toBeDefined();
            expect(memoryResults.large.optimized_memory).toBeLessThan(500); // <500MB
        }, 120000);

        test('should achieve significant memory savings with optimization', async () => {
            const result = await runPerformanceBenchmark(['--memory-optimization-test']);
            expect(result.success).toBe(true);
            
            const memoryResults = result.data.memory_optimization;
            expect(memoryResults).toBeDefined();
            
            // Check for at least 20% memory savings across scales
            for (const [scale, scaleResult] of Object.entries(memoryResults)) {
                if (scaleResult.memory_savings_percent !== undefined) {
                    expect(scaleResult.memory_savings_percent).toBeGreaterThan(20);
                }
            }
        }, 180000);
    });

    describe('Concurrent Processing Performance', () => {
        test('should achieve 2x+ speedup with concurrent processing', async () => {
            const result = await runPerformanceBenchmark(['--concurrent-test']);
            expect(result.success).toBe(true);
            
            const concurrentResults = result.data.performance_report.concurrent_performance;
            expect(concurrentResults).toBeDefined();
            expect(concurrentResults.avg_speedup).toBeGreaterThan(2.0); // Target 2x speedup minimum
            expect(concurrentResults.speedup_achieved).toBe(true);
        }, 180000);

        test('should maintain accuracy with concurrent processing', async () => {
            const result = await runPerformanceBenchmark(['--concurrent-accuracy-test']);
            expect(result.success).toBe(true);
            
            // Compare concurrent vs single-threaded accuracy
            const performanceReport = result.data.performance_report;
            expect(performanceReport.summary.avg_documentation_coverage).toBeGreaterThan(0.8); // >80% coverage
            expect(performanceReport.summary.avg_confidence_score).toBeGreaterThan(0.7); // >70% confidence
        }, 180000);

        test('should scale efficiently with worker count', async () => {
            const workerCounts = [1, 2, 4, 8];
            const results = [];
            
            for (const workers of workerCounts) {
                const result = await runPerformanceBenchmark(['--workers', workers.toString(), '--scale', 'medium']);
                expect(result.success).toBe(true);
                results.push({
                    workers,
                    avgTime: result.data.performance_report.summary.avg_processing_time
                });
            }
            
            // Verify performance improves with more workers (up to optimal point)
            const singleThreadTime = results[0].avgTime;
            const fourThreadTime = results[2].avgTime;
            const speedup = singleThreadTime / fourThreadTime;
            
            expect(speedup).toBeGreaterThan(1.5); // At least 1.5x speedup with 4 workers
        }, 300000);
    });

    describe('Cross-Language Performance Consistency', () => {
        const languages = ['rust', 'python', 'javascript'];
        
        test.each(languages)('should meet performance targets for %s', async (language) => {
            const result = await runPerformanceBenchmark(['--language', language, '--scale', 'medium']);
            expect(result.success).toBe(true);
            
            const targetCompliance = result.data.performance_report.target_compliance;
            expect(targetCompliance).toBeDefined();
            expect(targetCompliance.medium).toBeDefined();
            expect(targetCompliance.medium.overall_compliant).toBe(true);
        }, 120000);

        test('should show consistent performance across languages', async () => {
            const languageResults = {};
            
            for (const language of languages) {
                const result = await runPerformanceBenchmark(['--language', language, '--scale', 'medium']);
                expect(result.success).toBe(true);
                languageResults[language] = result.data.performance_report.summary.avg_processing_time;
            }
            
            // Check that performance variation is within reasonable bounds (2x difference max)
            const times = Object.values(languageResults);
            const maxTime = Math.max(...times);
            const minTime = Math.min(...times);
            const variation = maxTime / minTime;
            
            expect(variation).toBeLessThan(2.0); // Less than 2x variation between languages
        }, 360000);
    });

    describe('Performance Regression Detection', () => {
        test('should detect performance regressions', async () => {
            // Run baseline benchmark
            const baseline = await runPerformanceBenchmark(['--establish-baseline']);
            expect(baseline.success).toBe(true);
            
            // Run regression detection
            const regression = await runPerformanceBenchmark(['--regression-test']);
            expect(regression.success).toBe(true);
            
            const regressionAnalysis = regression.data.regression_analysis;
            expect(regressionAnalysis).toBeDefined();
            expect(regressionAnalysis.regressions_detected).toBeDefined();
            expect(regressionAnalysis.improvements_detected).toBeDefined();
        }, 240000);

        test('should provide regression recommendations', async () => {
            const result = await runPerformanceBenchmark(['--regression-analysis']);
            expect(result.success).toBe(true);
            
            const regressionAnalysis = result.data.regression_analysis;
            expect(regressionAnalysis).toBeDefined();
            expect(regressionAnalysis.recommendations).toBeDefined();
            expect(Array.isArray(regressionAnalysis.recommendations)).toBe(true);
        }, 120000);
    });

    describe('Performance Monitoring Integration', () => {
        test('should provide comprehensive performance metrics', async () => {
            const result = await runIndexerPerformanceTest();
            expect(result.success).toBe(true);
            
            const metrics = result.metrics;
            expect(metrics).toBeDefined();
            expect(metrics.processing_stats).toBeDefined();
            expect(metrics.cache_stats).toBeDefined();
            expect(metrics.memory_stats).toBeDefined();
            
            // Verify key performance indicators
            expect(metrics.processing_stats.avg_processing_time).toBeDefined();
            expect(metrics.cache_stats.hit_rate).toBeDefined();
            expect(metrics.memory_stats.peak_memory_mb).toBeDefined();
        }, 60000);

        test('should track performance over time', async () => {
            const results = [];
            
            // Run multiple performance tests
            for (let i = 0; i < 5; i++) {
                const result = await runIndexerPerformanceTest();
                expect(result.success).toBe(true);
                results.push(result.metrics.processing_stats.avg_processing_time);
            }
            
            // Check performance consistency (coefficient of variation < 0.3)
            const mean = results.reduce((a, b) => a + b) / results.length;
            const variance = results.reduce((a, b) => a + Math.pow(b - mean, 2), 0) / results.length;
            const stdDev = Math.sqrt(variance);
            const coeffOfVariation = stdDev / mean;
            
            expect(coeffOfVariation).toBeLessThan(0.3); // Less than 30% variation
        }, 300000);
    });

    describe('Benchmark Result Validation', () => {
        test('should generate valid benchmark results', async () => {
            const result = await runPerformanceBenchmark(['--full-benchmark']);
            expect(result.success).toBe(true);
            
            // Validate result structure
            expect(result.data).toBeDefined();
            expect(result.data.performance_report).toBeDefined();
            expect(result.data.memory_optimization).toBeDefined();
            
            const report = result.data.performance_report;
            expect(report.summary).toBeDefined();
            expect(report.scale_analysis).toBeDefined();
            expect(report.target_compliance).toBeDefined();
            
            // Validate numeric metrics
            expect(typeof report.summary.avg_processing_time).toBe('number');
            expect(typeof report.summary.avg_memory_usage).toBe('number');
            expect(typeof report.summary.avg_documentation_coverage).toBe('number');
        }, 300000);

        test('should save benchmark results to file', async () => {
            const result = await runPerformanceBenchmark(['--save-results']);
            expect(result.success).toBe(true);
            
            // Check if results file was created
            const resultsFile = path.join(__dirname, '..', 'performance_benchmark_results.json');
            expect(fs.existsSync(resultsFile)).toBe(true);
            
            // Validate saved results
            const savedResults = JSON.parse(fs.readFileSync(resultsFile, 'utf8'));
            expect(savedResults.benchmark_metadata).toBeDefined();
            expect(savedResults.benchmark_metadata.timestamp).toBeDefined();
            expect(savedResults.benchmark_metadata.system_info).toBeDefined();
        }, 180000);
    });

    describe('Performance Target Compliance', () => {
        test('should meet all performance targets for production readiness', async () => {
            const result = await runPerformanceBenchmark(['--production-readiness-test']);
            expect(result.success).toBe(true);
            
            const targetCompliance = result.data.performance_report.target_compliance;
            expect(targetCompliance).toBeDefined();
            
            // Check each scale meets targets
            const scales = ['small', 'medium', 'large', 'enterprise'];
            for (const scale of scales) {
                expect(targetCompliance[scale]).toBeDefined();
                expect(targetCompliance[scale].time_compliant).toBe(true);
                expect(targetCompliance[scale].memory_compliant).toBe(true);
                expect(targetCompliance[scale].overall_compliant).toBe(true);
            }
        }, 600000);

        test('should provide recommendations for non-compliant targets', async () => {
            const result = await runPerformanceBenchmark(['--recommendations-test']);
            expect(result.success).toBe(true);
            
            const report = result.data.performance_report;
            expect(report.recommendations).toBeDefined();
            expect(Array.isArray(report.recommendations)).toBe(true);
            
            // If there are recommendations, they should be actionable
            for (const recommendation of report.recommendations) {
                expect(typeof recommendation).toBe('string');
                expect(recommendation.length).toBeGreaterThan(10);
            }
        }, 180000);
    });

    // Helper functions
    async function runPerformanceBenchmark(args = []) {
        return new Promise((resolve, reject) => {
            const process = spawn(pythonPath, [performanceBenchmarkPath, ...args], {
                cwd: path.join(__dirname, '..'),
                stdio: ['ignore', 'pipe', 'pipe']
            });

            let stdout = '';
            let stderr = '';

            process.stdout.on('data', (data) => {
                stdout += data.toString();
            });

            process.stderr.on('data', (data) => {
                stderr += data.toString();
            });

            process.on('close', (code) => {
                if (code === 0) {
                    try {
                        // Try to parse JSON output
                        const lines = stdout.trim().split('\n');
                        let jsonLine = lines.find(line => line.startsWith('{'));
                        
                        if (jsonLine) {
                            const data = JSON.parse(jsonLine);
                            resolve({ success: true, data });
                        } else {
                            resolve({ success: true, data: { message: stdout } });
                        }
                    } catch (e) {
                        resolve({ success: true, data: { message: stdout, error: e.message } });
                    }
                } else {
                    reject(new Error(`Benchmark failed with code ${code}: ${stderr}`));
                }
            });

            process.on('error', (error) => {
                reject(error);
            });
        });
    }

    async function runIndexerPerformanceTest() {
        return new Promise((resolve, reject) => {
            const testScript = `
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))
from indexer_universal import UniversalCodeIndexer
import json

indexer = UniversalCodeIndexer()

# Test with sample content
test_content = '''/// Test function
pub fn test() -> i32 { 42 }'''

# Run performance test
result = indexer.benchmark_processing_performance(test_content, 5)
metrics = indexer.get_performance_metrics()

print(json.dumps({
    'success': True,
    'benchmark_result': result,
    'metrics': metrics
}))
`;

            const process = spawn(pythonPath, ['-c', testScript], {
                cwd: path.join(__dirname, '..'),
                stdio: ['ignore', 'pipe', 'pipe']
            });

            let stdout = '';
            let stderr = '';

            process.stdout.on('data', (data) => {
                stdout += data.toString();
            });

            process.stderr.on('data', (data) => {
                stderr += data.toString();
            });

            process.on('close', (code) => {
                if (code === 0) {
                    try {
                        const result = JSON.parse(stdout.trim());
                        resolve(result);
                    } catch (e) {
                        reject(new Error(`Failed to parse JSON: ${e.message}`));
                    }
                } else {
                    reject(new Error(`Test failed with code ${code}: ${stderr}`));
                }
            });
        });
    }
});