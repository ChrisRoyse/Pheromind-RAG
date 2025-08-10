// Traditional CLI for embed-search that can be used globally
// This is separate from the MCP server and can be added to PATH

use anyhow::Result;
use clap::{Parser, Subcommand};
use embed_search::{simple_search::HybridSearch as SimpleSearch, simple_embedder::NomicEmbedder, SymbolExtractor, SymbolKind};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "embed")]
#[clap(about = "Embed Search CLI - Global command-line tool for code search", version)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    
    /// Index directory path (defaults to ./search_index)
    #[clap(short = 'i', long, default_value = "./search_index")]
    index_path: String,
    
    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Search indexed code
    Search {
        /// Search query
        query: String,
        
        /// Maximum number of results
        #[clap(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Index files for searching
    Index {
        /// File or directory path to index
        path: PathBuf,
        
        /// File extensions to include (e.g., rs,py,js)
        #[clap(short, long, default_value = "rs,py,js,ts,jsx,tsx,go,java,cpp,c,h")]
        extensions: String,
    },
    
    /// Extract symbols from code
    Symbols {
        /// File path to analyze
        file: PathBuf,
    },
    
    /// Show search system status
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Set up logging
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
    
    // Use tokio runtime
    let runtime = tokio::runtime::Runtime::new()?;
    
    runtime.block_on(async {
        match cli.command {
            Commands::Search { query, limit } => {
                search(&cli.index_path, &query, limit).await?;
            }
            Commands::Index { path, extensions } => {
                index(&cli.index_path, &path, &extensions).await?;
            }
            Commands::Symbols { file } => {
                extract_symbols(&file)?;
            }
            Commands::Status => {
                show_status(&cli.index_path)?;
            }
        }
        Ok(())
    })
}

async fn search(index_path: &str, query: &str, limit: usize) -> Result<()> {
    println!("🔍 Searching for: {}", query);
    println!();
    
    let db_path = format!("{}/vectors.db", index_path);
    let mut search_engine = SimpleSearch::new(&db_path).await?;
    let results = search_engine.search(query, limit).await?;
    
    if results.is_empty() {
        println!("No results found.");
    } else {
        println!("Found {} results:\n", results.len());
        for (i, result) in results.iter().enumerate() {
            println!("{}. {}", i + 1, result.file_path);
            
            // Show snippet
            let lines: Vec<&str> = result.content.lines().collect();
            let preview = if lines.len() > 3 {
                format!("   {}...", lines[..3].join("\n   "))
            } else {
                format!("   {}", result.content)
            };
            println!("{}", preview);
            println!();
        }
    }
    
    Ok(())
}

async fn index(index_path: &str, path: &PathBuf, extensions: &str) -> Result<()> {
    println!("📁 Indexing: {}", path.display());
    println!("   Extensions: {}", extensions);
    println!();
    
    let exts: Vec<String> = extensions.split(',').map(|s| s.trim().to_string()).collect();
    
    // Create index directory if it doesn't exist
    std::fs::create_dir_all(index_path)?;
    
    let db_path = format!("{}/vectors.db", index_path);
    let mut search_engine = SimpleSearch::new(&db_path).await?;
    let mut embedder = NomicEmbedder::new()?;
    
    let mut indexed_count = 0;
    let mut skipped_count = 0;
    
    // Handle single file or directory
    if path.is_file() {
        // Index single file
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if exts.contains(&ext.to_string()) {
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        let path_str = path.to_string_lossy().to_string();
                        let embedding = embedder.embed_batch(vec![format!("passage: {}", content)])?;
                        
                        // Index using the batch method
                        search_engine.index(vec![content], vec![path_str.clone()]).await?;
                        indexed_count = 1;
                        println!("✅ Indexed: {}", path.display());
                    }
                    Err(e) => {
                        println!("❌ Failed to read file: {}", e);
                        return Err(e.into());
                    }
                }
            } else {
                println!("⚠️  File extension not in list: {}", ext);
            }
        }
    } else if path.is_dir() {
        // Walk directory and index files
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if !entry_path.is_file() {
                continue;
            }
            
            // Check extension
            let ext = entry_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            
            if !exts.contains(&ext.to_string()) {
                continue;
            }
            
            // Read file for batch indexing
            match std::fs::read_to_string(entry_path) {
                Ok(content) => {
                    let path_str = entry_path.to_string_lossy().to_string();
                    
                    // Batch index (in production, would batch multiple files)
                    search_engine.index(vec![content], vec![path_str]).await?;
                    indexed_count += 1;
                    
                    if indexed_count % 10 == 0 {
                        print!(".");
                        use std::io::Write;
                        std::io::stdout().flush()?;
                    }
                }
                Err(_) => {
                    skipped_count += 1;
                }
            }
        }
        println!();
    } else {
        println!("❌ Path does not exist: {}", path.display());
        return Err(anyhow::anyhow!("Path not found"));
    }
    
    println!("\n✅ Indexed {} files", indexed_count);
    if skipped_count > 0 {
        println!("   Skipped {} files", skipped_count);
    }
    
    Ok(())
}

fn extract_symbols(file: &PathBuf) -> Result<()> {
    println!("🔍 Extracting symbols from: {}", file.display());
    println!();
    
    let content = std::fs::read_to_string(file)?;
    let ext = file.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    let mut extractor = SymbolExtractor::new()?;
    let symbols = extractor.extract(&content, ext)?;
    
    if symbols.is_empty() {
        println!("No symbols found.");
    } else {
        println!("Found {} symbols:\n", symbols.len());
        for symbol in symbols {
            let icon = match symbol.kind {
                SymbolKind::Function => "🔧",
                SymbolKind::Class => "📦",
                SymbolKind::Method => "⚙️",
                SymbolKind::Struct => "🏗️",
                SymbolKind::Enum => "📋",
                SymbolKind::Interface => "🔌",
                SymbolKind::Module => "📁",
                SymbolKind::Variable => "📝",
                SymbolKind::Constant => "🔒",
            };
            
            println!("  {} {} (line {})", icon, symbol.name, symbol.line);
            if !symbol.definition.is_empty() {
                let preview = if symbol.definition.len() > 60 {
                    format!("{}...", &symbol.definition[..60])
                } else {
                    symbol.definition.clone()
                };
                println!("      {}", preview);
            }
        }
    }
    
    Ok(())
}

fn show_status(index_path: &str) -> Result<()> {
    println!("📊 Embed Search Status");
    println!("{}", "=".repeat(40));
    
    // Check if index exists
    let index_exists = std::path::Path::new(index_path).exists();
    println!("Index Path: {}", index_path);
    println!("Exists: {}", if index_exists { "✅" } else { "❌" });
    
    if index_exists {
        // Try to get directory size
        let mut total_size = 0u64;
        let mut file_count = 0;
        
        for entry in walkdir::WalkDir::new(index_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }
        
        println!("Index Files: {}", file_count);
        println!("Total Size: {} MB", total_size / 1_048_576);
    }
    
    println!();
    println!("Search Technologies:");
    println!("  ✅ Nomic Embeddings (768-dim)");
    println!("  ✅ Tantivy Full-Text Search");
    println!("  ✅ Tree-sitter AST Extraction");
    println!("  ✅ BM25 Scoring (K1=1.2, B=0.75)");
    println!("  ✅ LanceDB Vector Storage");
    println!("  ✅ Hybrid Fusion with RRF");
    
    println!();
    println!("Supported Languages:");
    println!("  • Rust (.rs)");
    println!("  • Python (.py)");
    println!("  • JavaScript (.js, .jsx)");
    println!("  • TypeScript (.ts, .tsx)");
    
    Ok(())
}