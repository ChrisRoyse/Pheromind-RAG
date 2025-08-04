#!/usr/bin/env node
/**
 * Model Download Script for MCP RAG Indexer
 * Downloads and caches ML models for offline operation
 */

const fs = require('fs-extra');
const path = require('path');
const fetch = require('node-fetch');
const chalk = require('chalk');
const ora = require('ora');

const PACKAGE_ROOT = path.join(__dirname, '..');

// ML Model configurations
const ML_MODELS = {
  'sentence-transformers/all-MiniLM-L6-v2': {
    files: [
      'config.json',
      'pytorch_model.bin',
      'sentence_bert_config.json',
      'tokenizer.json',
      'tokenizer_config.json',
      'vocab.txt',
      'modules.json',
      '1_Pooling/config.json'
    ],
    baseUrl: 'https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/',
    size: '~90MB',
    description: 'Sentence transformer model for semantic embeddings'
  },
  'sentence-transformers/all-mpnet-base-v2': {
    files: [
      'config.json',
      'pytorch_model.bin',
      'sentence_bert_config.json',
      'tokenizer.json',
      'tokenizer_config.json',
      'vocab.txt',
      'modules.json',
      '1_Pooling/config.json'
    ],
    baseUrl: 'https://huggingface.co/sentence-transformers/all-mpnet-base-v2/resolve/main/',
    size: '~420MB',
    description: 'Higher quality sentence transformer (larger size)'
  }
};

class ModelDownloader {
  constructor() {
    this.spinner = null;
  }
  
  async downloadAll() {
    console.log(chalk.blue.bold('🧠 ML Model Downloader'));
    console.log(chalk.gray('Downloads and caches machine learning models for offline operation'));
    console.log();
    
    try {
      // Clean models directory
      await this.cleanModelsDir();
      
      // Download default model
      const defaultModel = 'sentence-transformers/all-MiniLM-L6-v2';
      await this.downloadModel(defaultModel, ML_MODELS[defaultModel]);
      
      console.log();
      console.log(chalk.green.bold('✅ Default ML model downloaded successfully!'));
      console.log(chalk.cyan('To download additional models, use:'));
      console.log(chalk.gray('node download-models.js --model <model-name>'));
      
    } catch (error) {
      console.log();
      console.error(chalk.red.bold('❌ Model download failed:'), error.message);
      process.exit(1);
    }
  }
  
  async downloadSpecific(modelNames) {
    console.log(chalk.blue.bold(`🧠 Downloading models: ${modelNames.join(', ')}`));
    console.log();
    
    try {
      for (const modelName of modelNames) {
        if (!ML_MODELS[modelName]) {
          throw new Error(`Unknown model: ${modelName}`);
        }
        
        await this.downloadModel(modelName, ML_MODELS[modelName]);
      }
      
      console.log();
      console.log(chalk.green.bold('✅ Selected models downloaded successfully!'));
      
    } catch (error) {
      console.log();
      console.error(chalk.red.bold('❌ Model download failed:'), error.message);
      process.exit(1);
    }
  }
  
  async cleanModelsDir() {
    this.spinner = ora('Cleaning models directory...').start();
    
    try {
      const modelsDir = path.join(PACKAGE_ROOT, 'models');
      if (await fs.pathExists(modelsDir)) {
        await fs.remove(modelsDir);
      }
      await fs.ensureDir(modelsDir);
      
      this.spinner.succeed(chalk.green('✓ Models directory cleaned'));
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Failed to clean models directory'));
      throw error;
    }
  }
  
  async downloadModel(modelName, config) {
    this.spinner = ora(`Downloading ${modelName} (${config.size})...`).start();
    
    try {
      const modelDir = path.join(PACKAGE_ROOT, 'models', modelName);
      await fs.ensureDir(modelDir);
      
      let downloadedFiles = 0;
      const totalFiles = config.files.length;
      
      for (const file of config.files) {
        this.spinner.text = `Downloading ${modelName}/${file} (${downloadedFiles + 1}/${totalFiles})`;
        
        const url = `${config.baseUrl}${file}`;
        const filePath = path.join(modelDir, file);
        
        // Ensure subdirectory exists for nested files
        await fs.ensureDir(path.dirname(filePath));
        
        try {
          const response = await fetch(url, {
            timeout: 180000, // 3 minute timeout per file
            headers: {
              'User-Agent': 'mcp-rag-indexer-build'
            }
          });
          
          if (!response.ok) {
            // Some files might be optional
            if (response.status === 404 && file.includes('/')) {
              console.log(chalk.yellow(`⚠ Optional file not found: ${file}`));
              continue;
            }
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
          }
          
          const buffer = await response.buffer();
          await fs.writeFile(filePath, buffer);
          
          downloadedFiles++;
          
        } catch (error) {
          if (file.includes('/')) {
            // Optional nested file
            console.log(chalk.yellow(`⚠ Optional file failed: ${file} - ${error.message}`));
            continue;
          }
          throw error;
        }
      }
      
      // Validate essential files exist
      await this.validateModel(modelName, modelDir);
      
      this.spinner.succeed(chalk.green(`✓ ${modelName} downloaded (${downloadedFiles} files)`));
      
    } catch (error) {
      this.spinner.fail(chalk.red(`✗ Failed to download ${modelName}`));
      throw error;
    }
  }
  
