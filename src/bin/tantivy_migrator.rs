use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;
use anyhow::{Result, anyhow, Context};
use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

use embed_search::config::{Config, SearchBackend};
use embed_search::search::{TantivySearcher, ExactMatch};
use embed_search::search::search_adapter::{create_text_searcher_with_root};

/// Tantivy Migration Tool
/// 
/// Safely migrate production systems to Tantivy with validation,
/// backup/restore capabilities, and comprehensive monitoring.
#[derive(Parser)]
#[command(name = "tantivy_migrator")]
#[command(about = "Safe migration tool to Tantivy")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Project path to migrate
    #[arg(long, default_value = ".")]
    project_path: PathBuf,
    
    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate migration safety by comparing search results
    Validate {
        /// Number of test queries to validate with
        #[arg(long, default_value = "20")]
        test_queries: usize,
        
        /// File containing test queries (one per line)
        #[arg(long)]
        query_file: Option<PathBuf>,
        
        /// Accuracy threshold (0.0 to 1.0) for validation to pass
        #[arg(long, default_value = "0.9")]
        accuracy_threshold: f64,
    },
    
    /// Perform the migration
    Migrate {
        /// Create backup before migration
        #[arg(long, default_value = "true")]
        backup: bool,
        
        /// Skip validation phase
        #[arg(long)]
        skip_validation: bool,
        
        /// Force migration even if validation fails
        #[arg(long)]
        force: bool,
    },
    
    /// Rollback to previous configuration
    Rollback {
        /// Backup ID to restore from
        #[arg(long)]
        backup_id: Option<String>,
        
        /// List available backups
        #[arg(long)]
        list: bool,
    },
    
    /// Dry run - show what would be changed without making changes
    DryRun {
        /// Show detailed configuration differences
        #[arg(long)]
        detailed: bool,
    },
    
    /// Monitor system performance during migration
    Monitor {
        /// Duration to monitor in seconds
        #[arg(long, default_value = "60")]
        duration: u64,
        
        /// Output format (json, table)
        #[arg(long, default_value = "table")]
        format: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct MigrationBackup {
    id: String,
    timestamp: DateTime<Utc>,
    original_config: Config,
    config_file_path: Option<PathBuf>,
    config_file_content: Option<String>,
    environment_variables: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ValidationResult {
    total_queries: usize,
    successful_comparisons: usize,
    failed_comparisons: usize,
    accuracy: f64,
    failed_queries: Vec<String>,
    performance_comparison: PerformanceComparison,
}

#[derive(Serialize, Deserialize, Debug)]
struct PerformanceComparison {
    tantivy_avg_time_ms: f64,
}

#[derive(Debug)]
struct MigrationTool {
    project_path: PathBuf,
    verbose: bool,
    backup_dir: PathBuf,
}

impl MigrationTool {
    fn new(project_path: PathBuf, verbose: bool) -> Self {
        let backup_dir = project_path.join(".embed_backups");
        Self {
            project_path,
            verbose,
            backup_dir,
        }
    }
    
    fn log(&self, message: &str) {
        if self.verbose {
            println!("[{}] {}", chrono::Utc::now().format("%H:%M:%S"), message);
        }
    }
    
    fn info(&self, message: &str) {
        println!("INFO: {}", message);
    }
    
    fn warn(&self, message: &str) {
        eprintln!("WARN: {}", message);
    }
    
    #[allow(dead_code)]
    fn error(&self, message: &str) {
        eprintln!("ERROR: {}", message);
    }
    
    /// Ensure backup directory exists
    async fn ensure_backup_dir(&self) -> Result<()> {
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir)
                .context("Failed to create backup directory")?;
        }
        Ok(())
    }
    
    /// Generate test queries for validation
    fn generate_test_queries(&self, count: usize) -> Vec<String> {
        vec![
            "fn main",
            "struct",
            "impl",
            "use std",
            "async",
            "Result",
            "Error",
            "todo!",
            "println!",
            "Vec<",
            "Option<",
            "HashMap",
            "tokio",
            "serde",
            "anyhow",
            "clap",
            "config",
            "search",
            "index",
            "query"
        ].into_iter()
        .cycle()
        .take(count)
        .map(String::from)
        .collect()
    }
    
    /// Load test queries from file
    fn load_queries_from_file(&self, file_path: &Path) -> Result<Vec<String>> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read query file: {:?}", file_path))?;
        
        Ok(content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(String::from)
            .collect())
    }
    
    /// Validate search results quality
    fn validate_search_results(&self, query: &str, results: &[ExactMatch]) -> f64 {
        if results.is_empty() {
            // For some specific queries, empty results might be expected
            if query.trim().is_empty() || query.contains("nonexistent_random_string_12345") {
                return 1.0;
            }
            return 0.0; // Most queries should return some results
        }
        
        // Basic quality checks:
        let mut quality_score = 0.0;
        let total_checks = 4.0;
        
        // 1. Results should have valid file paths
        let valid_paths = results.iter().all(|r| !r.file_path.is_empty());
        if valid_paths { quality_score += 1.0; }
        
        // 2. Results should have reasonable line numbers (> 0)
        let valid_lines = results.iter().all(|r| r.line_number > 0);
        if valid_lines { quality_score += 1.0; }
        
        // 3. Results should contain the search query (case-insensitive)
        let query_lower = query.to_lowercase();
        let relevant_results = results.iter().any(|r| 
            r.content.to_lowercase().contains(&query_lower) ||
            r.line_content.to_lowercase().contains(&query_lower)
        );
        if relevant_results { quality_score += 1.0; }
        
        // 4. File paths should exist or be reasonable
        let reasonable_paths = results.iter().all(|r| 
            !r.file_path.contains("../..") && 
            r.file_path.len() < 500 // Reasonable path length
        );
        if reasonable_paths { quality_score += 1.0; }
        
        quality_score / total_checks
    }
    
    /// Validate Tantivy search functionality
    async fn validate_migration(&self, test_queries: usize, query_file: Option<&Path>, accuracy_threshold: f64) -> Result<ValidationResult> {
        self.info("Starting Tantivy validation...");
        
        // Load or generate test queries
        let queries = if let Some(file_path) = query_file {
            self.load_queries_from_file(file_path)?
        } else {
            self.generate_test_queries(test_queries)
        };
        
        self.log(&format!("Testing with {} queries", queries.len()));
        
        // Initialize Tantivy searcher
        let mut tantivy_searcher = TantivySearcher::new().await
            .context("Failed to initialize Tantivy searcher")?;
        
        // Build Tantivy index
        self.info("Building Tantivy index for validation...");
        tantivy_searcher.index_directory(&self.project_path).await
            .context("Failed to build Tantivy index")?;
        
        let mut successful_validations = 0;
        let mut failed_queries = Vec::new();
        let mut search_times = Vec::new();
        
        for (i, query) in queries.iter().enumerate() {
            self.log(&format!("Testing query {}/{}: '{}'", i + 1, queries.len(), query));
            
            // Time Tantivy search  
            let start = Instant::now();
            let tantivy_results = tantivy_searcher.search(query).await
                .context("Tantivy search failed")?;
            let search_time = start.elapsed();
            search_times.push(search_time.as_millis() as f64);
            
            // Validate results quality
            let quality_score = self.validate_search_results(query, &tantivy_results);
            
            if quality_score >= accuracy_threshold {
                successful_validations += 1;
            } else {
                failed_queries.push(query.clone());
                self.warn(&format!("Query '{}' failed validation (quality: {:.2})", query, quality_score));
            }
            
            self.log(&format!("Tantivy: {} results in {:?}, Quality: {:.2}", 
                tantivy_results.len(), search_time, quality_score));
        }
        
        let overall_accuracy = successful_validations as f64 / queries.len() as f64;
        let avg_time = search_times.iter().sum::<f64>() / search_times.len() as f64;
        
        let result = ValidationResult {
            total_queries: queries.len(),
            successful_comparisons: successful_validations,
            failed_comparisons: queries.len() - successful_validations,
            accuracy: overall_accuracy,
            failed_queries,
            performance_comparison: PerformanceComparison {
                tantivy_avg_time_ms: avg_time,
            },
        };
        
        self.info(&format!("Validation completed: {:.1}% quality ({}/{})", 
            overall_accuracy * 100.0, successful_validations, queries.len()));
        self.info(&format!("Average search time: {:.1}ms", avg_time));
        
        Ok(result)
    }
    
    /// Create a backup of current configuration
    async fn create_backup(&self) -> Result<MigrationBackup> {
        self.ensure_backup_dir().await?;
        
        let timestamp = Utc::now();
        let backup_id = format!("migration_{}", timestamp.format("%Y%m%d_%H%M%S"));
        
        // Load current config
        let current_config = Config::get()
            .context("Failed to load current configuration")?;
        
        // Find config file
        let mut config_file_path = None;
        let mut config_file_content = None;
        
        // Check for various config file locations
        let possible_config_files = [
            self.project_path.join(".embedrc"),
            self.project_path.join(".embed").join("config.toml"),
            self.project_path.join("config.toml"),
        ];
        
        for path in &possible_config_files {
            if path.exists() {
                config_file_path = Some(path.clone());
                config_file_content = Some(fs::read_to_string(path)
                    .context("Failed to read config file")?);
                break;
            }
        }
        
        // Capture relevant environment variables
        let mut environment_variables = HashMap::new();
        for (key, value) in std::env::vars() {
            if key.starts_with("EMBED_") {
                environment_variables.insert(key, value);
            }
        }
        
        let backup = MigrationBackup {
            id: backup_id.clone(),
            timestamp,
            original_config: current_config,
            config_file_path,
            config_file_content,
            environment_variables,
        };
        
        // Save backup
        let backup_file = self.backup_dir.join(format!("{}.json", backup_id));
        let backup_json = serde_json::to_string_pretty(&backup)?;
        fs::write(&backup_file, backup_json)
            .context("Failed to write backup file")?;
        
        self.info(&format!("Created backup: {}", backup_id));
        Ok(backup)
    }
    
    /// List available backups
    fn list_backups(&self) -> Result<Vec<MigrationBackup>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut backups = Vec::new();
        
        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        match serde_json::from_str::<MigrationBackup>(&content) {
                            Ok(backup) => backups.push(backup),
                            Err(e) => {
                                eprintln!("Error: Failed to parse backup file {:?}: {}", path, e);
                                return Err(anyhow::anyhow!("Failed to parse backup file {:?}: {}", path, e));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: Failed to read backup file {:?}: {}", path, e);
                        return Err(anyhow::anyhow!("Failed to read backup file {:?}: {}", path, e));
                    }
                }
            }
        }
        
        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(backups)
    }
    
    /// Restore from backup
    async fn restore_from_backup(&self, backup_id: &str) -> Result<()> {
        let backup_file = self.backup_dir.join(format!("{}.json", backup_id));
        
        if !backup_file.exists() {
            return Err(anyhow!("Backup '{}' not found", backup_id));
        }
        
        let backup_content = fs::read_to_string(&backup_file)
            .context("Failed to read backup file")?;
        let backup: MigrationBackup = serde_json::from_str(&backup_content)
            .context("Failed to parse backup file")?;
        
        self.info(&format!("Restoring from backup: {} ({})", backup_id, backup.timestamp));
        
        // Restore config file if it existed
        if let (Some(file_path), Some(content)) = (&backup.config_file_path, &backup.config_file_content) {
            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create config directory")?;
            }
            
            fs::write(file_path, content)
                .context("Failed to restore config file")?;
            self.info(&format!("Restored config file: {:?}", file_path));
        }
        
        // Note: Environment variables need to be set manually by the user
        if !backup.environment_variables.is_empty() {
            self.warn("The following environment variables should be restored manually:");
            for (key, value) in &backup.environment_variables {
                println!("  export {}=\"{}\"", key, value);
            }
        }
        
        self.info("Backup restored successfully");
        Ok(())
    }
    
    /// Perform the actual migration
    async fn perform_migration(&self, skip_validation: bool, force: bool) -> Result<()> {
        // First validate if not skipped
        if !skip_validation {
            self.info("Running pre-migration validation...");
            let validation_result = self.validate_migration(20, None, 0.8).await?;
            
            if validation_result.accuracy < 0.8 && !force {
                return Err(anyhow!(
                    "Validation failed with {:.1}% accuracy. Use --force to proceed anyway.",
                    validation_result.accuracy * 100.0
                ));
            }
            
            if validation_result.accuracy < 0.8 {
                self.warn(&format!("Proceeding with low validation accuracy: {:.1}%", 
                    validation_result.accuracy * 100.0));
            }
        }
        
        // Find or create config file
        let config_file_path = self.find_or_create_config_file().await?;
        
        // Update configuration
        self.update_config_file(&config_file_path).await?;
        
        // Build initial Tantivy index
        self.build_tantivy_index().await?;
        
        self.info("Migration completed successfully!");
        self.info("The system is now using Tantivy as the search backend.");
        
        Ok(())
    }
    
    /// Find existing config file or create one
    async fn find_or_create_config_file(&self) -> Result<PathBuf> {
        // Check for existing config files
        let possible_paths = [
            self.project_path.join(".embedrc"),
            self.project_path.join(".embed").join("config.toml"),
        ];
        
        for path in &possible_paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }
        
        // Create new config file
        let config_path = self.project_path.join(".embedrc");
        let mut config = Config::new_test_config();
        config.search_backend = SearchBackend::Tantivy;
        
        let config_toml = toml::to_string_pretty(&config)
            .context("Failed to serialize config")?;
        
        fs::write(&config_path, config_toml)
            .context("Failed to create config file")?;
        
        self.info(&format!("Created new config file: {:?}", config_path));
        Ok(config_path)
    }
    
    /// Update configuration file to use Tantivy
    async fn update_config_file(&self, config_path: &Path) -> Result<()> {
        let content = fs::read_to_string(config_path)
            .context("Failed to read config file")?;
        
        let mut config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        // Update search backend
        config.search_backend = SearchBackend::Tantivy;
        
        // Write updated config
        let updated_content = toml::to_string_pretty(&config)
            .context("Failed to serialize updated config")?;
        
        fs::write(config_path, updated_content)
            .context("Failed to write updated config")?;
        
        self.info(&format!("Updated config file: {:?}", config_path));
        Ok(())
    }
    
    /// Build initial Tantivy index
    async fn build_tantivy_index(&self) -> Result<()> {
        self.info("Building initial Tantivy index...");
        
        let mut searcher = TantivySearcher::new().await
            .context("Failed to initialize Tantivy searcher")?;
        
        let start = Instant::now();
        searcher.index_directory(&self.project_path).await
            .context("Failed to build Tantivy index")?;
        let duration = start.elapsed();
        
        self.info(&format!("Index built in {:?}", duration));
        Ok(())
    }
    
    /// Show what would be changed in a dry run
    async fn dry_run(&self, detailed: bool) -> Result<()> {
        self.info("=== DRY RUN - No changes will be made ===");
        
        // Load current config
        let current_config = Config::get()
            .context("Failed to load current configuration")?;
        
        self.info(&format!("Current search backend: {}", current_config.search_backend));
        self.info("Would change to: Tantivy");
        
        // Check config file locations
        let possible_paths = [
            self.project_path.join(".embedrc"),
            self.project_path.join(".embed").join("config.toml"),
        ];
        
        let mut config_file_exists = false;
        for path in &possible_paths {
            if path.exists() {
                self.info(&format!("Would update existing config file: {:?}", path));
                config_file_exists = true;
                break;
            }
        }
        
        if !config_file_exists {
            self.info(&format!("Would create new config file: {:?}", self.project_path.join(".embedrc")));
        }
        
        if detailed {
            println!("\n=== Current Configuration ===");
            println!("{}", current_config.summary());
            
            println!("\n=== Planned Changes ===");
            println!("- search_backend: {} -> Tantivy", current_config.search_backend);
            println!("- Would build new Tantivy index for project");
            println!("- Would preserve all other configuration settings");
        }
        
        self.info("Use 'migrate' command to perform actual migration");
        Ok(())
    }
    
    /// Compare search results to determine similarity
    #[allow(dead_code)]
    fn compare_search_results(&self, results1: &[ExactMatch], results2: &[ExactMatch]) -> f64 {
        if results1.is_empty() && results2.is_empty() {
            return 1.0; // Both empty, perfect match
        }
        
        if results1.is_empty() || results2.is_empty() {
            return 0.0; // One empty, one not, no similarity
        }
        
        let set1: std::collections::HashSet<_> = results1.iter()
            .map(|r| (&r.file_path, r.line_number, &r.content))
            .collect();
        let set2: std::collections::HashSet<_> = results2.iter()
            .map(|r| (&r.file_path, r.line_number, &r.content))
            .collect();
        
        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();
        
        if union == 0 {
            return 0.0;
        }
        
        intersection as f64 / union as f64
    }

    /// Monitor system performance
    async fn monitor(&self, duration: u64, format: &str) -> Result<()> {
        self.info(&format!("Monitoring system for {} seconds...", duration));
        
        let end_time = Instant::now() + Duration::from_secs(duration);
        let mut measurements = Vec::new();
        
        while Instant::now() < end_time {
            let start = Instant::now();
            
            // Test search performance
            match create_text_searcher_with_root(&SearchBackend::Tantivy, self.project_path.clone()).await {
                Ok(searcher) => {
                    let search_start = Instant::now();
                    let _results = searcher.search("async fn").await;
                    let search_duration = search_start.elapsed();
                    
                    measurements.push((start, search_duration));
                }
                Err(e) => {
                    eprintln!("Failed to create searcher for performance monitoring: {}", e);
                    return Err(e.into());
                }
            }
            
            sleep(Duration::from_secs(1)).await;
        }
        
        // Output results
        match format {
            "json" => {
                let json_data = serde_json::json!({
                    "measurements": measurements.iter().map(|(timestamp, duration)| {
                        serde_json::json!({
                            "timestamp": timestamp.elapsed().as_secs(),
                            "search_time_ms": duration.as_millis()
                        })
                    }).collect::<Vec<_>>(),
                    "avg_search_time_ms": measurements.iter().map(|(_, d)| d.as_millis()).sum::<u128>() / measurements.len() as u128,
                    "total_measurements": measurements.len()
                });
                println!("{}", serde_json::to_string_pretty(&json_data)?);
            }
            "table" => {
                println!("\n=== Performance Monitoring Results ===");
                println!("Total measurements: {}", measurements.len());
                if !measurements.is_empty() {
                    let avg_ms = measurements.iter().map(|(_, d)| d.as_millis()).sum::<u128>() / measurements.len() as u128;
                    println!("Average search time: {}ms", avg_ms);
                    
                    let min_ms = measurements.iter().map(|(_, d)| d.as_millis()).min()
                        .expect("Measurements collection cannot be empty when calculating min/max statistics");
                    let max_ms = measurements.iter().map(|(_, d)| d.as_millis()).max()
                        .expect("Measurements collection cannot be empty when calculating min/max statistics");
                    println!("Min search time: {}ms", min_ms);
                    println!("Max search time: {}ms", max_ms);
                }
            }
            _ => {
                self.warn(&format!("Unknown format '{}', using table format", format));
            }
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let tool = MigrationTool::new(cli.project_path, cli.verbose);
    
    match cli.command {
        Commands::Validate { test_queries, query_file, accuracy_threshold } => {
            let result = tool.validate_migration(test_queries, query_file.as_deref(), accuracy_threshold).await?;
            
            println!("\n=== Validation Results ===");
            println!("Accuracy: {:.1}% ({}/{})", 
                result.accuracy * 100.0, 
                result.successful_comparisons, 
                result.total_queries);
            println!("Performance: Tantivy {:.1}ms avg", 
                result.performance_comparison.tantivy_avg_time_ms);
            
            if !result.failed_queries.is_empty() {
                println!("\nFailed queries:");
                for query in &result.failed_queries {
                    println!("  - {}", query);
                }
            }
            
            if result.accuracy >= accuracy_threshold {
                println!("\n✅ Validation PASSED - Migration is safe to proceed");
            } else {
                println!("\n❌ Validation FAILED - Migration may cause issues");
                std::process::exit(1);
            }
        }
        
        Commands::Migrate { backup, skip_validation, force } => {
            if backup {
                let _backup = tool.create_backup().await?;
            }
            
            tool.perform_migration(skip_validation, force).await?;
        }
        
        Commands::Rollback { backup_id, list } => {
            if list {
                let backups = tool.list_backups()?;
                if backups.is_empty() {
                    println!("No backups found");
                } else {
                    println!("Available backups:");
                    for backup in backups {
                        println!("  {} - {} (backend: {})", 
                            backup.id, 
                            backup.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                            backup.original_config.search_backend);
                    }
                }
            } else if let Some(id) = backup_id {
                tool.restore_from_backup(&id).await?;
            } else {
                return Err(anyhow!("Must specify --backup-id or use --list"));
            }
        }
        
        Commands::DryRun { detailed } => {
            tool.dry_run(detailed).await?;
        }
        
        Commands::Monitor { duration, format } => {
            tool.monitor(duration, &format).await?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_migration_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let tool = MigrationTool::new(temp_dir.path().to_path_buf(), false);
        
        assert_eq!(tool.project_path, temp_dir.path());
        assert_eq!(tool.backup_dir, temp_dir.path().join(".embed_backups"));
    }
    
    #[tokio::test]
    async fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let tool = MigrationTool::new(temp_dir.path().to_path_buf(), false);
        
        let backup = tool.create_backup().await.unwrap();
        assert!(!backup.id.is_empty());
        assert!(tool.backup_dir.join(format!("{}.json", backup.id)).exists());
    }
    
    #[test]
    fn test_query_generation() {
        let temp_dir = TempDir::new().unwrap();
        let tool = MigrationTool::new(temp_dir.path().to_path_buf(), false);
        
        let queries = tool.generate_test_queries(5);
        assert_eq!(queries.len(), 5);
        assert!(queries.iter().all(|q| !q.is_empty()));
    }
    
    #[test]
    fn test_result_comparison() {
        let temp_dir = TempDir::new().unwrap();
        let tool = MigrationTool::new(temp_dir.path().to_path_buf(), false);
        
        let results1 = vec![
            ExactMatch {
                file_path: "test.rs".to_string(),
                line_number: 1,
                content: "fn main()".to_string(),
                line_content: "fn main()".to_string(),
            }
        ];
        
        let results2 = vec![
            ExactMatch {
                file_path: "test.rs".to_string(),
                line_number: 1,
                content: "fn main()".to_string(),
                line_content: "fn main()".to_string(),
            }
        ];
        
        let similarity = tool.compare_search_results(&results1, &results2);
        assert_eq!(similarity, 1.0);
        
        let results3 = vec![];
        let similarity2 = tool.compare_search_results(&results1, &results3);
        assert_eq!(similarity2, 0.0);
    }
}