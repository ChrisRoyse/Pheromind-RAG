#[cfg(feature = "tree-sitter")]
#[test]
fn test_css_query_debug() {
    use tree_sitter::{Parser, Query};
    
    // Test the current CSS query that's causing the error
    let lang = tree_sitter_css::LANGUAGE.into();
    let mut parser = Parser::new();
    parser.set_language(&lang).unwrap();
    
    let problematic_query = r#"
        (rule_set (selectors) @selector)
        (class_selector (class_name) @class.selector)
        (id_selector (id_name) @id.selector)
        (media_statement) @media
        (keyframes_statement name: (keyframes_name) @keyframes.name)
    "#;
    
    println!("Testing problematic CSS query...");
    
    // This should reveal the exact error
    match Query::new(&lang, problematic_query) {
        Ok(_) => println!("✅ CSS query compiled successfully!"),
        Err(e) => {
            println!("❌ CSS query error: {}", e);
            
            // Test each part individually to isolate the problem
            let queries_to_test = vec![
                ("rule_set", "(rule_set (selectors) @selector)"),
                ("class_selector", "(class_selector (class_name) @class.selector)"),
                ("id_selector", "(id_selector (id_name) @id.selector)"),
                ("media_statement", "(media_statement) @media"),
                ("keyframes_statement", "(keyframes_statement name: (keyframes_name) @keyframes.name)"),
            ];
            
            for (name, query_str) in queries_to_test {
                match Query::new(&lang, query_str) {
                    Ok(_) => println!("✅ {} query works", name),
                    Err(e) => println!("❌ {} query failed: {}", name, e),
                }
            }
        }
    }
    
    // Test a simple CSS sample to verify the parser works
    let css_code = "@keyframes slideIn { from { opacity: 0; } to { opacity: 1; } }";
    if let Some(tree) = parser.parse(css_code, None) {
        println!("✅ CSS parsing works");
        println!("Tree: {}", tree.root_node().to_sexp());
    } else {
        println!("❌ CSS parsing failed");
    }
}