  async validateModel(modelName, modelDir) {
    // Check for essential files
    const essentialFiles = ['config.json', 'pytorch_model.bin', 'tokenizer.json'];
    
    for (const file of essentialFiles) {
      const filePath = path.join(modelDir, file);
      if (!await fs.pathExists(filePath)) {
        throw new Error(`Essential model file missing: ${file}`);
      }
    }
    
    // Check file sizes (basic validation)
    const modelBinPath = path.join(modelDir, 'pytorch_model.bin');
    const stat = await fs.stat(modelBinPath);
    
    if (stat.size < 1000000) { // Less than 1MB is suspicious
      throw new Error('Model file appears to be truncated or corrupted');
    }
  }
  
  async listAvailableModels() {
    console.log(chalk.blue.bold('Available Models:'));
    console.log();
    
    for (const [modelName, config] of Object.entries(ML_MODELS)) {
      console.log(`${chalk.cyan(modelName)}`);
      console.log(`  ${chalk.gray(config.description)}`);
      console.log(`  ${chalk.gray('Size:')} ${config.size}`);
      console.log(`  ${chalk.gray('Files:')} ${config.files.length}`);
      console.log();
    }
  }
  
  async getModelsSummary() {
    const modelsDir = path.join(PACKAGE_ROOT, 'models');
    
    if (!await fs.pathExists(modelsDir)) {
      return { total: 0, models: [], totalSize: 0 };
    }
    
    const modelDirs = await fs.readdir(modelsDir);
    const validModels = [];
    let totalSize = 0;
    
    for (const modelDir of modelDirs) {
      const modelPath = path.join(modelsDir, modelDir);
      const stat = await fs.stat(modelPath);
      
      if (stat.isDirectory()) {
        // Check if it's a valid model
        const configPath = path.join(modelPath, 'config.json');
        const modelBinPath = path.join(modelPath, 'pytorch_model.bin');
        
        if (await fs.pathExists(configPath) && await fs.pathExists(modelBinPath)) {
          const modelBinStat = await fs.stat(modelBinPath);
          totalSize += modelBinStat.size;
          validModels.push(modelDir);
        }
      }
    }
    
    return {
      total: validModels.length,
      models: validModels,
      totalSize: totalSize
    };
  }
  
  formatSize(bytes) {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }
  
  async optimizeModels() {
    this.spinner = ora('Optimizing models for production...').start();
    
    try {
      const modelsDir = path.join(PACKAGE_ROOT, 'models');
      
      if (!await fs.pathExists(modelsDir)) {
        throw new Error('No models found to optimize');
      }
      
      // Remove unnecessary files that increase package size
      const unnecessaryFiles = [
        'training_args.bin',
        'optimizer.pt',
        'scheduler.pt',
        'scaler.pt',
        'trainer_state.json',
        '.gitattributes',
        'README.md'
      ];
      
      let removedCount = 0;
      let savedBytes = 0;
      
      const modelDirs = await fs.readdir(modelsDir);
      
      for (const modelDir of modelDirs) {
        const modelPath = path.join(modelsDir, modelDir);
        
        for (const file of unnecessaryFiles) {
          const filePath = path.join(modelPath, file);
          
          if (await fs.pathExists(filePath)) {
            const stat = await fs.stat(filePath);
            await fs.remove(filePath);
            removedCount++;
            savedBytes += stat.size;
          }
        }
      }
      
      this.spinner.succeed(chalk.green(`✓ Optimized models (removed ${removedCount} files, saved ${this.formatSize(savedBytes)})`));
      
    } catch (error) {
      this.spinner.fail(chalk.red('✗ Model optimization failed'));
      throw error;
    }
  }
}

// CLI Interface
async function main() {
  const args = process.argv.slice(2);
  const downloader = new ModelDownloader();
  
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
${chalk.blue.bold('ML Model Downloader')}

Usage:
  node download-models.js [options]

Options:
  --help, -h        Show this help message
  --list            List available models
  --model <name>    Download specific model
  --clean           Clean models directory only
  --summary         Show download summary
  --optimize        Optimize models for production

Examples:
  node download-models.js                                    # Download default model
  node download-models.js --model sentence-transformers/all-mpnet-base-v2
  node download-models.js --list                             # List available models
  node download-models.js --summary                          # Show what's downloaded
  node download-models.js --optimize                         # Optimize existing models
    `);
    return;
  }
  
  if (args.includes('--list')) {
    await downloader.listAvailableModels();
    return;
  }
  
  if (args.includes('--clean')) {
    await downloader.cleanModelsDir();
    return;
  }
  
  if (args.includes('--summary')) {
    const summary = await downloader.getModelsSummary();
    console.log(chalk.blue.bold('Models Summary:'));
    console.log(`Total models: ${summary.total}`);
    console.log(`Models: ${summary.models.join(', ') || 'None'}`);
    console.log(`Total size: ${downloader.formatSize(summary.totalSize)}`);
    return;
  }
  
  if (args.includes('--optimize')) {
    await downloader.optimizeModels();
    return;
  }
  
  // Check for --model flag
  const modelIndex = args.indexOf('--model');
  if (modelIndex !== -1 && modelIndex + 1 < args.length) {
    const modelName = args[modelIndex + 1];
    await downloader.downloadSpecific([modelName]);
  } else {
    await downloader.downloadAll();
  }
}

if (require.main === module) {
  main().catch((error) => {
    console.error(chalk.red('Fatal error:'), error);
    process.exit(1);
  });
}

module.exports = { ModelDownloader, ML_MODELS };