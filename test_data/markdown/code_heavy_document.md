# Code-Heavy Documentation

This document contains extensive code examples to test code block preservation.

## JavaScript Examples

### Basic Functions

```javascript
// Simple function
function greet(name) {
    return `Hello, ${name}!`;
}

// Arrow function
const multiply = (a, b) => a * b;

// Async function
async function fetchData(url) {
    try {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    } catch (error) {
        console.error('Fetch error:', error);
        throw error;
    }
}
```

### React Component

```jsx
import React, { useState, useEffect } from 'react';

const UserProfile = ({ userId }) => {
    const [user, setUser] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        const loadUser = async () => {
            try {
                setLoading(true);
                const userData = await fetchUser(userId);
                setUser(userData);
            } catch (err) {
                setError(err.message);
            } finally {
                setLoading(false);
            }
        };

        loadUser();
    }, [userId]);

    if (loading) return <div>Loading...</div>;
    if (error) return <div>Error: {error}</div>;
    if (!user) return <div>User not found</div>;

    return (
        <div className="user-profile">
            <h2>{user.name}</h2>
            <p>Email: {user.email}</p>
            <p>Joined: {new Date(user.joinDate).toLocaleDateString()}</p>
        </div>
    );
};

export default UserProfile;
```

## Python Examples

### Class Definition

```python
from typing import List, Optional, Dict, Any
from datetime import datetime
import asyncio
import logging

logger = logging.getLogger(__name__)

class DataProcessor:
    """
    A class for processing various data types with async capabilities.
    """
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.processed_count = 0
        self._cache = {}
        
    async def process_items(self, items: List[Dict]) -> List[Dict]:
        """
        Process a list of items asynchronously.
        
        Args:
            items: List of dictionaries to process
            
        Returns:
            List of processed items
            
        Raises:
            ValueError: If items list is empty
            ProcessingError: If processing fails
        """
        if not items:
            raise ValueError("Items list cannot be empty")
            
        logger.info(f"Processing {len(items)} items")
        
        # Process in batches
        batch_size = self.config.get('batch_size', 10)
        results = []
        
        for i in range(0, len(items), batch_size):
            batch = items[i:i + batch_size]
            batch_results = await self._process_batch(batch)
            results.extend(batch_results)
            
        self.processed_count += len(results)
        logger.info(f"Successfully processed {len(results)} items")
        
        return results
    
    async def _process_batch(self, batch: List[Dict]) -> List[Dict]:
        """Process a single batch of items."""
        tasks = [self._process_single_item(item) for item in batch]
        return await asyncio.gather(*tasks, return_exceptions=True)
    
    async def _process_single_item(self, item: Dict) -> Dict:
        """Process a single item with caching."""
        item_id = item.get('id')
        
        # Check cache first
        if item_id in self._cache:
            return self._cache[item_id]
        
        # Simulate processing time
        await asyncio.sleep(0.1)
        
        processed_item = {
            'id': item_id,
            'processed_at': datetime.utcnow().isoformat(),
            'data': item.get('data', {}),
            'status': 'processed'
        }
        
        # Cache result
        self._cache[item_id] = processed_item
        
        return processed_item
    
    def get_stats(self) -> Dict[str, Any]:
        """Get processing statistics."""
        return {
            'processed_count': self.processed_count,
            'cache_size': len(self._cache),
            'config': self.config
        }
```

## Rust Examples

### Struct and Implementation

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

#[derive(Debug)]
pub struct UserService {
    users: Arc<RwLock<HashMap<u64, User>>>,
    next_id: Arc<Mutex<u64>>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
    
    pub async fn create_user(
        &self, 
        username: String, 
        email: String
    ) -> Result<User> {
        // Validate input
        if username.trim().is_empty() {
            return Err(anyhow::anyhow!("Username cannot be empty"));
        }
        
        if email.trim().is_empty() || !email.contains('@') {
            return Err(anyhow::anyhow!("Invalid email address"));
        }
        
        // Generate new ID
        let id = {
            let mut next_id = self.next_id.lock()
                .context("Failed to acquire next_id lock")?;
            let current_id = *next_id;
            *next_id += 1;
            current_id
        };
        
        // Create user
        let user = User {
            id,
            username,
            email,
            created_at: chrono::Utc::now(),
            is_active: true,
        };
        
        // Store user
        {
            let mut users = self.users.write().await;
            users.insert(id, user.clone());
        }
        
        Ok(user)
    }
    
    pub async fn get_user(&self, id: u64) -> Option<User> {
        let users = self.users.read().await;
        users.get(&id).cloned()
    }
    
    pub async fn list_users(&self) -> Vec<User> {
        let users = self.users.read().await;
        users.values().cloned().collect()
    }
    
    pub async fn update_user(&self, id: u64, updates: UserUpdate) -> Result<User> {
        let mut users = self.users.write().await;
        
        let user = users.get_mut(&id)
            .context("User not found")?;
        
        if let Some(username) = updates.username {
            if username.trim().is_empty() {
                return Err(anyhow::anyhow!("Username cannot be empty"));
            }
            user.username = username;
        }
        
        if let Some(email) = updates.email {
            if email.trim().is_empty() || !email.contains('@') {
                return Err(anyhow::anyhow!("Invalid email address"));
            }
            user.email = email;
        }
        
        if let Some(is_active) = updates.is_active {
            user.is_active = is_active;
        }
        
        Ok(user.clone())
    }
}

