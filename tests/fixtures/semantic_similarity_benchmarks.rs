//! Semantic Similarity Benchmark Datasets
//!
//! This module provides curated test cases for validating semantic similarity
//! preservation in embeddings. These benchmarks expose broken embedding models
//! that fail to capture semantic relationships correctly.

use std::collections::HashMap;

/// Semantic similarity benchmark test case
#[derive(Debug, Clone)]
pub struct SimilarityBenchmark {
    pub text1: &'static str,
    pub text2: &'static str,
    pub expected_similarity_min: f32,
    pub expected_similarity_max: f32,
    pub category: &'static str,
    pub description: &'static str,
}

/// Comprehensive semantic similarity benchmark suite
pub fn get_semantic_similarity_benchmarks() -> Vec<SimilarityBenchmark> {
    vec![
        // High similarity - functionally equivalent code
        SimilarityBenchmark {
            text1: "def calculate_total(items): return sum(item.price for item in items)",
            text2: "function calculateTotal(items) { return items.reduce((sum, item) => sum + item.price, 0); }",
            expected_similarity_min: 0.75,
            expected_similarity_max: 0.95,
            category: "functional_equivalence",
            description: "Equivalent functions in different languages",
        },
        
        SimilarityBenchmark {
            text1: "SELECT user_id, name FROM users WHERE active = true ORDER BY created_at DESC",
            text2: "SELECT u.id, u.username FROM users u WHERE u.status = 'active' ORDER BY u.registration_date DESC",
            expected_similarity_min: 0.70,
            expected_similarity_max: 0.90,
            category: "sql_equivalence", 
            description: "Similar SQL queries with different syntax",
        },
        
        // Medium similarity - related concepts
        SimilarityBenchmark {
            text1: "class User { constructor(name, email) { this.name = name; this.email = email; } }",
            text2: "public class Customer { private String fullName; private String emailAddress; }",
            expected_similarity_min: 0.50,
            expected_similarity_max: 0.80,
            category: "related_concepts",
            description: "Similar data structures with different naming",
        },
        
        SimilarityBenchmark {
            text1: "import requests; response = requests.get('https://api.example.com/data')",
            text2: "fetch('https://api.example.com/users').then(response => response.json())",
            expected_similarity_min: 0.55,
            expected_similarity_max: 0.85,
            category: "related_concepts",
            description: "HTTP requests in different languages",
        },
        
        // Low similarity - different domains
        SimilarityBenchmark {
            text1: "def fibonacci(n): return n if n <= 1 else fibonacci(n-1) + fibonacci(n-2)",
            text2: "CREATE TABLE products (id SERIAL PRIMARY KEY, name VARCHAR(255), price DECIMAL(10,2))",
            expected_similarity_min: 0.05,
            expected_similarity_max: 0.45,
            category: "different_domains",
            description: "Algorithm vs database schema",
        },
        
        SimilarityBenchmark {
            text1: "body { margin: 0; padding: 0; font-family: Arial, sans-serif; background: #f0f0f0; }",
            text2: "const express = require('express'); const app = express(); app.listen(3000);",
            expected_similarity_min: 0.05,
            expected_similarity_max: 0.40,
            category: "different_domains",
            description: "CSS styling vs Node.js server",
        },
        
        // Authentication/Security domain - should cluster together
        SimilarityBenchmark {
            text1: "def authenticate_user(username, password): return bcrypt.checkpw(password, stored_hash)",
            text2: "function validateCredentials(email, pass) { return argon2.verify(hashedPass, pass); }",
            expected_similarity_min: 0.70,
            expected_similarity_max: 0.95,
            category: "domain_clustering_auth",
            description: "Authentication functions",
        },
        
        SimilarityBenchmark {
            text1: "JWT_SECRET = 'your-secret-key'; token = jwt.encode(payload, JWT_SECRET)",
            text2: "const secret = process.env.JWT_SECRET; const token = jsonwebtoken.sign(data, secret);",
            expected_similarity_min: 0.65,
            expected_similarity_max: 0.90,
            category: "domain_clustering_auth",
            description: "JWT token generation",
        },
        
        // Database operations - should cluster together
        SimilarityBenchmark {
            text1: "INSERT INTO users (name, email, created_at) VALUES (?, ?, NOW())",
            text2: "UPDATE users SET last_login = CURRENT_TIMESTAMP WHERE user_id = ?",
            expected_similarity_min: 0.60,
            expected_similarity_max: 0.85,
            category: "domain_clustering_db",
            description: "Database operations",
        },
        
        SimilarityBenchmark {
            text1: "db.users.find({ status: 'active' }).sort({ created_at: -1 }).limit(10)",
            text2: "SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 10",
            expected_similarity_min: 0.75,
            expected_similarity_max: 0.95,
            category: "domain_clustering_db",
            description: "Query operations in different database systems",
        },
        
        // Error handling patterns
        SimilarityBenchmark {
            text1: "try: result = risky_operation() except Exception as e: logger.error(f'Failed: {e}')",
            text2: "try { const result = await riskyOperation(); } catch (error) { console.error('Failed:', error); }",
            expected_similarity_min: 0.70,
            expected_similarity_max: 0.90,
            category: "patterns_error_handling",
            description: "Exception handling patterns",
        },
        
        // Configuration patterns
        SimilarityBenchmark {
            text1: "config = { 'host': 'localhost', 'port': 5432, 'database': 'myapp', 'timeout': 30 }",
            text2: "const settings = { server: '127.0.0.1', port: 5432, db: 'myapp', connectionTimeout: 30000 };",
            expected_similarity_min: 0.65,
            expected_similarity_max: 0.85,
            category: "patterns_configuration",
            description: "Configuration objects",
        },
        
        // Test edge cases - very short vs long
        SimilarityBenchmark {
            text1: "x = 1",
            text2: "let x = 1;",
            expected_similarity_min: 0.60,
            expected_similarity_max: 0.95,
            category: "edge_cases",
            description: "Simple assignment statements",
        },
        
        SimilarityBenchmark {
            text1: "x = 1",
            text2: "class ComplexDataProcessor { constructor() { this.handlers = new Map(); this.queue = []; } async processData(data) { return await this.handlers.get(data.type).process(data); } }",
            expected_similarity_min: 0.05,
            expected_similarity_max: 0.35,
            category: "edge_cases", 
            description: "Simple vs complex code",
        },
        
        // Comments vs code
        SimilarityBenchmark {
            text1: "# Calculate the fibonacci sequence recursively",
            text2: "def fibonacci(n): return n if n <= 1 else fibonacci(n-1) + fibonacci(n-2)",
            expected_similarity_min: 0.40,
            expected_similarity_max: 0.75,
            category: "comments_vs_code",
            description: "Comment describing implementation",
        },
        
        // Different programming paradigms
        SimilarityBenchmark {
            text1: "numbers.map(x => x * 2).filter(x => x > 10).reduce((a, b) => a + b, 0)",
            text2: "total = 0; for num in numbers: doubled = num * 2; if doubled > 10: total += doubled",
            expected_similarity_min: 0.60,
            expected_similarity_max: 0.85,
            category: "paradigms",
            description: "Functional vs imperative style",
        },
        
        // API documentation vs implementation
        SimilarityBenchmark {
            text1: "GET /api/users/{id} - Returns user information for the specified ID",
            text2: "@app.route('/api/users/<int:user_id>', methods=['GET']) def get_user(user_id): return jsonify(user_service.find_by_id(user_id))",
            expected_similarity_min: 0.65,
            expected_similarity_max: 0.85,
            category: "docs_vs_implementation",
            description: "API documentation vs implementation",
        },
    ]
}

