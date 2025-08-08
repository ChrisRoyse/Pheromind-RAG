use embed_search::search::bm25_fixed::{BM25Engine, BM25Match};

#[test]
fn debug_universal_term_idf() {
    let mut engine = BM25Engine::new().expect("Failed to create BM25Engine");
    
    // Add just a few documents to debug
    engine.index_document("doc1", "universal_term content1");
    engine.index_document("doc2", "universal_term content2");
    engine.index_document("doc3", "universal_term content3");
    
    println!("Testing IDF calculation for universal term...");
    
    let universal_idf = engine.calculate_idf("universal"); // Search for "universal" not "universal_term"
    let term_idf = engine.calculate_idf("term"); // This should also appear in all 3
    let nonexistent_idf = engine.calculate_idf("nonexistent_term");
    let partial_idf = engine.calculate_idf("content1"); // appears in 1 doc
    
    println!("Universal term IDF (appears in all 3): {}", universal_idf);
    println!("Term IDF (appears in all 3): {}", term_idf);
    println!("Nonexistent term IDF: {}", nonexistent_idf);
    println!("Partial term IDF (appears in 1): {}", partial_idf);
    
    // Test the BM25 formula manually
    let n = 3.0f32; // total docs
    let df = 3.0f32; // docs containing universal_term
    let ratio: f32 = (n - df + 0.5) / (df + 0.5);
    println!("Manual ratio calculation: ({} - {} + 0.5) / ({} + 0.5) = {}", n, df, df, ratio);
    println!("ln({}) = {}", ratio, ratio.ln());
    
    // This should be epsilon-protected
    const EPSILON: f32 = 0.01;
    let expected = if ratio <= 0.0 { EPSILON } else { ratio.ln().max(EPSILON) };
    println!("Expected IDF with epsilon protection: {}", expected);
}