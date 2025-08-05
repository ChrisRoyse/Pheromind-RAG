// Go Analytics Dashboard Service
package analytics

import (
    "context"
    "encoding/json"
    "fmt"
    "log"
    "sync"
    "time"
    
    "github.com/gorilla/websocket"
    "github.com/prometheus/client_golang/prometheus"
    "github.com/segmentio/kafka-go"
    "go.mongodb.org/mongo-driver/bson"
    "go.mongodb.org/mongo-driver/mongo"
)

// MetricsCollector collects and aggregates metrics from various sources
type MetricsCollector struct {
    mongoClient   *mongo.Client
    kafkaReader   *kafka.Reader
    wsClients     sync.Map
    metrics       sync.Map
    aggregators   []Aggregator
    alertManager  *AlertManager
    
    // Prometheus metrics
    requestCounter   prometheus.Counter
    responseTime     prometheus.Histogram
    activeUsers      prometheus.Gauge
    errorRate        prometheus.Counter
}

// NewMetricsCollector creates a new metrics collector instance
func NewMetricsCollector(mongoURI string, kafkaBrokers []string) (*MetricsCollector, error) {
    // Connect to MongoDB
    client, err := mongo.Connect(context.Background(), options.Client().ApplyURI(mongoURI))
    if err != nil {
        return nil, fmt.Errorf("failed to connect to MongoDB: %w", err)
    }
    
    // Create Kafka reader
    reader := kafka.NewReader(kafka.ReaderConfig{
        Brokers:   kafkaBrokers,
        Topic:     "analytics-events",
        GroupID:   "analytics-dashboard",
        Partition: 0,
    })
    
    mc := &MetricsCollector{
        mongoClient:  client,
        kafkaReader:  reader,
        alertManager: NewAlertManager(),
        
        // Initialize Prometheus metrics
        requestCounter: prometheus.NewCounter(prometheus.CounterOpts{
            Name: "dashboard_requests_total",
            Help: "Total number of requests to the dashboard",
        }),
        responseTime: prometheus.NewHistogram(prometheus.HistogramOpts{
            Name:    "dashboard_response_time_seconds",
            Help:    "Response time distribution",
            Buckets: prometheus.DefBuckets,
        }),
        activeUsers: prometheus.NewGauge(prometheus.GaugeOpts{
            Name: "dashboard_active_users",
            Help: "Number of active users",
        }),
        errorRate: prometheus.NewCounter(prometheus.CounterOpts{
            Name: "dashboard_errors_total",
            Help: "Total number of errors",
        }),
    }
    
    // Register Prometheus metrics
    prometheus.MustRegister(mc.requestCounter, mc.responseTime, mc.activeUsers, mc.errorRate)
    
    // Initialize default aggregators
    mc.aggregators = []Aggregator{
        NewCountAggregator("page_views"),
        NewSumAggregator("revenue"),
        NewAverageAggregator("response_time"),
        NewPercentileAggregator("load_time", []float64{0.5, 0.95, 0.99}),
    }
    
    return mc, nil
}

// Start begins collecting metrics from all sources
func (mc *MetricsCollector) Start(ctx context.Context) error {
    errChan := make(chan error, 3)
    
    // Start Kafka consumer
    go func() {
        if err := mc.consumeKafkaEvents(ctx); err != nil {
            errChan <- fmt.Errorf("kafka consumer error: %w", err)
        }
    }()
    
    // Start metrics aggregation
    go func() {
        if err := mc.aggregateMetrics(ctx); err != nil {
            errChan <- fmt.Errorf("aggregation error: %w", err)
        }
    }()
    
    // Start alert monitoring
    go func() {
        if err := mc.monitorAlerts(ctx); err != nil {
            errChan <- fmt.Errorf("alert monitoring error: %w", err)
        }
    }()
    
    select {
    case <-ctx.Done():
        return ctx.Err()
    case err := <-errChan:
        return err
    }
}

// consumeKafkaEvents reads events from Kafka and processes them
func (mc *MetricsCollector) consumeKafkaEvents(ctx context.Context) error {
    for {
        select {
        case <-ctx.Done():
            return nil
        default:
            msg, err := mc.kafkaReader.ReadMessage(ctx)
            if err != nil {
                log.Printf("Error reading Kafka message: %v", err)
                continue
            }
            
            var event AnalyticsEvent
            if err := json.Unmarshal(msg.Value, &event); err != nil {
                log.Printf("Error unmarshaling event: %v", err)
                continue
            }
            
            // Process event
            mc.processEvent(&event)
            
            // Broadcast to WebSocket clients
            mc.broadcastEvent(&event)
        }
    }
}

// processEvent processes a single analytics event
func (mc *MetricsCollector) processEvent(event *AnalyticsEvent) {
    // Update Prometheus metrics
    mc.requestCounter.Inc()
    
    if event.Type == "error" {
        mc.errorRate.Inc()
    }
    
    if event.ResponseTime > 0 {
        mc.responseTime.Observe(event.ResponseTime)
    }
    
    // Store in time-series format
    metric := &Metric{
        Name:      event.Type,
        Value:     event.Value,
        Tags:      event.Tags,
        Timestamp: event.Timestamp,
    }
    
    mc.metrics.Store(fmt.Sprintf("%s:%d", event.Type, event.Timestamp.Unix()), metric)
    
    // Apply aggregators
    for _, aggregator := range mc.aggregators {
        if aggregator.Matches(event.Type) {
            aggregator.Add(event.Value)
        }
    }
}

