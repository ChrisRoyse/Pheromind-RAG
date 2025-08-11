# Complex Mixed Content Document

This document combines all markdown elements to test comprehensive chunking scenarios.

## Executive Summary

> "The future of software development lies in the seamless integration of AI-powered tools with human creativity and domain expertise."
> 
> — Tech Industry Report 2024

This comprehensive guide covers the implementation of a modern development workflow that incorporates:

1. **Automated Testing Pipelines**
2. **AI-Assisted Code Review**
3. **Performance Monitoring**
4. **Security Compliance**

### Key Performance Indicators

| Metric | Current | Target | Status |
|--------|---------|--------|---------|
| Code Coverage | 78% | 85% | 🟡 In Progress |
| Build Time | 12 min | 8 min | 🔴 Needs Work |
| Security Score | 92% | 95% | 🟡 In Progress |
| User Satisfaction | 4.2/5 | 4.5/5 | 🟢 On Track |

## Chapter 1: Architecture Overview

### System Components

The architecture consists of several interconnected components:

```mermaid
graph TD
    A[Frontend Application] --> B[API Gateway]
    B --> C[Authentication Service]
    B --> D[Business Logic Service]
    B --> E[Data Access Layer]
    E --> F[Primary Database]
    E --> G[Cache Layer]
    D --> H[External APIs]
    D --> I[Message Queue]
    I --> J[Background Workers]
```

#### Frontend Layer

The frontend is built using React with TypeScript:

```typescript
interface UserProfile {
  id: string;
  username: string;
  email: string;
  preferences: UserPreferences;
  permissions: Permission[];
}

interface UserPreferences {
  theme: 'light' | 'dark' | 'auto';
  language: string;
  notifications: NotificationSettings;
}

interface NotificationSettings {
  email: boolean;
  push: boolean;
  sms: boolean;
  frequency: 'immediate' | 'daily' | 'weekly';
}

class UserProfileManager {
  private apiClient: ApiClient;
  private cacheManager: CacheManager;
  
  constructor(apiClient: ApiClient, cacheManager: CacheManager) {
    this.apiClient = apiClient;
    this.cacheManager = cacheManager;
  }
  
  async getUserProfile(userId: string): Promise<UserProfile> {
    const cacheKey = `user_profile_${userId}`;
    
    // Try cache first
    const cached = await this.cacheManager.get<UserProfile>(cacheKey);
    if (cached) {
      return cached;
    }
    
    // Fetch from API
    const profile = await this.apiClient.get<UserProfile>(`/users/${userId}`);
    
    // Cache for future requests
    await this.cacheManager.set(cacheKey, profile, { ttl: 300 }); // 5 minutes
    
    return profile;
  }
  
  async updateUserPreferences(
    userId: string, 
    preferences: Partial<UserPreferences>
  ): Promise<void> {
    await this.apiClient.patch(`/users/${userId}/preferences`, preferences);
    
    // Invalidate cache
    await this.cacheManager.delete(`user_profile_${userId}`);
    
    // Emit update event
    this.eventBus.emit('user_preferences_updated', { userId, preferences });
  }
}
```

#### Backend Services

##### Authentication Service

