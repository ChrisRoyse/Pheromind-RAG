# Tantivy Migration Tool

A comprehensive, safe migration tool for transitioning production systems from Ripgrep to Tantivy search backend with validation, backup/restore capabilities, and monitoring.

## Overview

The Tantivy Migration Tool provides a production-ready solution for migrating from the Ripgrep text search backend to Tantivy full-text search. It includes comprehensive safety features:

- **Validation**: Compare search results between backends to ensure compatibility
- **Backup/Restore**: Atomic backup and rollback capabilities  
- **Monitoring**: Performance tracking and system health monitoring
- **Dry Run**: Preview changes without applying them
- **Progress Tracking**: Detailed logging and progress reporting

## Installation

Build the migration tool:

```bash
cargo build --bin tantivy_migrator --release
```

The binary will be available at `target/release/tantivy_migrator` (or `.exe` on Windows).

## Quick Start

### 1. Dry Run (Recommended First Step)

Preview what changes would be made without applying them:

```bash
tantivy_migrator --verbose dry-run --detailed
```

### 2. Validate Migration Safety

Test search compatibility between backends:

```bash
tantivy_migrator --verbose validate --test-queries 20 --accuracy-threshold 0.9
```

### 3. Perform Migration

Execute the migration with backup:

```bash
tantivy_migrator --verbose migrate --backup
```

### 4. Monitor Performance

Monitor system performance after migration:

```bash
tantivy_migrator monitor --duration 120 --format table
```

## Commands

### `validate`

Validate migration safety by comparing search results between Ripgrep and Tantivy.

```bash
tantivy_migrator validate [OPTIONS]
```

**Options:**
- `--test-queries <N>` - Number of test queries to validate (default: 20)
- `--query-file <FILE>` - File containing test queries (one per line)
- `--accuracy-threshold <FLOAT>` - Accuracy threshold (0.0-1.0) for validation to pass (default: 0.9)

**Example:**
```bash
# Basic validation
tantivy_migrator validate --test-queries 50 --accuracy-threshold 0.85

# Validation with custom queries
tantivy_migrator validate --query-file test_queries.txt
```

**Interpreting Results:**
- **Accuracy**: Percentage of queries that produce similar results between backends
- **Performance**: Speed comparison showing Tantivy's performance advantage
- **Failed Queries**: Specific queries where results differed significantly

### `migrate`

Perform the actual migration from Ripgrep to Tantivy.

```bash
tantivy_migrator migrate [OPTIONS]
```

**Options:**
- `--backup` - Create backup before migration (default: true)
- `--skip-validation` - Skip validation phase
- `--force` - Force migration even if validation fails

**Example:**
```bash
# Safe migration with backup and validation
tantivy_migrator --verbose migrate

# Force migration without validation (not recommended)
tantivy_migrator migrate --skip-validation --force
```

### `rollback`

Rollback to a previous configuration using backup data.

```bash
tantivy_migrator rollback [OPTIONS]
```

**Options:**
- `--backup-id <ID>` - Specific backup ID to restore
- `--list` - List available backups

**Example:**
```bash
# List available backups
tantivy_migrator rollback --list

# Restore specific backup
tantivy_migrator rollback --backup-id migration_20250106_143022
```

### `dry-run`

Preview changes without applying them.

```bash
tantivy_migrator dry-run [OPTIONS]
```

**Options:**
- `--detailed` - Show detailed configuration differences

**Example:**
```bash
tantivy_migrator --verbose dry-run --detailed
```

### `monitor`

Monitor system performance during or after migration.

```bash
tantivy_migrator monitor [OPTIONS]
```

**Options:**
- `--duration <SECONDS>` - Duration to monitor (default: 60)
- `--format <FORMAT>` - Output format: json, table (default: table)

**Example:**
```bash
# Monitor for 5 minutes with table output
tantivy_migrator monitor --duration 300 --format table

# Monitor with JSON output for automation
tantivy_migrator monitor --duration 60 --format json
```

## Global Options

- `--project-path <PATH>` - Project path to migrate (default: current directory)
- `--verbose` - Enable verbose logging
- `--help` - Show help information

## Configuration

The tool works with the existing configuration system:

### Configuration Files

The tool will automatically find and update configuration in these locations (in order):
1. `.embedrc` (project root)
2. `.embed/config.toml` (project root)
3. `config.toml` (project root)

If no configuration file exists, the tool will create `.embedrc`.

### Environment Variables

Environment variables with `EMBED_` prefix will be captured in backups but must be manually restored:

```bash
export EMBED_SEARCH_BACKEND=tantivy
export EMBED_MAX_SEARCH_RESULTS=50
```

## Migration Process

### Phase 1: Pre-Migration

1. **Backup Creation**: Automatically creates timestamped backup of:
   - Current configuration
   - Configuration files
   - Relevant environment variables

2. **Validation** (if not skipped):
   - Builds temporary Tantivy index
   - Runs test queries on both backends
   - Compares results for accuracy
   - Measures performance differences

### Phase 2: Migration

1. **Configuration Update**: Changes `search_backend` to `Tantivy`
2. **Index Building**: Creates initial Tantivy index for the project
3. **Verification**: Confirms migration completed successfully

### Phase 3: Post-Migration

1. **Monitoring**: Optional performance monitoring
2. **Rollback**: Available if issues are detected

## Understanding Validation Results

### Search Accuracy

The validation compares search results between Ripgrep and Tantivy:

