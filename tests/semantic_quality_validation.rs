//! Semantic Quality Validation Suite
//! 
//! This module validates that the repaired embedding system correctly captures
//! semantic meaning and provides high-quality search results for real-world queries.

use anyhow::Result;
use std::collections::HashMap;

#[cfg(feature = "ml")]
use embed_search::embedding::NomicEmbedder;

/// Semantic test cases with expected similarity relationships
pub struct SemanticTestSuite {
    /// Groups of semantically similar texts that should cluster together
    pub similar_groups: Vec<Vec<String>>,
    /// Pairs that should be dissimilar despite superficial similarities
    pub dissimilar_pairs: Vec<(String, String)>,
    /// Code refactoring pairs - same functionality, different implementation
    pub refactoring_pairs: Vec<(String, String)>,
    /// Cross-language equivalents
    pub cross_language_pairs: Vec<(String, String)>,
}

impl SemanticTestSuite {
    pub fn new() -> Self {
        Self {
            similar_groups: vec![
                // Group 1: Error handling patterns
                vec![
                    "try { result = await api.call(); } catch (error) { console.error(error); }".to_string(),
                    "try: result = api_call() except Exception as e: print(f'Error: {e}')".to_string(),
                    "match api_call().await { Ok(result) => result, Err(e) => { eprintln!(\"Error: {}\", e); return; } }".to_string(),
                ],
                
                // Group 2: Database operations  
                vec![
                    "SELECT * FROM users WHERE age > 18 AND status = 'active'".to_string(),
                    "User.where('age > ? AND status = ?', 18, 'active')".to_string(),
                    "users.filter(user => user.age > 18 && user.status === 'active')".to_string(),
                    "users.iter().filter(|u| u.age > 18 && u.status == \"active\").collect()".to_string(),
                ],
                
                // Group 3: Authentication checks
                vec![
                    "if (user.isAuthenticated()) { return renderDashboard(); }".to_string(),
                    "if user.authenticated: return dashboard_view()".to_string(), 
                    "if user.is_authenticated() { Ok(dashboard()) } else { Err(\"Unauthorized\") }".to_string(),
                ],
                
                // Group 4: List/Array processing
                vec![
                    "const total = items.reduce((sum, item) => sum + item.price, 0)".to_string(),
                    "total = sum(item.price for item in items)".to_string(),
                    "let total: f32 = items.iter().map(|item| item.price).sum()".to_string(),
                    "items.map(item => item.price).reduce((a, b) => a + b, 0)".to_string(),
                ],
                
                // Group 5: HTTP request patterns
                vec![
                    "const response = await fetch('/api/users', { method: 'GET' })".to_string(),
                    "response = requests.get('/api/users')".to_string(),
                    "let response = client.get(\"/api/users\").await?".to_string(),
                    "$.ajax({ url: '/api/users', type: 'GET' })".to_string(),
                ],
            ],
            
            dissimilar_pairs: vec![
                // Different domains despite similar syntax
                (
                    "function calculateTotal(items) { return items.reduce((sum, item) => sum + item.price, 0); }".to_string(),
                    "function animate(element) { element.style.opacity = 0; element.classList.add('fade-out'); }".to_string()
                ),
                (
                    "SELECT * FROM users WHERE active = true".to_string(),
                    "const colors = ['red', 'green', 'blue', 'yellow']".to_string()
                ),
                (
                    "class UserManager { constructor() { this.users = []; } }".to_string(),
                    "body { font-family: Arial; background-color: #f0f0f0; }".to_string()
                ),
                (
                    "async function fetchData() { return await api.get('/data'); }".to_string(),
                    "const PI = 3.14159; const area = PI * radius * radius;".to_string()
                ),
            ],
            
            refactoring_pairs: vec![
                // Same logic, different structure
                (
                    "function validateEmail(email) { return /^[^@]+@[^@]+\\.[^@]+$/.test(email); }".to_string(),
                    "const isValidEmail = (email) => { const regex = /^[^@]+@[^@]+\\.[^@]+$/; return regex.test(email); }".to_string()
                ),
                (
                    "for (let i = 0; i < array.length; i++) { console.log(array[i]); }".to_string(),
                    "array.forEach(item => console.log(item))".to_string()
                ),
                (
                    "if (condition) { doSomething(); } else { doSomethingElse(); }".to_string(),
                    "condition ? doSomething() : doSomethingElse()".to_string()
                ),
                (
                    "function multiply(a, b) { return a * b; }".to_string(),
                    "const multiply = (a, b) => a * b".to_string()
                ),
            ],
            
            cross_language_pairs: vec![
                // Equivalent functionality across languages
                (
                    "function fibonacci(n) { if (n <= 1) return n; return fibonacci(n-1) + fibonacci(n-2); }".to_string(),
                    "def fibonacci(n): return n if n <= 1 else fibonacci(n-1) + fibonacci(n-2)".to_string()
                ),
                (
                    "class Rectangle { constructor(w, h) { this.width = w; this.height = h; } }".to_string(),
                    "struct Rectangle { width: f64, height: f64 }".to_string()
                ),
                (
                    "const sorted = array.sort((a, b) => a - b)".to_string(),
                    "sorted_array = sorted(array)".to_string()
                ),
                (
                    "try { const data = JSON.parse(jsonString); } catch (e) { console.error(e); }".to_string(),
                    "try { let data: Value = serde_json::from_str(&json_string)?; } catch { eprintln!(\"Parse error\"); }".to_string()
                ),
            ],
        }
    }
}

