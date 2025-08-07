# Task 017: Gauge Metrics for Resource Usage

## Overview
Implement gauge metrics to track current resource usage like memory, CPU, and active connections.

## Objectives
- Track current resource utilization
- Monitor active connection counts
- Provide real-time system health metrics
- Enable resource-based alerting

## Requirements
- Gauge metrics for memory usage, CPU utilization
- Active connection and thread pool metrics
- Cache size and utilization gauges
- Real-time value updates

## Implementation Steps
1. Define gauge metric types
2. Implement gauge value tracking
3. Add resource monitoring collectors
4. Create periodic update mechanisms
5. Add validation and unit tests

## Acceptance Criteria
- [ ] Gauge metrics reflect current resource states
- [ ] Values update in real-time
- [ ] Resource monitoring covers key system components
- [ ] Memory usage tracking is accurate
- [ ] Unit tests validate gauge behavior

## Dependencies
- Histogram metrics implementation (task_016)
- System resource monitoring capabilities

## Estimated Time
10 minutes

## Files to Modify/Create
- `src/monitoring/metrics/gauges.rs` - Gauge implementations
- `src/monitoring/resource_monitor.rs` - Resource collection logic
- `tests/unit/monitoring/gauge_tests.rs` - Unit tests

## Testing Strategy
- Unit tests for gauge set/get operations
- Resource monitoring accuracy tests
- Update frequency validation tests