- **Perfect Match (1.0)**: Identical results
- **High Similarity (0.8-0.99)**: Minor differences, safe to proceed
- **Medium Similarity (0.5-0.79)**: Noticeable differences, review failed queries
- **Low Similarity (<0.5)**: Significant differences, migration not recommended

### Common Differences

Different search behaviors are expected:

1. **Ripgrep**: Substring matching, case-insensitive options
2. **Tantivy**: Tokenized search, exact phrase matching

**Example differences:**
- Query `"use std"` might find different matches due to tokenization
- Special characters and punctuation are handled differently
- Word boundaries and stemming can affect results

### Performance Metrics

The tool reports:
- **Average search time** for each backend
- **Speedup factor** (how much faster Tantivy is)
- **Individual query performance**

Tantivy typically shows 2-10x speedup over Ripgrep.

## Backup and Restore

### Backup Structure

Backups are stored in `.embed_backups/` with the format:
```
.embed_backups/migration_YYYYMMDD_HHMMSS.json
```

Each backup contains:
- Original configuration
- Configuration file path and content
- Environment variables
- Timestamp and metadata

### Restore Process

Restoration includes:
1. **Config File**: Automatically restored
2. **Environment Variables**: Listed for manual restoration
3. **Verification**: Confirms restore completed

**Manual Environment Variable Restoration:**
```bash
# The tool will output commands like:
export EMBED_SEARCH_BACKEND="ripgrep"
export EMBED_MAX_SEARCH_RESULTS="20"
```

## Error Handling

### Validation Failures

If validation fails (accuracy below threshold):
- Review failed queries in the output
- Consider adjusting accuracy threshold
- Use `--force` flag to proceed anyway (not recommended for production)

### Migration Failures

If migration fails:
- Backup is automatically preserved
- No changes are applied (atomic operation)
- Use rollback to restore previous state

### Common Issues

1. **Tantivy Index Building Fails**:
   - Check disk space
   - Verify file permissions
   - Ensure project path is correct

2. **Configuration File Not Found**:
   - Tool will create default configuration
   - Review created config before proceeding

3. **Permission Issues**:
   - Ensure write access to project directory
   - Check backup directory permissions

## Production Deployment

### Recommended Process

1. **Development/Testing**:
   ```bash
   # Test on development environment
   tantivy_migrator --verbose dry-run --detailed
   tantivy_migrator --verbose validate --test-queries 100
   ```

2. **Staging Environment**:
   ```bash
   # Full migration test
   tantivy_migrator --verbose migrate --backup
   # Performance testing
   tantivy_migrator monitor --duration 300
   ```

3. **Production Deployment**:
   ```bash
   # Final migration
   tantivy_migrator --verbose migrate --backup
   # Continuous monitoring
   tantivy_migrator monitor --duration 600 --format json > migration_metrics.json
   ```

### Rollback Plan

Always have a rollback plan:

```bash
# List backups
tantivy_migrator rollback --list

# Rollback if issues detected
tantivy_migrator rollback --backup-id <latest-backup-id>

# Verify rollback
tantivy_migrator dry-run --detailed
```

### Monitoring and Alerts

Set up monitoring for:
- Search performance metrics
- Error rates
- System resource usage
- Index building progress

## Troubleshooting

### Issue: Low Validation Accuracy

**Symptoms**: Validation reports <80% accuracy

**Solutions**:
1. Review failed queries in output
2. Consider if differences are acceptable for your use case
3. Adjust accuracy threshold if appropriate
4. Create custom query file with representative queries

### Issue: Migration Hangs During Index Building

**Symptoms**: Process stops during "Building initial Tantivy index"

**Solutions**:
1. Check available disk space
2. Monitor system resources (CPU, memory)
3. Verify no file locks on project directory
4. Consider indexing smaller subdirectories first

### Issue: Rollback Incomplete

**Symptoms**: Configuration restored but environment variables not set

**Solutions**:
1. Manually apply environment variable commands from rollback output
2. Restart application/services using the configuration
3. Verify configuration loading with `dry-run`

### Issue: Performance Regression

**Symptoms**: Search slower after migration

**Solutions**:
1. Verify Tantivy index built successfully
2. Check index size and structure
3. Monitor system resources during search
4. Consider index optimization settings

## Advanced Usage

### Custom Query Files

Create test files for validation:

```text
# test_queries.txt
fn main
struct User
impl Debug
use std::collections
async fn process
Result<Vec<String>>
```

### Automation Integration

Use JSON output for automation:

```bash
# Automated validation
RESULT=$(tantivy_migrator validate --test-queries 50 --accuracy-threshold 0.9)
if [ $? -eq 0 ]; then
    echo "Validation passed, proceeding with migration"
    tantivy_migrator migrate --backup
else
    echo "Validation failed, aborting migration"
    exit 1
fi
```

### Continuous Monitoring

Set up periodic monitoring:

```bash
#!/bin/bash
# monitor_search_performance.sh
while true; do
    tantivy_migrator monitor --duration 60 --format json >> search_metrics.log
    sleep 300  # Wait 5 minutes
done
```

## Security Considerations

1. **Backup Storage**: Backups may contain sensitive configuration
2. **File Permissions**: Ensure appropriate access controls
3. **Environment Variables**: May contain secrets, handle carefully
4. **Index Data**: Consider if indexed content is sensitive

## Support and Contributing

For issues, questions, or contributions:
1. Check existing issues and documentation
2. Provide detailed reproduction steps
3. Include relevant logs and configuration
4. Test with `--verbose` flag for detailed output

## Changelog

### v1.0.0
- Initial release
- Full validation, migration, and rollback support
- Comprehensive monitoring and reporting
- Production-ready safety features