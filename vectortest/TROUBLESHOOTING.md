# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with the e-commerce platform. If you can't find a solution here, please check our [GitHub Issues](https://github.com/example/ecommerce/issues) or contact support.

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Runtime Errors](#runtime-errors)
3. [Database Problems](#database-problems)
4. [API Issues](#api-issues)
5. [Performance Problems](#performance-problems)
6. [Docker and Container Issues](#docker-and-container-issues)
7. [Authentication and Security](#authentication-and-security)
8. [Payment Gateway Problems](#payment-gateway-problems)
9. [Search and Indexing](#search-and-indexing)
10. [Debugging Techniques](#debugging-techniques)

## Installation Issues

### Node.js Version Mismatch

**Problem:** Error message indicating incompatible Node.js version

```bash
error: Node.js version 16.x is not supported. Please use version 18.x or higher.
```

**Solution:**

1. Check your current Node.js version:
```bash
node --version
```

2. Install the correct version using nvm:
```bash
nvm install 18
nvm use 18
nvm alias default 18
```

3. Verify the installation:
```bash
node --version  # Should show v18.x.x
npm --version   # Should show 9.x.x
```

### npm Install Failures

**Problem:** Dependencies fail to install with various errors

**Common Causes and Solutions:**

1. **Clear npm cache:**
```bash
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

2. **Network issues (behind proxy):**
```bash
npm config set proxy http://proxy.company.com:8080
npm config set https-proxy http://proxy.company.com:8080
```

3. **Permission errors on macOS/Linux:**
```bash
# Option 1: Fix npm permissions
mkdir ~/.npm-global
npm config set prefix '~/.npm-global'
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc

# Option 2: Use sudo (not recommended)
sudo npm install
```

4. **Platform-specific dependencies:**
```bash
# Install build tools for native dependencies
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt-get install build-essential

# Windows
npm install --global windows-build-tools
```

### Docker Compose Issues

**Problem:** Docker Compose fails to start services

**Solution:**

1. **Check Docker daemon:**
```bash
docker info
# If daemon not running:
sudo systemctl start docker  # Linux
# or restart Docker Desktop on macOS/Windows
```

2. **Port conflicts:**
```bash
# Check if ports are in use
lsof -i :3000  # macOS/Linux
netstat -ano | findstr :3000  # Windows

# Change ports in docker-compose.yml if needed
```

3. **Insufficient resources:**
```bash
# Check Docker resource limits
docker system df
docker system prune -a  # Clean up unused resources
```

## Runtime Errors

### Application Won't Start

**Problem:** Application crashes on startup

**Diagnostic Steps:**

1. **Check logs:**
```bash
# Development
npm run dev -- --verbose

# Production
docker logs ecommerce-app

# Check system logs
journalctl -u ecommerce  # Linux systemd
```

2. **Verify environment variables:**
```bash
# Check if .env file exists
ls -la .env*

# Validate required variables
npm run validate:env
```

3. **Common startup errors:**

```javascript
// Error: Cannot find module
Error: Cannot find module 'express'
// Solution: Run npm install

// Error: EADDRINUSE
Error: listen EADDRINUSE: address already in use :::3000
// Solution: Kill process using port or change port

// Error: ECONNREFUSED
Error: connect ECONNREFUSED 127.0.0.1:5432
// Solution: Start database service
```

### Memory Leaks

**Problem:** Application memory usage grows over time

**Diagnosis:**

1. **Monitor memory usage:**
```javascript
// Add memory monitoring
setInterval(() => {
  const used = process.memoryUsage();
  console.log('Memory Usage:', {
    rss: `${Math.round(used.rss / 1024 / 1024)} MB`,
    heapTotal: `${Math.round(used.heapTotal / 1024 / 1024)} MB`,
    heapUsed: `${Math.round(used.heapUsed / 1024 / 1024)} MB`,
    external: `${Math.round(used.external / 1024 / 1024)} MB`,
  });
}, 30000);
```

2. **Generate heap snapshot:**
```bash
# Start with inspect flag
node --inspect server.js

# In Chrome DevTools:
# 1. Navigate to chrome://inspect
# 2. Click "inspect" for your process
# 3. Go to Memory tab
# 4. Take heap snapshot
```

3. **Common causes:**
- Unclosed database connections
- Event listener accumulation
- Large arrays/objects in closure scope
- Circular references

**Solutions:**

```javascript
// Fix: Close connections properly
class DatabaseService {
  async query(sql) {
    const connection = await this.pool.getConnection();
    try {
      return await connection.execute(sql);
    } finally {
      connection.release(); // Always release
    }
  }
}

// Fix: Remove event listeners
class EventManager {
  constructor() {
    this.listeners = new WeakMap();
  }
  
  cleanup() {
    // Remove all listeners on cleanup
    this.emitter.removeAllListeners();
  }
}
```

## Database Problems

### Connection Pool Exhaustion

**Problem:** "Too many connections" or timeout errors

**Diagnosis:**
```sql
-- Check current connections (PostgreSQL)
SELECT count(*) FROM pg_stat_activity;

-- See active queries
SELECT pid, now() - pg_stat_activity.query_start AS duration, query 
FROM pg_stat_activity 
WHERE (now() - pg_stat_activity.query_start) > interval '5 minutes';
```

**Solutions:**

1. **Increase pool size:**
```javascript
// config/database.js
const pool = new Pool({
  host: process.env.DB_HOST,
  port: process.env.DB_PORT,
  database: process.env.DB_NAME,
  user: process.env.DB_USER,
  password: process.env.DB_PASSWORD,
  max: 20, // Increase from default 10
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
});
```

2. **Fix connection leaks:**
```javascript
// Bad - connection leak
async function getUser(id) {
  const client = await pool.connect();
  const result = await client.query('SELECT * FROM users WHERE id = $1', [id]);
  // Missing: client.release()
  return result.rows[0];
}

// Good - proper release
async function getUser(id) {
  const client = await pool.connect();
  try {
    const result = await client.query('SELECT * FROM users WHERE id = $1', [id]);
    return result.rows[0];
  } finally {
    client.release();
  }
}
```

### Slow Queries

**Problem:** Database queries taking too long

**Diagnosis:**

1. **Enable query logging:**
```sql
-- PostgreSQL
ALTER SYSTEM SET log_min_duration_statement = 1000; -- Log queries > 1 second
SELECT pg_reload_conf();
```

2. **Analyze query performance:**
```sql
EXPLAIN ANALYZE SELECT * FROM orders 
WHERE user_id = '123' 
AND created_at > '2024-01-01';
```

**Solutions:**

1. **Add indexes:**
```sql
-- Find missing indexes
SELECT 
  schemaname,
  tablename,
  attname,
  n_distinct,
  most_common_vals
FROM pg_stats
WHERE tablename = 'orders'
AND n_distinct > 100;

-- Create composite index
CREATE INDEX CONCURRENTLY idx_orders_user_created 
ON orders(user_id, created_at DESC);
```

2. **Optimize queries:**
```javascript
// Bad - N+1 query
const orders = await db.query('SELECT * FROM orders');
for (const order of orders) {
  order.items = await db.query('SELECT * FROM order_items WHERE order_id = $1', [order.id]);
}

// Good - Single query with join
const orders = await db.query(`
  SELECT 
    o.*,
    json_agg(oi.*) as items
  FROM orders o
  LEFT JOIN order_items oi ON oi.order_id = o.id
  GROUP BY o.id
`);
```

### Migration Failures

**Problem:** Database migrations fail or get stuck

**Solutions:**

1. **Check migration status:**
```bash
npm run migrate:status
```

2. **Fix stuck migrations:**
```sql
-- Check migration lock
SELECT * FROM migrations_lock;

-- Release lock if stuck
UPDATE migrations_lock SET is_locked = 0;
```

3. **Rollback failed migration:**
```bash
npm run migrate:rollback
# Fix the migration file
npm run migrate:latest
```

## API Issues

### CORS Errors

**Problem:** Browser shows CORS policy errors

```
Access to XMLHttpRequest at 'https://api.example.com' from origin 'https://app.example.com' 
has been blocked by CORS policy: No 'Access-Control-Allow-Origin' header is present.
```

**Solutions:**

1. **Configure CORS properly:**
```javascript
// Express.js
const cors = require('cors');

app.use(cors({
  origin: process.env.ALLOWED_ORIGINS?.split(',') || '*',
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization'],
}));
```

2. **Nginx configuration:**
```nginx
location /api {
    add_header 'Access-Control-Allow-Origin' '$http_origin' always;
    add_header 'Access-Control-Allow-Methods' 'GET, POST, PUT, DELETE, OPTIONS' always;
    add_header 'Access-Control-Allow-Headers' 'Authorization, Content-Type' always;
    add_header 'Access-Control-Allow-Credentials' 'true' always;
    
    if ($request_method = 'OPTIONS') {
        return 204;
    }
    
    proxy_pass http://backend;
}
```

### Rate Limiting Issues

**Problem:** Getting 429 Too Many Requests errors

**Solutions:**

1. **Check rate limit headers:**
```javascript
// Client-side handling
async function apiCall(url) {
  const response = await fetch(url);
  
  if (response.status === 429) {
    const retryAfter = response.headers.get('Retry-After');
    console.log(`Rate limited. Retry after ${retryAfter} seconds`);
    
    // Implement exponential backoff
    await sleep(retryAfter * 1000);
    return apiCall(url); // Retry
  }
  
  return response;
}
```

2. **Adjust rate limits:**
```javascript
// Server-side configuration
const rateLimit = require('express-rate-limit');

const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // Increase limit
  message: 'Too many requests from this IP',
  standardHeaders: true,
  legacyHeaders: false,
});
```

## Performance Problems

### Slow Page Load Times

**Problem:** Pages taking too long to load

**Diagnosis:**

1. **Use Chrome DevTools:**
   - Network tab: Check slow resources
   - Performance tab: Record page load
   - Lighthouse: Run performance audit

2. **Server-side timing:**
```javascript
// Add timing middleware
app.use((req, res, next) => {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - start;
    console.log(`${req.method} ${req.url} - ${duration}ms`);
  });
  
  next();
});
```

**Solutions:**

1. **Enable compression:**
```javascript
const compression = require('compression');
app.use(compression());
```

2. **Implement caching:**
```javascript
// Browser caching
app.use(express.static('public', {
  maxAge: '1d',
  etag: true
}));

// Redis caching
async function getCachedData(key) {
  const cached = await redis.get(key);
  if (cached) return JSON.parse(cached);
  
  const data = await fetchFromDatabase();
  await redis.setex(key, 300, JSON.stringify(data)); // 5 min cache
  return data;
}
```

### High CPU Usage

**Problem:** Application consuming excessive CPU

**Diagnosis:**

```bash
# Profile Node.js application
node --prof app.js
# Process the log
node --prof-process isolate-0xnnnnnnnnnnnn-v8.log > processed.txt

# Monitor in production
pm2 monit
```

**Common Causes:**

1. **Synchronous operations:**
```javascript
// Bad - Blocks event loop
function hashPassword(password) {
  return bcrypt.hashSync(password, 10);
}

// Good - Non-blocking
async function hashPassword(password) {
  return bcrypt.hash(password, 10);
}
```

2. **Inefficient algorithms:**
```javascript
// Bad - O(n²)
function findDuplicates(arr) {
  const duplicates = [];
  for (let i = 0; i < arr.length; i++) {
    for (let j = i + 1; j < arr.length; j++) {
      if (arr[i] === arr[j]) duplicates.push(arr[i]);
    }
  }
  return duplicates;
}

// Good - O(n)
function findDuplicates(arr) {
  const seen = new Set();
  const duplicates = new Set();
  
  for (const item of arr) {
    if (seen.has(item)) {
      duplicates.add(item);
    }
    seen.add(item);
  }
  
  return Array.from(duplicates);
}
```

## Docker and Container Issues

### Container Keeps Restarting

**Problem:** Docker container in restart loop

**Diagnosis:**
```bash
# Check container logs
docker logs --tail 50 -f container_name

# Inspect container
docker inspect container_name

# Check events
docker events --filter container=container_name
```

**Solutions:**

1. **Fix health check:**
```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=40s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1
```

2. **Increase memory limits:**
```yaml
# docker-compose.yml
services:
  app:
    mem_limit: 1g
    memswap_limit: 2g
```

### Build Failures

**Problem:** Docker build fails with various errors

**Solutions:**

1. **Clear Docker cache:**
```bash
docker builder prune -af
docker system prune -af
```

2. **Multi-stage build issues:**
```dockerfile
# Ensure files are copied from correct stage
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

FROM node:18-alpine
WORKDIR /app
# Copy from builder stage
COPY --from=builder /app/node_modules ./node_modules
COPY . .
```

## Authentication and Security

### JWT Token Issues

**Problem:** "Invalid token" or "Token expired" errors

**Debugging:**

```javascript
// Decode token to check claims
const jwt = require('jsonwebtoken');

function debugToken(token) {
  try {
    const decoded = jwt.decode(token, { complete: true });
    console.log('Header:', decoded.header);
    console.log('Payload:', decoded.payload);
    
    // Verify expiration
    const now = Math.floor(Date.now() / 1000);
    if (decoded.payload.exp < now) {
      console.log('Token expired:', new Date(decoded.payload.exp * 1000));
    }
  } catch (error) {
    console.error('Invalid token format:', error);
  }
}
```

**Solutions:**

1. **Handle token refresh:**
```javascript
// Client-side token refresh
class AuthService {
  async makeRequest(url, options = {}) {
    let response = await fetch(url, {
      ...options,
      headers: {
        ...options.headers,
        Authorization: `Bearer ${this.getAccessToken()}`
      }
    });
    
    if (response.status === 401) {
      // Try refreshing token
      await this.refreshAccessToken();
      
      // Retry request
      response = await fetch(url, {
        ...options,
        headers: {
          ...options.headers,
          Authorization: `Bearer ${this.getAccessToken()}`
        }
      });
    }
    
    return response;
  }
}
```

### Session Management Problems

**Problem:** Users getting logged out unexpectedly

**Common Causes:**

1. **Redis connection issues:**
```javascript
// Add Redis error handling
redis.on('error', (err) => {
  console.error('Redis error:', err);
  // Fallback to memory store temporarily
});

redis.on('reconnecting', () => {
  console.log('Reconnecting to Redis...');
});
```

2. **Session configuration:**
```javascript
app.use(session({
  store: new RedisStore({ client: redis }),
  secret: process.env.SESSION_SECRET,
  resave: false,
  saveUninitialized: false,
  cookie: {
    secure: process.env.NODE_ENV === 'production', // HTTPS only
    httpOnly: true,
    maxAge: 1000 * 60 * 60 * 24 * 7, // 7 days
    sameSite: 'lax'
  }
}));
```

## Payment Gateway Problems

### Payment Failures

**Problem:** Payments failing with various errors

**Debugging Steps:**

1. **Check webhook logs:**
```javascript
// Log all webhook events
app.post('/webhooks/stripe', (req, res) => {
  console.log('Webhook received:', {
    type: req.body.type,
    id: req.body.id,
    created: new Date(req.body.created * 1000)
  });
  
  // Process webhook...
});
```

2. **Verify webhook signatures:**
```javascript
const stripe = require('stripe')(process.env.STRIPE_SECRET_KEY);

function verifyWebhookSignature(payload, signature, secret) {
  try {
    return stripe.webhooks.constructEvent(payload, signature, secret);
  } catch (err) {
    console.error('Webhook signature verification failed:', err.message);
    return null;
  }
}
```

### Idempotency Issues

**Problem:** Duplicate charges or orders

**Solution:**

```javascript
// Implement idempotency
class PaymentService {
  async processPayment(orderId, amount, idempotencyKey) {
    // Check if already processed
    const existing = await redis.get(`payment:${idempotencyKey}`);
    if (existing) {
      return JSON.parse(existing);
    }
    
    // Process payment
    const result = await stripe.charges.create({
      amount: amount * 100,
      currency: 'usd',
      description: `Order ${orderId}`,
      idempotency_key: idempotencyKey
    });
    
    // Cache result
    await redis.setex(
      `payment:${idempotencyKey}`,
      86400, // 24 hours
      JSON.stringify(result)
    );
    
    return result;
  }
}
```

## Search and Indexing

### Elasticsearch Issues

**Problem:** Search returning no or incorrect results

**Diagnosis:**

```bash
# Check cluster health
curl -X GET "localhost:9200/_cluster/health?pretty"

# Check index mapping
curl -X GET "localhost:9200/products/_mapping?pretty"

# Test query
curl -X GET "localhost:9200/products/_search?pretty" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match": {
      "name": "laptop"
    }
  }
}'
```

**Solutions:**

1. **Reindex data:**
```javascript
async function reindexProducts() {
  // Delete old index
  await elastic.indices.delete({ index: 'products' });
  
  // Create with proper mapping
  await elastic.indices.create({
    index: 'products',
    body: {
      mappings: {
        properties: {
          name: { type: 'text', analyzer: 'standard' },
          description: { type: 'text' },
          price: { type: 'float' },
          category: { type: 'keyword' },
          created_at: { type: 'date' }
        }
      }
    }
  });
  
  // Bulk index
  const products = await getProductsFromDB();
  const body = products.flatMap(doc => [
    { index: { _index: 'products', _id: doc.id } },
    doc
  ]);
  
  await elastic.bulk({ refresh: true, body });
}
```

### Search Relevance Issues

**Problem:** Search results not relevant

**Solutions:**

1. **Improve analyzers:**
```json
{
  "settings": {
    "analysis": {
      "analyzer": {
        "product_analyzer": {
          "tokenizer": "standard",
          "filter": ["lowercase", "stop", "synonym", "stemmer"]
        }
      },
      "filter": {
        "synonym": {
          "type": "synonym",
          "synonyms": [
            "laptop,notebook",
            "phone,mobile,smartphone"
          ]
        }
      }
    }
  }
}
```

2. **Boost important fields:**
```javascript
const searchQuery = {
  query: {
    multi_match: {
      query: searchTerm,
      fields: [
        "name^3",        // 3x boost
        "brand^2",       // 2x boost
        "description"    // 1x boost
      ],
      type: "best_fields",
      fuzziness: "AUTO"
    }
  }
};
```

## Debugging Techniques

### Remote Debugging

**Setup remote debugging:**

```bash
# Start Node.js with debug flag
node --inspect=0.0.0.0:9229 server.js

# For Docker
docker run -p 9229:9229 -e NODE_OPTIONS='--inspect=0.0.0.0:9229' myapp
```

**Connect from VS Code:**
```json
// .vscode/launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "node",
      "request": "attach",
      "name": "Attach to Remote",
      "address": "localhost",
      "port": 9229,
      "remoteRoot": "/app",
      "localRoot": "${workspaceFolder}"
    }
  ]
}
```

### Logging Best Practices

```javascript
// Structured logging with context
const winston = require('winston');