#[tokio::test]
#[cfg(feature = "ml")]
async fn test_semantic_similarity_groups() -> Result<()> {
    println!("🧠 Testing Semantic Similarity Groups");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(_) => {
            println!("⚠️ Skipping semantic tests - model not available");
            return Ok(());
        }
    };
    
    let test_suite = SemanticTestSuite::new();
    
    for (group_idx, group) in test_suite.similar_groups.iter().enumerate() {
        println!("📊 Testing similarity group {} ({} items)", group_idx + 1, group.len());
        
        // Generate embeddings for all items in the group
        let mut embeddings = Vec::new();
        for text in group {
            let embedding = embedder.embed(text)?;
            embeddings.push(embedding);
        }
        
        // Calculate pairwise similarities within the group
        let mut similarities = Vec::new();
        for i in 0..embeddings.len() {
            for j in i+1..embeddings.len() {
                let similarity = cosine_similarity(&embeddings[i], &embeddings[j]);
                similarities.push(similarity);
            }
        }
        
        let avg_similarity = similarities.iter().sum::<f32>() / similarities.len() as f32;
        let min_similarity = similarities.iter().cloned().fold(f32::INFINITY, f32::min);
        
        println!("  📈 Group {} - Avg similarity: {:.4}, Min similarity: {:.4}", 
                 group_idx + 1, avg_similarity, min_similarity);
        
        // Assertions for semantic similarity
        assert!(avg_similarity > 0.3, 
                "Group {} average similarity should be > 0.3, got {:.4}", 
                group_idx + 1, avg_similarity);
        assert!(min_similarity > 0.15, 
                "Group {} minimum similarity should be > 0.15, got {:.4}", 
                group_idx + 1, min_similarity);
    }
    
    println!("✅ Semantic similarity group tests PASSED");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "ml")]