// aggregateMetrics periodically aggregates metrics
func (mc *MetricsCollector) aggregateMetrics(ctx context.Context) error {
    ticker := time.NewTicker(1 * time.Minute)
    defer ticker.Stop()
    
    for {
        select {
        case <-ctx.Done():
            return nil
        case <-ticker.C:
            mc.performAggregation()
        }
    }
}

// performAggregation performs the actual aggregation
func (mc *MetricsCollector) performAggregation() {
    now := time.Now()
    
    // Aggregate metrics for different time windows
    windows := []struct {
        name     string
        duration time.Duration
    }{
        {"1m", 1 * time.Minute},
        {"5m", 5 * time.Minute},
        {"1h", 1 * time.Hour},
        {"1d", 24 * time.Hour},
    }
    
    for _, window := range windows {
        startTime := now.Add(-window.duration)
        
        // Collect metrics for this window
        windowMetrics := mc.getMetricsInRange(startTime, now)
        
        // Calculate aggregates
        aggregates := make(map[string]interface{})
        for _, aggregator := range mc.aggregators {
            result := aggregator.Calculate(windowMetrics)
            aggregates[aggregator.Name()] = result
        }
        
        // Store aggregated results
        mc.storeAggregates(window.name, aggregates)
        
        // Check for alerts
        mc.checkAlerts(window.name, aggregates)
    }
}

// getMetricsInRange retrieves metrics within a time range
func (mc *MetricsCollector) getMetricsInRange(start, end time.Time) []*Metric {
    var metrics []*Metric
    
    mc.metrics.Range(func(key, value interface{}) bool {
        metric := value.(*Metric)
        if metric.Timestamp.After(start) && metric.Timestamp.Before(end) {
            metrics = append(metrics, metric)
        }
        return true
    })
    
    return metrics
}

// storeAggregates stores aggregated metrics in MongoDB
func (mc *MetricsCollector) storeAggregates(window string, aggregates map[string]interface{}) {
    ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()
    
    collection := mc.mongoClient.Database("analytics").Collection("aggregates")
    
    doc := bson.M{
        "window":    window,
        "timestamp": time.Now(),
        "data":      aggregates,
    }
    
    if _, err := collection.InsertOne(ctx, doc); err != nil {
        log.Printf("Error storing aggregates: %v", err)
    }
}

// WebSocket handling for real-time updates
func (mc *MetricsCollector) HandleWebSocket(w http.ResponseWriter, r *http.Request) {
    upgrader := websocket.Upgrader{
        CheckOrigin: func(r *http.Request) bool { return true },
    }
    
    conn, err := upgrader.Upgrade(w, r, nil)
    if err != nil {
        log.Printf("WebSocket upgrade error: %v", err)
        return
    }
    defer conn.Close()
    
    clientID := generateClientID()
    mc.wsClients.Store(clientID, conn)
    defer mc.wsClients.Delete(clientID)
    
    // Update active users gauge
    mc.updateActiveUsers()
    defer mc.updateActiveUsers()
    
    // Send initial dashboard data
    if err := mc.sendDashboardData(conn); err != nil {
        log.Printf("Error sending initial data: %v", err)
        return
    }
    
    // Handle client messages
    for {
        var msg ClientMessage
        if err := conn.ReadJSON(&msg); err != nil {
            if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
                log.Printf("WebSocket error: %v", err)
            }
            break
        }
        
        mc.handleClientMessage(conn, &msg)
    }
}

// broadcastEvent sends an event to all connected WebSocket clients
func (mc *MetricsCollector) broadcastEvent(event *AnalyticsEvent) {
    message := map[string]interface{}{
        "type":  "event",
        "event": event,
    }
    
    mc.wsClients.Range(func(key, value interface{}) bool {
        conn := value.(*websocket.Conn)
        if err := conn.WriteJSON(message); err != nil {
            log.Printf("Error broadcasting to client %v: %v", key, err)
            mc.wsClients.Delete(key)
        }
        return true
    })
}

// sendDashboardData sends the current dashboard state to a client
func (mc *MetricsCollector) sendDashboardData(conn *websocket.Conn) error {
    // Collect current metrics
    currentMetrics := mc.getCurrentMetrics()
    
    // Get recent aggregates
    aggregates := mc.getRecentAggregates()
    
    // Get active alerts
    alerts := mc.alertManager.GetActiveAlerts()
    
    dashboardData := map[string]interface{}{
        "type":       "dashboard",
        "metrics":    currentMetrics,
        "aggregates": aggregates,
        "alerts":     alerts,
        "timestamp":  time.Now(),
    }
    
    return conn.WriteJSON(dashboardData)
}

