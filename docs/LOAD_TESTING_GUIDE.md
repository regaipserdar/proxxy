# Load Testing Guide

This guide covers comprehensive load testing for the Proxxy distributed proxy system using the `oha` tool.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Test Server Setup](#test-server-setup)
- [Basic Load Tests](#basic-load-tests)
- [Proxy Load Tests](#proxy-load-tests)
- [Performance Metrics](#performance-metrics)
- [Database Performance](#database-performance)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Install oha
```bash
# Install using cargo
cargo install oha

# Or using homebrew (macOS)
brew install oha
```

### Start Services
```bash
# 1. Start orchestrator
./target/release/orchestrator

# 2. Start proxy-agent
./target/release/proxy-agent --orchestrator-url http://127.0.0.1:50051

# 3. Start test server
cd test-server && cargo run --release
```

## Test Server Setup

The test server provides simple JSON endpoints for load testing:

### Features
- **Fast JSON Response**: `{"message":"Hello from test server","timestamp":1234567890}`
- **Multiple Endpoints**: `/` and `/test`
- **No Directory Listing**: Security-focused
- **Static File Serving**: Optional for additional testing

### Endpoints
```bash
# Main test endpoint
curl http://127.0.0.1:8000/

# Alternative test endpoint  
curl http://127.0.0.1:8000/test

# Static file (if needed)
curl http://127.0.0.1:8000/test.json
```
```bash
rooter@rooterbyte ~/Documents/proxxy (gui-development*) $ sudo lsof -i :8000
Password:
COMMAND   PID   USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
Python  55954 rooter    4u  IPv6 0xbd5d901a5682dff5      0t0  TCP *:irdmi (LISTEN)
rooter@rooterbyte ~/Documents/proxxy (gui-development*) $ kill -9 55954
rooter@rooterbyte ~/Documents/proxxy (gui-development*) $ sudo lsof -i :8000

pkill orchestrator
pkill proxy-agent
pkill test-server
```
## Basic Load Tests

### Test 1: Direct Server (Baseline)
```bash
oha -n 1000 -c 50 http://127.0.0.1:8000
```

**Expected Results:**
- **Success Rate**: ~99-100%
- **Response Time**: <10ms average
- **RPS**: 1000+

### Test 2: Higher Load
```bash
oha -n 5000 -c 100 http://127.0.0.1:8000
```

### Test 3: Maximum Load
```bash
oha -n 10000 -c 200 http://127.0.0.1:8000
```

## Proxy Load Tests

### Test 1: Basic Proxy Performance
```bash
oha -n 1000 -c 50 -x http://127.0.0.1:9095 http://127.0.0.1:8000
```

**Expected Results:**
- **Success Rate**: ~95-98%
- **Response Time**: 40-100ms average
- **RPS**: 600-1200
- **DB Records**: ~1000 saved

### Test 2: High Concurrency
```bash
oha -n 5000 -c 100 -x http://127.0.0.1:9095 http://127.0.0.1:8000
```

### Test 3: Stress Test
```bash
oha -n 10000 -c 200 -x http://127.0.0.1:9095 http://127.0.0.1:8000
```

## Performance Metrics

### Key Metrics to Monitor

#### Response Time
- **P50 (Median)**: 50th percentile
- **P95**: 95th percentile
- **P99**: 99th percentile
- **Average**: Mean response time

#### Throughput
- **RPS**: Requests per second
- **Success Rate**: % successful requests
- **Error Rate**: % failed requests

#### Database Performance
```bash
# Check total requests saved
sqlite3 proxxy.db "SELECT COUNT(*) FROM http_transactions;"

# Check recent requests
sqlite3 proxxy.db "SELECT COUNT(*) FROM http_transactions WHERE req_timestamp > strftime('%s', 'now', '-5 minutes');"

# Monitor database size
ls -lh proxxy.db*
```

## Test Results Reference

### Optimized System Results (After WAL + Async DB)
```
Test: 10,000 requests, 50 concurrent connections
Success Rate: 95.51%
Total Time: 8.56 sec
RPS: 1167.78
Average Response Time: 41.3ms
P95: 90.7ms
Database Records: 9551 saved
```

### Performance Comparison

| Metric | Before Optimization | After Optimization | Improvement |
|--------|-------------------|-------------------|-------------|
| Avg Response Time | 94.2ms | 41.3ms | 56% faster |
| RPS | 642 | 1167 | 82% higher |
| DB Success Rate | 85% | 99.9% | 18% better |
| Database Locks | Frequent | None | 100% eliminated |

## Database Performance

### WAL Mode Configuration
The system now uses WAL (Write-Ahead Logging) mode for better concurrency:

```sql
PRAGMA journal_mode=WAL;    -- Better concurrency
PRAGMA synchronous=NORMAL;  -- Faster writes
```

### Monitoring Database Performance
```bash
# Check database settings
sqlite3 proxxy.db "PRAGMA journal_mode;"
sqlite3 proxxy.db "PRAGMA synchronous;"

# Monitor active connections
sqlite3 proxxy.db "PRAGMA busy_timeout;"

# Check database health
sqlite3 proxxy.db "PRAGMA integrity_check;"
```

## Advanced Testing

### Custom Headers Test
```bash
oha -n 1000 -c 50 \
  -H "Authorization: Bearer token123" \
  -H "Content-Type: application/json" \
  -x http://127.0.0.1:9095 \
  http://127.0.0.1:8000
```

### POST Request Test
```bash
echo '{"test":"data"}' | oha -n 1000 -c 50 \
  -m POST \
  -H "Content-Type: application/json" \
  -x http://127.0.0.1:9095 \
  http://127.0.0.1:8000
```

### Timeout and Retry Testing
```bash
oha -n 1000 -c 50 \
  --timeout 10s \
  --redirect 5 \
  -x http://127.0.0.1:9095 \
  http://127.0.0.1:8000
```

## Troubleshooting

### Common Issues

#### 1. "Can't assign requested address" Error
**Cause**: Socket exhaustion on macOS
**Solution**: 
```bash
# Increase socket limits
sudo sysctl -w net.inet.ip.portrange.first=1024
sudo sysctl -w net.inet.ip.portrange.last=65535
```

#### 2. "Database is locked" Error
**Cause**: High concurrent writes
**Solution**: Already fixed with WAL mode and async processing

#### 3. High Response Times
**Check**:
- Database locks
- Agent connectivity
- Network latency
- System resources

### Performance Tuning

#### Database Optimization
```bash
# Optimize SQLite for high writes
sqlite3 proxxy.db "PRAGMA cache_size=10000;"
sqlite3 proxxy.db "PRAGMA temp_store=MEMORY;"
```

#### System Monitoring
```bash
# Monitor system resources during tests
top -p $(pgrep orchestrator)
top -p $(pgrep proxy-agent)

# Monitor network connections
netstat -an | grep :9095
```

## Continuous Testing

### Automated Test Script
```bash
#!/bin/bash
# test_continuous.sh

echo "Starting continuous load test..."

while true; do
    echo "$(date): Running load test..."
    
    oha -n 1000 -c 50 -x http://127.0.0.1:9095 http://127.0.0.1:8000 > test_result_$(date +%s).json
    
    # Check database
    count=$(sqlite3 proxxy.db "SELECT COUNT(*) FROM http_transactions;")
    echo "$(date): Database records: $count"
    
    sleep 60  # Wait 1 minute between tests
done
```

### Results Collection
```bash
# Create test results directory
mkdir -p test_results

# Run test with different loads
for c in 10 25 50 100 200; do
    echo "Testing with $c concurrent connections..."
    oha -n 5000 -c $c -x http://127.0.0.1:9095 http://127.0.0.1:8000 \
        > test_results/test_c${c}_$(date +%s).json
done
```

## Best Practices

### Before Running Tests
1. **Clean Database**: `sqlite3 proxxy.db "DELETE FROM http_transactions;"`
2. **Restart Services**: Fresh start for accurate results
3. **Monitor Resources**: Use `top` or `htop`
4. **Close Other Apps**: Avoid interference

### During Tests
1. **Monitor Logs**: `tail -f orchestrator.log proxy-agent.log`
2. **Check Database**: Verify records are being saved
3. **Watch Error Rates**: High errors indicate issues
4. **System Resources**: Monitor CPU/memory usage

### After Tests
1. **Save Results**: Export test outputs
2. **Database Analysis**: Check saved records
3. **Performance Analysis**: Compare metrics
4. **Clean Up**: Reset for next test

## Expected Performance Benchmarks

### System Targets
- **RPS**: 1000+ requests per second
- **Response Time**: <50ms average
- **Success Rate**: >95%
- **Database Success**: >99%

### Bottlenecks to Watch
- **Database Locks**: Should be eliminated with WAL
- **Socket Limits**: macOS may hit limits >200 concurrent
- **Memory Usage**: Monitor for leaks
- **CPU Usage**: Should stay <80%

## Conclusion

This load testing setup provides comprehensive testing of the Proxxy system's performance under various loads. The asynchronous database operations and WAL mode have significantly improved performance, eliminating database locks and enabling higher throughput.

For production deployment, regularly run these tests to monitor performance regression and ensure the system meets the required benchmarks.