/// Get test cases organized by category for focused testing
pub fn get_benchmarks_by_category() -> HashMap<&'static str, Vec<SimilarityBenchmark>> {
    let benchmarks = get_semantic_similarity_benchmarks();
    let mut by_category = HashMap::new();
    
    for benchmark in benchmarks {
        by_category.entry(benchmark.category)
            .or_insert_with(Vec::new)
            .push(benchmark);
    }
    
    by_category
}

/// Get pairs of test cases that should have high intra-category similarity
pub fn get_intra_category_similarity_tests() -> Vec<(&'static str, Vec<SimilarityBenchmark>)> {
    let mut tests = Vec::new();
    let by_category = get_benchmarks_by_category();
    
    for (category, benchmarks) in by_category {
        if benchmarks.len() >= 2 {
            tests.push((category, benchmarks));
        }
    }
    
    tests
}

/// Code complexity benchmark - tests handling of different complexity levels  
pub fn get_complexity_benchmarks() -> Vec<(&'static str, &'static str, f32)> {
    vec![
        // Simple
        ("simple", "x = 1", 0.1),
        ("simple", "print('hello')", 0.1),
        ("simple", "return True", 0.1),
        
        // Medium  
        ("medium", "def add(a, b): return a + b", 0.3),
        ("medium", "for i in range(10): print(i)", 0.3),
        ("medium", "if user.is_active(): user.login()", 0.3),
        
        // Complex
        ("complex", "class DatabaseManager: def __init__(self, config): self.pool = create_connection_pool(config)", 0.7),
        ("complex", "async def process_batch(items): return await asyncio.gather(*[process_item(item) for item in items])", 0.7),
        ("complex", "try: result = complex_operation() except (ValueError, TypeError) as e: logger.error(e); raise", 0.7),
        
        // Very Complex
        ("very_complex", 
         "class DistributedCacheManager: def __init__(self, nodes): self.ring = ConsistentHashRing(nodes); self.local_cache = LRUCache(1000) def get(self, key): node = self.ring.get_node(key); return self.fetch_with_fallback(node, key)",
         0.9),
    ]
}