// getCurrentMetrics returns current metric values
func (mc *MetricsCollector) getCurrentMetrics() map[string]interface{} {
    metrics := make(map[string]interface{})
    
    // Get latest values for each metric type
    mc.metrics.Range(func(key, value interface{}) bool {
        metric := value.(*Metric)
        
        // Only include recent metrics (last 5 minutes)
        if time.Since(metric.Timestamp) < 5*time.Minute {
            if existing, ok := metrics[metric.Name]; !ok || metric.Timestamp.After(existing.(*Metric).Timestamp) {
                metrics[metric.Name] = metric
            }
        }
        return true
    })
    
    return metrics
}

// getRecentAggregates retrieves recent aggregated data
func (mc *MetricsCollector) getRecentAggregates() map[string]interface{} {
    ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()
    
    collection := mc.mongoClient.Database("analytics").Collection("aggregates")
    
    // Get aggregates from last hour
    filter := bson.M{
        "timestamp": bson.M{
            "$gte": time.Now().Add(-1 * time.Hour),
        },
    }
    
    cursor, err := collection.Find(ctx, filter)
    if err != nil {
        log.Printf("Error retrieving aggregates: %v", err)
        return nil
    }
    defer cursor.Close(ctx)
    
    aggregates := make(map[string]interface{})
    for cursor.Next(ctx) {
        var doc bson.M
        if err := cursor.Decode(&doc); err != nil {
            continue
        }
        
        window := doc["window"].(string)
        aggregates[window] = doc["data"]
    }
    
    return aggregates
}

// handleClientMessage processes messages from WebSocket clients
func (mc *MetricsCollector) handleClientMessage(conn *websocket.Conn, msg *ClientMessage) {
    switch msg.Type {
    case "subscribe":
        // Handle metric subscription
        mc.subscribeToMetric(conn, msg.Metric)
    case "query":
        // Handle custom query
        mc.handleQuery(conn, msg.Query)
    case "export":
        // Handle data export request
        mc.handleExport(conn, msg.ExportOptions)
    default:
        log.Printf("Unknown message type: %s", msg.Type)
    }
}

// Alert monitoring
func (mc *MetricsCollector) monitorAlerts(ctx context.Context) error {
    ticker := time.NewTicker(30 * time.Second)
    defer ticker.Stop()
    
    for {
        select {
        case <-ctx.Done():
            return nil
        case <-ticker.C:
            mc.evaluateAlertRules()
        }
    }
}

// checkAlerts checks if any alert conditions are met
func (mc *MetricsCollector) checkAlerts(window string, aggregates map[string]interface{}) {
    for name, value := range aggregates {
        if rule := mc.alertManager.GetRule(name); rule != nil {
            if rule.Evaluate(value) {
                alert := &Alert{
                    Name:      rule.Name,
                    Severity:  rule.Severity,
                    Message:   fmt.Sprintf("Alert: %s = %v", name, value),
                    Timestamp: time.Now(),
                }
                
                mc.alertManager.TriggerAlert(alert)
                mc.broadcastAlert(alert)
            }
        }
    }
}

// broadcastAlert sends an alert to all connected clients
func (mc *MetricsCollector) broadcastAlert(alert *Alert) {
    message := map[string]interface{}{
        "type":  "alert",
        "alert": alert,
    }
    
    mc.wsClients.Range(func(key, value interface{}) bool {
        conn := value.(*websocket.Conn)
        conn.WriteJSON(message)
        return true
    })
}

// updateActiveUsers updates the active users gauge
func (mc *MetricsCollector) updateActiveUsers() {
    count := 0
    mc.wsClients.Range(func(_, _ interface{}) bool {
        count++
        return true
    })
    mc.activeUsers.Set(float64(count))
}

// Types
type AnalyticsEvent struct {
    Type         string                 `json:"type"`
    Value        float64                `json:"value"`
    Tags         map[string]string      `json:"tags"`
    Timestamp    time.Time              `json:"timestamp"`
    UserID       string                 `json:"user_id"`
    SessionID    string                 `json:"session_id"`
    ResponseTime float64                `json:"response_time,omitempty"`
}

type Metric struct {
    Name      string
    Value     float64
    Tags      map[string]string
    Timestamp time.Time
}

type ClientMessage struct {
    Type          string                 `json:"type"`
    Metric        string                 `json:"metric,omitempty"`
    Query         map[string]interface{} `json:"query,omitempty"`
    ExportOptions *ExportOptions         `json:"export_options,omitempty"`
}

type ExportOptions struct {
    Format    string    `json:"format"`
    StartTime time.Time `json:"start_time"`
    EndTime   time.Time `json:"end_time"`
    Metrics   []string  `json:"metrics"`
}

type Alert struct {
    Name      string    `json:"name"`
    Severity  string    `json:"severity"`
    Message   string    `json:"message"`
    Timestamp time.Time `json:"timestamp"`
}

// Aggregator interface
type Aggregator interface {
    Name() string
    Matches(metricName string) bool
    Add(value float64)
    Calculate(metrics []*Metric) interface{}
    Reset()
}