const logger = winston.createLogger({
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  defaultMeta: { service: 'api' },
  transports: [
    new winston.transports.File({ filename: 'error.log', level: 'error' }),
    new winston.transports.File({ filename: 'combined.log' })
  ]
});

// Usage with context
logger.info('Order processed', {
  orderId: order.id,
  userId: user.id,
  amount: order.total,
  processingTime: Date.now() - startTime
});
```

### Performance Profiling

```javascript
// CPU profiling
const v8Profiler = require('v8-profiler-next');

// Start profiling
v8Profiler.startProfiling('CPU profile');

// Your code here...

// Stop profiling
const profile = v8Profiler.stopProfiling();
profile.export((error, result) => {
  fs.writeFileSync('cpu-profile.cpuprofile', result);
  profile.delete();
});

// Memory profiling
const heapdump = require('heapdump');

// Trigger heap snapshot
heapdump.writeSnapshot((err, filename) => {
  console.log('Heap snapshot written to', filename);
});
```

## Getting Additional Help

If you're still experiencing issues:

1. **Search existing issues:** [GitHub Issues](https://github.com/example/ecommerce/issues)
2. **Ask the community:** [Discord Server](https://discord.gg/example)
3. **Contact support:** support@example.com
4. **Emergency hotline:** +1-555-HELP (for production issues)

When reporting issues, please include:
- Error messages and stack traces
- Steps to reproduce
- Environment details (OS, Node version, etc.)
- Relevant configuration files
- Log excerpts

Remember to sanitize any sensitive information before sharing!