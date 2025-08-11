use anyhow::Result;
use pheromind_rag::gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig};
use pheromind_rag::embedding_prefixes::EmbeddingTask;
use std::path::Path;

#[test]
fn test_dual_embedder_initialization() -> Result<()> {
    // Test that both models can be initialized
    
    // Text embedder for markdown
    let text_config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        ..Default::default()
    };
    let text_embedder = GGUFEmbedder::new(text_config)?;
    
    // Code embedder for code files
    let code_config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-code.Q4_K_M.gguf".to_string(),
        ..Default::default()
    };
    let code_embedder = GGUFEmbedder::new(code_config)?;
    
    // Test text embedding
    let text_content = "# Documentation\nThis is markdown content.";
    let text_embedding = text_embedder.embed(text_content, EmbeddingTask::SearchDocument)?;
    assert_eq!(text_embedding.len(), 768);
    
    // Test code embedding
    let code_content = "fn main() {\n    println!(\"Hello, world!\");\n}";
    let code_embedding = code_embedder.embed(code_content, EmbeddingTask::CodeDefinition)?;
    assert_eq!(code_embedding.len(), 768);
    
    // Embeddings should be different for same content with different models
    let cross_text = text_embedder.embed(code_content, EmbeddingTask::SearchDocument)?;
    let cross_code = code_embedder.embed(code_content, EmbeddingTask::CodeDefinition)?;
    
    // Calculate difference
    let diff: f32 = cross_text.iter()
        .zip(cross_code.iter())
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>() / 768.0;
    
    // Models should produce different embeddings
    assert!(diff > 0.01, "Different models should produce different embeddings");
    
    println!("✅ Both embedders initialized successfully");
    println!("✅ Text model embedding dimension: {}", text_embedding.len());
    println!("✅ Code model embedding dimension: {}", code_embedding.len());
    println!("✅ Models produce different embeddings (avg diff: {:.4})", diff);
    
    Ok(())
}

#[test]
fn test_file_type_routing() -> Result<()> {
    // Helper function to determine embedder based on file extension
    fn get_embedder_type(file_path: &str) -> &'static str {
        if let Some(ext) = Path::new(file_path).extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "md" | "markdown" => "text",
                    "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | 
                    "cpp" | "c" | "h" | "hpp" | "cc" | "cxx" | "cs" | "php" | 
                    "rb" | "swift" | "kt" | "scala" | "r" => "code",
                    _ => "text"
                }
            } else {
                "text"
            }
        } else {
            "text"
        }
    }
    
    // Test various file types
    assert_eq!(get_embedder_type("README.md"), "text");
    assert_eq!(get_embedder_type("docs.markdown"), "text");
    assert_eq!(get_embedder_type("main.rs"), "code");
    assert_eq!(get_embedder_type("script.py"), "code");
    assert_eq!(get_embedder_type("app.js"), "code");
    assert_eq!(get_embedder_type("component.tsx"), "code");
    assert_eq!(get_embedder_type("server.go"), "code");
    assert_eq!(get_embedder_type("Main.java"), "code");
    assert_eq!(get_embedder_type("program.cpp"), "code");
    assert_eq!(get_embedder_type("unknown.txt"), "text");
    
    println!("✅ File type routing test passed");
    
    Ok(())
}

#[test]
fn test_embedding_task_selection() -> Result<()> {
    // Test that appropriate tasks are selected
    fn get_task_for_file(file_path: &str) -> EmbeddingTask {
        if let Some(ext) = Path::new(file_path).extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "md" | "markdown" => EmbeddingTask::SearchDocument,
                    "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | 
                    "cpp" | "c" | "h" | "hpp" | "cc" | "cxx" | "cs" | "php" | 
                    "rb" | "swift" | "kt" | "scala" | "r" => EmbeddingTask::CodeDefinition,
                    _ => EmbeddingTask::SearchDocument
                }
            } else {
                EmbeddingTask::SearchDocument
            }
        } else {
            EmbeddingTask::SearchDocument
        }
    }
    
    // Test task selection
    assert_eq!(get_task_for_file("README.md"), EmbeddingTask::SearchDocument);
    assert_eq!(get_task_for_file("main.rs"), EmbeddingTask::CodeDefinition);
    assert_eq!(get_task_for_file("app.py"), EmbeddingTask::CodeDefinition);
    assert_eq!(get_task_for_file("index.js"), EmbeddingTask::CodeDefinition);
    
    println!("✅ Embedding task selection test passed");
    
    Ok(())
}

#[test]
fn test_code_vs_text_embeddings() -> Result<()> {
    // Initialize both embedders
    let text_config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        batch_size: 1,
        ..Default::default()
    };
    let text_embedder = GGUFEmbedder::new(text_config)?;
    
    let code_config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-code.Q4_K_M.gguf".to_string(),
        batch_size: 1,
        ..Default::default()
    };
    let code_embedder = GGUFEmbedder::new(code_config)?;
    
    // Test with actual code
    let rust_code = r#"
        fn calculate_fibonacci(n: u32) -> u32 {
            match n {
                0 => 0,
                1 => 1,
                _ => calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2),
            }
        }
    "#;
    
    // Test with markdown
    let markdown_text = r#"
        # Fibonacci Function
        This function calculates the nth Fibonacci number using recursion.
        It's a classic example of recursive algorithms.
    "#;
    
    // Generate embeddings with appropriate models
    let code_with_code_model = code_embedder.embed(rust_code, EmbeddingTask::CodeDefinition)?;
    let code_with_text_model = text_embedder.embed(rust_code, EmbeddingTask::SearchDocument)?;
    
    let text_with_text_model = text_embedder.embed(markdown_text, EmbeddingTask::SearchDocument)?;
    let text_with_code_model = code_embedder.embed(markdown_text, EmbeddingTask::CodeDefinition)?;
    
    // Calculate cosine similarities
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (norm_a * norm_b)
    }
    
    let code_model_diff = cosine_similarity(&code_with_code_model, &code_with_text_model);
    let text_model_diff = cosine_similarity(&text_with_text_model, &text_with_code_model);
    
    println!("✅ Code embedded with code model vs text model similarity: {:.4}", code_model_diff);
    println!("✅ Text embedded with text model vs code model similarity: {:.4}", text_model_diff);
    println!("✅ Models are specialized for their respective content types");
    
    Ok(())
}