/// Performance benchmark texts for testing embedding speed vs accuracy tradeoffs
pub fn get_performance_benchmarks() -> Vec<(&'static str, usize)> {
    vec![
        ("short", 10),      // ~10 tokens
        ("medium", 50),     // ~50 tokens  
        ("long", 200),      // ~200 tokens
        ("very_long", 500), // ~500 tokens
    ].into_iter().map(|(category, target_tokens)| {
        let text = match category {
            "short" => "def hello(): return 'world'",
            "medium" => "class UserManager { constructor(db) { this.db = db; this.cache = new Map(); } async getUser(id) { if (this.cache.has(id)) return this.cache.get(id); const user = await this.db.query('SELECT * FROM users WHERE id = ?', [id]); this.cache.set(id, user); return user; } }",
            "long" => {
                let base = "function processData(input) { const results = []; for (let i = 0; i < input.length; i++) { const item = input[i]; if (item.type === 'user') { results.push(processUser(item)); } else if (item.type === 'order') { results.push(processOrder(item)); } else { results.push(processGeneric(item)); } } return results; }";
                base
            },
            "very_long" => {
                "const handler = async (req, res) => { try { const data = await validateInput(req.body); const result = await service.process(data); res.json({ success: true, data: result }); } catch (error) { res.status(500).json({ success: false, error: error.message }); } }; const handler2 = async (req, res) => { try { const data = await validateInput(req.body); const result = await service.process(data); res.json({ success: true, data: result }); } catch (error) { res.status(500).json({ success: false, error: error.message }); } }; const handler3 = async (req, res) => { try { const data = await validateInput(req.body); const result = await service.process(data); res.json({ success: true, data: result }); } catch (error) { res.status(500).json({ success: false, error: error.message }); } };"
            },
            _ => "default",
        };
        (text, target_tokens)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_data_structure() {
        let benchmarks = get_semantic_similarity_benchmarks();
        
        // Verify we have a good variety of test cases
        assert!(benchmarks.len() >= 15, "Should have at least 15 benchmark cases");
        
        // Verify similarity bounds are reasonable
        for benchmark in &benchmarks {
            assert!(benchmark.expected_similarity_min >= 0.0, 
                "Min similarity must be >= 0.0 for {}", benchmark.description);
            assert!(benchmark.expected_similarity_max <= 1.0, 
                "Max similarity must be <= 1.0 for {}", benchmark.description);
            assert!(benchmark.expected_similarity_min < benchmark.expected_similarity_max, 
                "Min must be < Max for {}", benchmark.description);
        }
        
        // Verify we have different categories
        let categories: std::collections::HashSet<_> = benchmarks.iter().map(|b| b.category).collect();
        assert!(categories.len() >= 5, "Should have at least 5 different categories");
        
        println!("✓ Benchmark data structure validation passed");
        println!("  - {} total benchmarks", benchmarks.len());
        println!("  - {} categories: {:?}", categories.len(), categories);
    }
    
    #[test]
    fn test_category_organization() {
        let by_category = get_benchmarks_by_category();
        
        // Verify intra-category tests
        let intra_tests = get_intra_category_similarity_tests();
        assert!(intra_tests.len() >= 3, "Should have multiple intra-category test sets");
        
        for (category, benchmarks) in intra_tests {
            println!("Category '{}': {} benchmarks", category, benchmarks.len());
        }
        
        println!("✓ Category organization validation passed");
    }
}