#[derive(Debug, Default)]
pub struct UserUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
    pub is_active: Option<bool>,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_creation() {
        let service = UserService::new();
        
        let user = service.create_user(
            "testuser".to_string(),
            "test@example.com".to_string()
        ).await.expect("Failed to create user");
        
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
        assert_eq!(user.id, 1);
    }
    
    #[tokio::test]
    async fn test_user_retrieval() {
        let service = UserService::new();
        
        let created_user = service.create_user(
            "testuser".to_string(),
            "test@example.com".to_string()
        ).await.expect("Failed to create user");
        
        let retrieved_user = service.get_user(created_user.id).await;
        assert!(retrieved_user.is_some());
        assert_eq!(retrieved_user.unwrap().username, "testuser");
    }
}
```

## Shell Scripts

### Deployment Script

```bash
#!/bin/bash

set -euo pipefail

# Configuration
APP_NAME="my-app"
DOCKER_IMAGE="$APP_NAME:latest"
CONTAINER_NAME="$APP_NAME-container"
PORT="3000"
ENV_FILE=".env.production"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
    exit 1
}

check_prerequisites() {
    log "Checking prerequisites..."
    
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed"
    fi
    
    if [[ ! -f "$ENV_FILE" ]]; then
        warn "Environment file $ENV_FILE not found"
        read -p "Continue without environment file? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

build_image() {
    log "Building Docker image..."
    
    if ! docker build -t "$DOCKER_IMAGE" .; then
        error "Failed to build Docker image"
    fi
    
    log "Docker image built successfully"
}

stop_existing_container() {
    if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
        log "Stopping existing container..."
        docker stop "$CONTAINER_NAME"
        docker rm "$CONTAINER_NAME"
    fi
}

deploy_container() {
    log "Deploying new container..."
    
    local docker_run_cmd="docker run -d --name $CONTAINER_NAME -p $PORT:$PORT"
    
    if [[ -f "$ENV_FILE" ]]; then
        docker_run_cmd="$docker_run_cmd --env-file $ENV_FILE"
    fi
    
    docker_run_cmd="$docker_run_cmd $DOCKER_IMAGE"
    
    if ! eval "$docker_run_cmd"; then
        error "Failed to start container"
    fi
    
    log "Container deployed successfully"
}

health_check() {
    log "Performing health check..."
    
    local max_attempts=30
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        if curl -f "http://localhost:$PORT/health" &> /dev/null; then
            log "Health check passed"
            return 0
        fi
        
        log "Health check attempt $attempt/$max_attempts failed, retrying..."
        sleep 2
        ((attempt++))
    done
    
    error "Health check failed after $max_attempts attempts"
}

cleanup() {
    log "Cleaning up old images..."
    docker image prune -f
}

main() {
    log "Starting deployment of $APP_NAME..."
    
    check_prerequisites
    build_image
    stop_existing_container
    deploy_container
    health_check
    cleanup
    
    log "Deployment completed successfully!"
    log "Application is running at http://localhost:$PORT"
}

# Run main function
main "$@"
```

## SQL Examples

### Database Schema

```sql
-- Users table
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE
);

-- Posts table
CREATE TABLE posts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'archived')),
    published_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Comments table
CREATE TABLE comments (
    id BIGSERIAL PRIMARY KEY,
    post_id BIGINT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id BIGINT REFERENCES comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    is_approved BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_status ON posts(status);
CREATE INDEX idx_posts_published_at ON posts(published_at);
CREATE INDEX idx_comments_post_id ON comments(post_id);
CREATE INDEX idx_comments_user_id ON comments(user_id);
CREATE INDEX idx_comments_parent_id ON comments(parent_id);

-- Functions and triggers
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at 
    BEFORE UPDATE ON users 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_posts_updated_at 
    BEFORE UPDATE ON posts 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_comments_updated_at 
    BEFORE UPDATE ON comments 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Complex query examples
WITH popular_posts AS (
    SELECT 
        p.id,
        p.title,
        p.user_id,
        u.username,
        COUNT(c.id) as comment_count,
        p.published_at
    FROM posts p
    JOIN users u ON p.user_id = u.id
    LEFT JOIN comments c ON p.id = c.post_id
    WHERE p.status = 'published'
        AND p.published_at >= NOW() - INTERVAL '30 days'
    GROUP BY p.id, p.title, p.user_id, u.username, p.published_at
    HAVING COUNT(c.id) >= 5
)
SELECT 
    pp.*,
    RANK() OVER (ORDER BY pp.comment_count DESC) as popularity_rank
FROM popular_posts pp
ORDER BY pp.comment_count DESC, pp.published_at DESC;
```

This document tests various code block scenarios including:
- Multiple programming languages
- Inline code snippets
- Complex nested structures
- Long code blocks
- Mixed content with code and text
- Special characters and syntax highlighting