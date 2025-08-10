use anyhow::Result;
use clap::{Parser, Subcommand};
use walkdir::WalkDir;
use std::fs;
// std::path::Path temporarily removed

mod simple_embedder;
mod simple_storage;
mod simple_search;

use simple_search::HybridSearch;

#[derive(Parser)]
#[command(name = "embed-search")]
#[command(about = "Simplified embedding search using real tech stack")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Index files in a directory
    Index {
        /// Directory to index
        path: String,
    },
    /// Search for content
    Search {
        /// Search query
        query: String,
    },
    /// Clear all indexed data
    Clear,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = "./simple_embed.db";

    match cli.command {
        Commands::Index { path } => {
            println!("Indexing files in: {}", path);
            let mut search = HybridSearch::new(db_path).await?;
            
            let mut contents = Vec::new();
            let mut file_paths = Vec::new();
            
            // Walk directory and collect files
            for entry in WalkDir::new(&path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| {
                    if let Some(ext) = e.path().extension() {
                        matches!(ext.to_str(), Some("rs") | Some("py") | Some("js") | Some("ts"))
                    } else {
                        false
                    }
                }) {
                
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.len() < 10000 { // Skip very large files
                        contents.push(content);
                        file_paths.push(entry.path().display().to_string());
                    }
                }
                
                // Process in batches
                if contents.len() >= 10 {
                    println!("Indexing batch of {} files", contents.len());
                    search.index(contents.clone(), file_paths.clone()).await?;
                    contents.clear();
                    file_paths.clear();
                }
            }
            
            // Process remaining files
            if !contents.is_empty() {
                println!("Indexing final batch of {} files", contents.len());
                search.index(contents, file_paths).await?;
            }
            
            println!("Indexing complete!");
        },
        
        Commands::Search { query } => {
            println!("Searching for: {}", query);
            let mut search = HybridSearch::new(db_path).await?;
            
            let results = search.search(&query, 10).await?;
            
            if results.is_empty() {
                println!("No results found");
            } else {
                println!("Found {} results:", results.len());
                for (i, result) in results.iter().enumerate() {
                    println!("\n{}. {} ({})", i + 1, result.file_path, result.match_type);
                    println!("   Score: {:.3}", result.score);
                    let preview = if result.content.len() > 100 {
                        format!("{}...", &result.content[..100])
                    } else {
                        result.content.clone()
                    };
                    println!("   {}", preview.replace('\n', " "));
                }
            }
        },
        
        Commands::Clear => {
            println!("Clearing all indexed data");
            let mut search = HybridSearch::new(db_path).await?;
            search.clear().await?;
            println!("Data cleared!");
        },
    }

    Ok(())
}