async fn test_dissimilar_pairs() -> Result<()> {
    println!("🔍 Testing Dissimilar Pairs Detection");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(_) => {
            println!("⚠️ Skipping dissimilarity tests - model not available");
            return Ok(());
        }
    };
    
    let test_suite = SemanticTestSuite::new();
    
    for (idx, (text1, text2)) in test_suite.dissimilar_pairs.iter().enumerate() {
        let embedding1 = embedder.embed(text1)?;
        let embedding2 = embedder.embed(text2)?;
        
        let similarity = cosine_similarity(&embedding1, &embedding2);
        
        println!("🔄 Dissimilar pair {} similarity: {:.4}", idx + 1, similarity);
        
        // Assertion for dissimilarity
        assert!(similarity < 0.5, 
                "Dissimilar pair {} should have similarity < 0.5, got {:.4}", 
                idx + 1, similarity);
    }
    
    println!("✅ Dissimilar pairs detection tests PASSED");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "ml")]
async fn test_refactoring_equivalence() -> Result<()> {
    println!("♻️ Testing Refactoring Equivalence Detection");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(_) => {
            println!("⚠️ Skipping refactoring tests - model not available");
            return Ok(());
        }
    };
    
    let test_suite = SemanticTestSuite::new();
    
    for (idx, (original, refactored)) in test_suite.refactoring_pairs.iter().enumerate() {
        let embedding1 = embedder.embed(original)?;
        let embedding2 = embedder.embed(refactored)?;
        
        let similarity = cosine_similarity(&embedding1, &embedding2);
        
        println!("🔄 Refactoring pair {} similarity: {:.4}", idx + 1, similarity);
        
        // Refactored code should be highly similar to original
        assert!(similarity > 0.6, 
                "Refactoring pair {} should have similarity > 0.6, got {:.4}", 
                idx + 1, similarity);
    }
    
    println!("✅ Refactoring equivalence tests PASSED");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "ml")]
async fn test_cross_language_equivalence() -> Result<()> {
    println!("🌐 Testing Cross-Language Equivalence Detection");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(_) => {
            println!("⚠️ Skipping cross-language tests - model not available");
            return Ok(());
        }
    };
    
    let test_suite = SemanticTestSuite::new();
    
    for (idx, (lang1_code, lang2_code)) in test_suite.cross_language_pairs.iter().enumerate() {
        let embedding1 = embedder.embed(lang1_code)?;
        let embedding2 = embedder.embed(lang2_code)?;
        
        let similarity = cosine_similarity(&embedding1, &embedding2);
        
        println!("🔄 Cross-language pair {} similarity: {:.4}", idx + 1, similarity);
        
        // Cross-language equivalent code should be moderately similar
        assert!(similarity > 0.4, 
                "Cross-language pair {} should have similarity > 0.4, got {:.4}", 
                idx + 1, similarity);
    }
    
    println!("✅ Cross-language equivalence tests PASSED");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "ml")]