```python
from typing import Optional, Dict, Any
from datetime import datetime, timedelta
import jwt
import bcrypt
from fastapi import FastAPI, HTTPException, Depends
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
import redis
import asyncpg

app = FastAPI(title="Authentication Service", version="1.0.0")
security = HTTPBearer()

class AuthenticationService:
    def __init__(self, db_pool: asyncpg.Pool, redis_client: redis.Redis):
        self.db_pool = db_pool
        self.redis = redis_client
        self.jwt_secret = os.getenv("JWT_SECRET")
        self.jwt_algorithm = "HS256"
        self.access_token_expire = timedelta(hours=1)
        self.refresh_token_expire = timedelta(days=30)
    
    async def authenticate_user(self, username: str, password: str) -> Optional[Dict[str, Any]]:
        """Authenticate user credentials and return user data if valid."""
        async with self.db_pool.acquire() as connection:
            user = await connection.fetchrow(
                "SELECT id, username, email, password_hash, is_active FROM users WHERE username = $1",
                username
            )
            
            if not user or not user['is_active']:
                return None
            
            if not bcrypt.checkpw(password.encode('utf-8'), user['password_hash'].encode('utf-8')):
                return None
            
            # Update last login
            await connection.execute(
                "UPDATE users SET last_login_at = NOW() WHERE id = $1",
                user['id']
            )
            
            return {
                'id': user['id'],
                'username': user['username'],
                'email': user['email']
            }
    
    def generate_tokens(self, user_data: Dict[str, Any]) -> Dict[str, str]:
        """Generate access and refresh tokens for user."""
        now = datetime.utcnow()
        
        # Access token payload
        access_payload = {
            'user_id': user_data['id'],
            'username': user_data['username'],
            'exp': now + self.access_token_expire,
            'iat': now,
            'type': 'access'
        }
        
        # Refresh token payload
        refresh_payload = {
            'user_id': user_data['id'],
            'exp': now + self.refresh_token_expire,
            'iat': now,
            'type': 'refresh'
        }
        
        access_token = jwt.encode(access_payload, self.jwt_secret, algorithm=self.jwt_algorithm)
        refresh_token = jwt.encode(refresh_payload, self.jwt_secret, algorithm=self.jwt_algorithm)
        
        # Store refresh token in Redis
        self.redis.setex(
            f"refresh_token:{user_data['id']}", 
            int(self.refresh_token_expire.total_seconds()),
            refresh_token
        )
        
        return {
            'access_token': access_token,
            'refresh_token': refresh_token,
            'token_type': 'Bearer',
            'expires_in': int(self.access_token_expire.total_seconds())
        }

@app.post("/auth/login")
async def login(credentials: LoginRequest, auth_service: AuthenticationService = Depends()):
    user = await auth_service.authenticate_user(credentials.username, credentials.password)
    
    if not user:
        raise HTTPException(status_code=401, detail="Invalid credentials")
    
    tokens = auth_service.generate_tokens(user)
    
    return {
        'user': user,
        'tokens': tokens
    }
```

### Database Design

Our database schema follows normalization principles while optimizing for read performance:

```sql
-- Core user management tables
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE,
    profile_data JSONB DEFAULT '{}'
);

-- User sessions and authentication
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(255) UNIQUE NOT NULL,
    refresh_token VARCHAR(255) UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_accessed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT,
    is_active BOOLEAN DEFAULT true
);

-- Application data
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(50) DEFAULT 'active' CHECK (status IN ('active', 'archived', 'deleted')),
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE project_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'admin', 'member', 'viewer')),
    permissions JSONB DEFAULT '{}',
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(project_id, user_id)
);

-- Optimized indexes
CREATE INDEX idx_users_email_active ON users(email) WHERE is_active = true;
CREATE INDEX idx_users_username_active ON users(username) WHERE is_active = true;
CREATE INDEX idx_user_sessions_token ON user_sessions(session_token);
CREATE INDEX idx_user_sessions_user_active ON user_sessions(user_id) WHERE is_active = true;
CREATE INDEX idx_projects_owner ON projects(owner_id);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_project_members_project ON project_members(project_id);
CREATE INDEX idx_project_members_user ON project_members(user_id);
```

## Chapter 2: Development Workflow

### Git Workflow Strategy

We use a modified GitFlow approach with the following branch structure:

1. **main** - Production-ready code
2. **develop** - Integration branch for features  
3. **feature/** - New feature development
4. **hotfix/** - Critical production fixes
5. **release/** - Release preparation

#### Branch Protection Rules

| Branch | Protection Rules | Required Checks |
|--------|-----------------|-----------------|
| main | • Require PR reviews (2+)<br>• Dismiss stale reviews<br>• Require status checks<br>• Restrict pushes | • CI/CD Pipeline<br>• Security Scan<br>• Code Coverage<br>• Performance Tests |
| develop | • Require PR reviews (1+)<br>• Require status checks | • Unit Tests<br>• Integration Tests<br>• Linting |
| feature/* | • Require status checks | • Unit Tests<br>• Linting |

### Continuous Integration Pipeline

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  NODE_VERSION: '18'
  PYTHON_VERSION: '3.11'
  DOCKER_REGISTRY: 'ghcr.io'

jobs:
  # Code Quality Checks
  lint-and-format:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run ESLint
        run: npm run lint
      
      - name: Run Prettier
        run: npm run format:check
      
      - name: TypeScript Check
        run: npm run type-check

  # Security Scanning
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          format: 'sarif'
          output: 'trivy-results.sarif'
      
      - name: Upload Trivy scan results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: 'trivy-results.sarif'
      
      - name: Run npm audit
        run: npm audit --audit-level moderate

  # Unit Tests
  unit-tests:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: [16, 18, 20]
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js ${{ matrix.node-version }}
        uses: actions/setup-node@v3
        with:
          node-version: ${{ matrix.node-version }}
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run unit tests
        run: npm run test:unit -- --coverage
        env:
          CI: true
      
      - name: Upload coverage reports
        uses: codecov/codecov-action@v3
        with:
          file: ./coverage/lcov.info
          flags: unit-tests
          name: codecov-${{ matrix.node-version }}

  # Integration Tests
  integration-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: password
          POSTGRES_DB: test_db
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run database migrations
        run: npm run db:migrate
        env:
          DATABASE_URL: postgresql://postgres:password@localhost:5432/test_db
      
      - name: Run integration tests
        run: npm run test:integration
        env:
          DATABASE_URL: postgresql://postgres:password@localhost:5432/test_db
          REDIS_URL: redis://localhost:6379
```

### Testing Strategy

#### Test Pyramid Implementation

```
         /\
        /E2E\     <- 10% (Cypress, Playwright)
       /------\
      /API    \   <- 20% (Supertest, Postman)
     /--------\
    /Component\ <- 30% (React Testing Library)
   /----------\
  /   Unit     \ <- 40% (Jest, Vitest)
 /--------------\
```

##### Unit Testing Example

```javascript
// src/services/__tests__/UserService.test.js
import { UserService } from '../UserService';
import { ApiClient } from '../ApiClient';
import { CacheManager } from '../CacheManager';

// Mock dependencies
jest.mock('../ApiClient');
jest.mock('../CacheManager');

describe('UserService', () => {
  let userService;
  let mockApiClient;
  let mockCacheManager;

  beforeEach(() => {
    mockApiClient = new ApiClient();
    mockCacheManager = new CacheManager();
    userService = new UserService(mockApiClient, mockCacheManager);
    
    // Reset mocks
    jest.clearAllMocks();
  });

  describe('getUserProfile', () => {
    const userId = 'user-123';
    const mockProfile = {
      id: userId,
      username: 'testuser',
      email: 'test@example.com',
      preferences: { theme: 'dark' }
    };

    it('should return cached profile when available', async () => {
      // Arrange
      mockCacheManager.get.mockResolvedValue(mockProfile);

      // Act
      const result = await userService.getUserProfile(userId);

      // Assert
      expect(result).toEqual(mockProfile);
      expect(mockCacheManager.get).toHaveBeenCalledWith(`user_profile_${userId}`);
      expect(mockApiClient.get).not.toHaveBeenCalled();
    });

    it('should fetch from API when not cached', async () => {
      // Arrange
      mockCacheManager.get.mockResolvedValue(null);
      mockApiClient.get.mockResolvedValue(mockProfile);

      // Act
      const result = await userService.getUserProfile(userId);

      // Assert
      expect(result).toEqual(mockProfile);
      expect(mockCacheManager.get).toHaveBeenCalledWith(`user_profile_${userId}`);
      expect(mockApiClient.get).toHaveBeenCalledWith(`/users/${userId}`);
      expect(mockCacheManager.set).toHaveBeenCalledWith(
        `user_profile_${userId}`,
        mockProfile,
        { ttl: 300 }
      );
    });

    it('should handle API errors gracefully', async () => {
      // Arrange
      const apiError = new Error('API Error');
      mockCacheManager.get.mockResolvedValue(null);
      mockApiClient.get.mockRejectedValue(apiError);

      // Act & Assert
      await expect(userService.getUserProfile(userId))
        .rejects.toThrow('API Error');
      expect(mockCacheManager.set).not.toHaveBeenCalled();
    });
  });

  describe('updateUserPreferences', () => {
    const userId = 'user-123';
    const preferences = { theme: 'light', language: 'es' };

    it('should update preferences and invalidate cache', async () => {
      // Arrange
      mockApiClient.patch.mockResolvedValue(undefined);

      // Act
      await userService.updateUserPreferences(userId, preferences);

      // Assert
      expect(mockApiClient.patch).toHaveBeenCalledWith(
        `/users/${userId}/preferences`,
        preferences
      );
      expect(mockCacheManager.delete).toHaveBeenCalledWith(`user_profile_${userId}`);
    });
  });
});
```

## Chapter 3: Performance Optimization

### Monitoring and Metrics

#### Application Performance Monitoring

We track the following key metrics:

| Metric Category | KPI | Target | Current | Trend |
|-----------------|-----|--------|---------|--------|
| **Response Time** | | | | |
| API Response (P95) | < 500ms | 450ms | 312ms | ⬇️ -15% |
| Page Load Time | < 2s | 1.5s | 1.8s | ⬇️ -8% |
| Database Queries (P95) | < 100ms | 80ms | 65ms | ⬇️ -12% |
| **Throughput** | | | | |
| Requests per Second | > 1000 | 1200 | 1350 | ⬆️ +18% |
| Concurrent Users | > 500 | 800 | 920 | ⬆️ +12% |
| **Error Rates** | | | | |
| HTTP 5xx Errors | < 0.1% | 0.05% | 0.03% | ⬇️ -40% |
| HTTP 4xx Errors | < 2% | 1.5% | 1.2% | ⬇️ -20% |
| **Resource Usage** | | | | |
| CPU Utilization | < 70% | 60% | 45% | ⬇️ -25% |
| Memory Usage | < 80% | 70% | 58% | ⬇️ -18% |
| Disk I/O | < 80% | 70% | 42% | ⬇️ -30% |

#### Performance Optimization Techniques

1. **Database Optimization**
   - Query indexing strategy
   - Connection pooling
   - Read replicas for scaling
   - Query result caching

2. **Application-Level Caching**
   ```typescript
   class CacheManager {
     private redis: RedisClient;
     private localCache: LRUCache<string, any>;
     
     constructor() {
       this.redis = new RedisClient(process.env.REDIS_URL);
       this.localCache = new LRUCache({ 
         max: 1000,
         ttl: 1000 * 60 * 5 // 5 minutes
       });
     }
     
     async get<T>(key: string): Promise<T | null> {
       // L1 Cache: In-memory
       if (this.localCache.has(key)) {
         return this.localCache.get(key) as T;
       }
       
       // L2 Cache: Redis
       const redisValue = await this.redis.get(key);
       if (redisValue) {
         const parsed = JSON.parse(redisValue) as T;
         this.localCache.set(key, parsed);
         return parsed;
       }
       
       return null;
     }
     
     async set<T>(key: string, value: T, options?: { ttl?: number }): Promise<void> {
       const serialized = JSON.stringify(value);
       
       // Set in both caches
       this.localCache.set(key, value);
       
       if (options?.ttl) {
         await this.redis.setex(key, options.ttl, serialized);
       } else {
         await this.redis.set(key, serialized);
       }
     }
   }
   ```

3. **Frontend Optimization**
   - Code splitting and lazy loading
   - Image optimization and compression  
   - Service worker for offline functionality
   - Bundle size reduction

### Load Testing Results

#### Scenario 1: Normal Load

```bash
# Artillery.js load test configuration
config:
  target: 'https://api.example.com'
  phases:
    - duration: 300
      arrivalRate: 10
      name: "Ramp up"
    - duration: 600  
      arrivalRate: 50
      name: "Sustained load"
    - duration: 300
      arrivalRate: 100
      name: "Peak load"

scenarios:
  - name: "User journey"
    weight: 70
    flow:
      - post:
          url: "/auth/login"
          json:
            username: "testuser"
            password: "password123"
          capture:
            - json: "$.tokens.access_token"
              as: "token"
      - get:
          url: "/api/users/me"
          headers:
            Authorization: "Bearer {{ token }}"
      - get:
          url: "/api/projects"
          headers:
            Authorization: "Bearer {{ token }}"
```

**Results:**
- **Average Response Time**: 156ms
- **95th Percentile**: 312ms  
- **99th Percentile**: 845ms
- **Error Rate**: 0.02%
- **Peak RPS**: 2,150

#### Scenario 2: Stress Test

| Metric | Target | Result | Status |
|--------|--------|--------|--------|
| Max RPS | 3,000 | 2,850 | ⚠️ 95% |
| Response Time (P95) | < 1s | 890ms | ✅ Pass |
| Error Rate | < 1% | 0.15% | ✅ Pass |
| Memory Usage | < 4GB | 3.2GB | ✅ Pass |
| CPU Usage | < 85% | 78% | ✅ Pass |

## Chapter 4: Security Implementation

### Security Checklist

- [x] **Authentication & Authorization**
  - [x] JWT token implementation
  - [x] Refresh token rotation
  - [x] Role-based access control (RBAC)
  - [x] Multi-factor authentication (MFA)
  
- [x] **Data Protection**
  - [x] Data encryption at rest
  - [x] Data encryption in transit (TLS 1.3)
  - [x] PII data anonymization
  - [x] GDPR compliance measures
  
- [x] **Input Validation**
  - [x] SQL injection prevention
  - [x] XSS protection
  - [x] CSRF tokens
  - [x] Rate limiting
  
- [x] **Infrastructure Security**
  - [x] WAF configuration
  - [x] DDoS protection
  - [x] Network segmentation
  - [x] Container security scanning

### Security Monitoring Dashboard

| Security Metric | Current | Threshold | Alert Level |
|------------------|---------|-----------|-------------|
| Failed Login Attempts | 23/hour | 100/hour | 🟢 Normal |
| Suspicious API Calls | 5/hour | 50/hour | 🟢 Normal |
| Malformed Requests | 12/hour | 200/hour | 🟢 Normal |
| Rate Limit Violations | 8/hour | 100/hour | 🟢 Normal |
| Certificate Expiry | 87 days | 30 days | 🟢 Normal |
| Vulnerability Scan Score | 98/100 | 95/100 | 🟢 Normal |

### Incident Response Playbook

#### Security Incident Classification

| Severity | Definition | Response Time | Escalation |
|----------|------------|---------------|------------|
| **P0 - Critical** | Data breach, system compromise | < 15 min | CISO, CTO |
| **P1 - High** | Service disruption, auth bypass | < 30 min | Security Team Lead |
| **P2 - Medium** | Vulnerability found, failed audit | < 2 hours | Security Engineer |
| **P3 - Low** | Policy violation, minor config | < 24 hours | DevOps Team |

## Conclusion

This document demonstrates the comprehensive testing capabilities needed for markdown chunking:

### Content Types Covered

1. **Headers**: Multiple hierarchy levels (H1-H6)
2. **Code Blocks**: Multiple languages with syntax highlighting
3. **Tables**: Simple, complex, and nested data structures
4. **Lists**: Ordered, unordered, nested, and task lists
5. **Mixed Content**: Complex combinations requiring careful parsing
6. **Special Elements**: Quotes, links, images, math formulas
7. **Large Content**: Performance testing with substantial text

### Expected Chunking Behavior

The chunking system should:
- Preserve code block integrity
- Maintain table structure across chunks
- Handle nested lists appropriately
- Keep related content together (headers with their sections)
- Manage large documents efficiently
- Serialize/deserialize reliably for storage

### Test Coverage Goals

- **Functional**: All markdown elements parsed correctly
- **Performance**: Large documents processed within time limits  
- **Edge Cases**: Malformed content handled gracefully
- **Integration**: Works with search and storage systems
- **Reliability**: Consistent results across multiple runs

---

*This comprehensive test document contains 2,847 lines and tests all major markdown parsing scenarios.*