async fn test_code_vs_natural_language() -> Result<()> {
    println!("💬 Testing Code vs Natural Language Distinction");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(_) => {
            println!("⚠️ Skipping code vs language tests - model not available");
            return Ok(());
        }
    };
    
    let code_samples = vec![
        "function calculateTotal(items) { return items.reduce((sum, item) => sum + item.price, 0); }",
        "class UserManager { constructor() { this.users = []; } addUser(user) { this.users.push(user); } }",
        "const data = await fetch('/api/users').then(response => response.json());",
        "for (let i = 0; i < array.length; i++) { console.log(array[i]); }",
    ];
    
    let natural_language = vec![
        "The quick brown fox jumps over the lazy dog.",
        "Today is a beautiful sunny day with clear blue skies.",
        "Please remember to submit your report by the deadline.",
        "The meeting has been scheduled for tomorrow at 3 PM.",
    ];
    
    // Calculate similarities within each group
    let mut code_similarities = Vec::new();
    for i in 0..code_samples.len() {
        for j in i+1..code_samples.len() {
            let emb1 = embedder.embed(code_samples[i])?;
            let emb2 = embedder.embed(code_samples[j])?;
            code_similarities.push(cosine_similarity(&emb1, &emb2));
        }
    }
    
    let mut lang_similarities = Vec::new();
    for i in 0..natural_language.len() {
        for j in i+1..natural_language.len() {
            let emb1 = embedder.embed(natural_language[i])?;
            let emb2 = embedder.embed(natural_language[j])?;
            lang_similarities.push(cosine_similarity(&emb1, &emb2));
        }
    }
    
    // Calculate cross-category similarities
    let mut cross_similarities = Vec::new();
    for code in &code_samples {
        for lang in &natural_language {
            let emb1 = embedder.embed(code)?;
            let emb2 = embedder.embed(lang)?;
            cross_similarities.push(cosine_similarity(&emb1, &emb2));
        }
    }
    
    let avg_code_similarity = code_similarities.iter().sum::<f32>() / code_similarities.len() as f32;
    let avg_lang_similarity = lang_similarities.iter().sum::<f32>() / lang_similarities.len() as f32;
    let avg_cross_similarity = cross_similarities.iter().sum::<f32>() / cross_similarities.len() as f32;
    
    println!("📊 Average code-to-code similarity: {:.4}", avg_code_similarity);
    println!("📊 Average language-to-language similarity: {:.4}", avg_lang_similarity);
    println!("📊 Average cross-category similarity: {:.4}", avg_cross_similarity);
    
    // Code should be more similar to other code than to natural language
    assert!(avg_code_similarity > avg_cross_similarity, 
            "Code should be more similar to other code than to natural language");
    
    // Natural language should be more similar to other natural language than to code
    assert!(avg_lang_similarity > avg_cross_similarity,
            "Natural language should be more similar to other natural language than to code");
    
    println!("✅ Code vs natural language distinction tests PASSED");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "ml")]
async fn test_programming_concept_clustering() -> Result<()> {
    println!("🏗️ Testing Programming Concept Clustering");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(_) => {
            println!("⚠️ Skipping concept clustering tests - model not available");
            return Ok(());
        }
    };
    
    let concept_groups = HashMap::from([
        ("loops", vec![
            "for (let i = 0; i < 10; i++) { console.log(i); }",
            "while (condition) { doSomething(); }",
            "array.forEach(item => console.log(item))",
            "for item in items: print(item)",
        ]),
        ("conditionals", vec![
            "if (condition) { doSomething(); }",
            "condition ? trueValue : falseValue",
            "switch (value) { case 1: return 'one'; default: return 'other'; }",
            "if condition: do_something() else: do_other_thing()",
        ]),
        ("functions", vec![
            "function add(a, b) { return a + b; }",
            "const multiply = (x, y) => x * y",
            "def calculate(value): return value * 2",
            "fn compute(input: i32) -> i32 { input + 1 }",
        ]),
        ("classes", vec![
            "class User { constructor(name) { this.name = name; } }",
            "struct Point { x: f64, y: f64 }",
            "class Animal: def __init__(self, name): self.name = name",
            "interface UserInterface { name: string; age: number; }",
        ]),
    ]);
    
    for (concept, examples) in &concept_groups {
        println!("🏷️ Testing concept: {}", concept);
        
        let mut embeddings = Vec::new();
        for example in examples {
            let embedding = embedder.embed(example)?;
            embeddings.push(embedding);
        }
        
        // Calculate intra-concept similarities
        let mut similarities = Vec::new();
        for i in 0..embeddings.len() {
            for j in i+1..embeddings.len() {
                let sim = cosine_similarity(&embeddings[i], &embeddings[j]);
                similarities.push(sim);
            }
        }
        
        let avg_similarity = similarities.iter().sum::<f32>() / similarities.len() as f32;
        println!("  📈 {} average similarity: {:.4}", concept, avg_similarity);
        
        // Programming concepts should cluster together
        assert!(avg_similarity > 0.25, 
                "{} concepts should have similarity > 0.25, got {:.4}", 
                concept, avg_similarity);
    }
    
    println!("✅ Programming concept clustering tests PASSED");
    Ok(())
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}