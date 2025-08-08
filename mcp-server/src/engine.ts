import { promises as fs } from 'fs';
import { extname } from 'path';
import { glob } from 'glob';

/**
 * Search engine types supported by the system
 */
export enum EngineType {
  EXACT = 'exact',
  SEMANTIC = 'semantic', 
  HYBRID = 'hybrid',
  NEURAL = 'neural'
}

/**
 * File content with metadata
 */
export interface IndexedFile {
  path: string;
  content: string;
  size: number;
  lastModified: number;
  language: string;
  embeddings?: {
    [EngineType.SEMANTIC]?: number[];
    [EngineType.HYBRID]?: number[];
    [EngineType.NEURAL]?: number[];
  };
}

/**
 * Search result from an engine
 */
export interface SearchResult {
  engine: EngineType;
  results: Array<{
    file: IndexedFile;
    score: number;
    matches?: Array<{
      line: number;
      content: string;
      score: number;
    }>;
  }>;
  executionTime: number;
  error?: string;
}

/**
 * Parallel search results across all engines
 */
export interface ParallelSearchResults {
  query: string;
  results: SearchResult[];
  totalTime: number;
}

/**
 * Indexing status information
 */
export interface IndexStatus {
  isIndexing: boolean;
  filesIndexed: number;
  totalFiles: number;
  currentFile?: string;
  startTime?: number;
  estimatedCompletion?: number;
  errors: string[];
}

/**
 * Main embedding engine that orchestrates all search types
 */
export class EmbeddingEngine {
  private indexedFiles: Map<string, IndexedFile> = new Map();
  private watchers: Map<string, any> = new Map();
  private indexStatus: IndexStatus = {
    isIndexing: false,
    filesIndexed: 0,
    totalFiles: 0,
    errors: []
  };

  // Native bridge to Rust implementations (loaded dynamically)
  private rustBridge?: any;

  constructor() {
    this.loadRustBridge();
  }

  /**
   * Load Rust bridge for high-performance operations
   */
  private async loadRustBridge(): Promise<void> {
    try {
      // Attempt to load native module - may not exist during development
      const bridgePath = '../../mcp-bridge/index.node';
      const module = await import(/* @vite-ignore */ bridgePath).catch(() => null);
      if (module) {
        this.rustBridge = module;
        console.log('[Engine] Rust bridge loaded successfully');
      } else {
        console.log('[Engine] Rust bridge not available, using JavaScript fallback');
      }
    } catch {
      console.log('[Engine] Rust bridge not available, using JavaScript fallback');
    }
  }

  /**
   * Index a directory with optional file watching
   */
  async indexDirectory(
    directory: string, 
    options: {
      extensions?: string[];
      watchFiles?: boolean;
      excludePatterns?: string[];
      maxFileSize?: number;
    } = {}
  ): Promise<void> {
    const {
      extensions = ['.ts', '.js', '.py', '.rs', '.cpp', '.h', '.md', '.txt'],
      watchFiles = false,
      excludePatterns = ['node_modules/**', '.git/**', 'dist/**', 'build/**'],
      maxFileSize = 1024 * 1024 // 1MB
    } = options;

    this.indexStatus.isIndexing = true;
    this.indexStatus.startTime = Date.now();
    this.indexStatus.errors = [];

    try {
      // Find all files to index
      const patterns = extensions.map(ext => `**/*${ext}`);
      const files: string[] = [];
      
      for (const pattern of patterns) {
        const matches = await glob(pattern, {
          cwd: directory,
          ignore: excludePatterns,
          absolute: true
        });
        files.push(...matches);
      }

      this.indexStatus.totalFiles = files.length;
      this.indexStatus.filesIndexed = 0;

      // Index files in parallel batches
      const batchSize = 10;
      for (let i = 0; i < files.length; i += batchSize) {
        const batch = files.slice(i, i + batchSize);
        await Promise.all(
          batch.map(async (filePath) => {
            try {
              await this.indexFile(filePath, maxFileSize);
              this.indexStatus.filesIndexed++;
              this.indexStatus.currentFile = filePath;
            } catch (error) {
              const errorMsg = `Failed to index ${filePath}: ${error instanceof Error ? error.message : String(error)}`;
              this.indexStatus.errors.push(errorMsg);
              console.error('[Engine]', errorMsg);
            }
          })
        );
      }

      // Set up file watching if requested
      if (watchFiles) {
        await this.setupFileWatcher(directory, extensions, excludePatterns, maxFileSize);
      }

    } finally {
      this.indexStatus.isIndexing = false;
      delete this.indexStatus.currentFile;
    }
  }

  /**
   * Index a single file
   */
  private async indexFile(filePath: string, maxFileSize: number): Promise<void> {
    const stats = await fs.stat(filePath);
    
    if (stats.size > maxFileSize) {
      throw new Error(`File too large: ${stats.size} bytes`);
    }

    const content = await fs.readFile(filePath, 'utf8');
    const language = this.detectLanguage(filePath);

    const indexedFile: IndexedFile = {
      path: filePath,
      content,
      size: stats.size,
      lastModified: stats.mtime.getTime(),
      language,
      embeddings: {}
    };

    // Generate embeddings for semantic search engines
    if (this.rustBridge) {
      try {
        // Use Rust implementations for better performance
        indexedFile.embeddings = {
          [EngineType.SEMANTIC]: await this.rustBridge.generateSemanticEmbedding(content),
          [EngineType.HYBRID]: await this.rustBridge.generateHybridEmbedding(content),
          [EngineType.NEURAL]: await this.rustBridge.generateNeuralEmbedding(content)
        };
      } catch (error) {
        console.warn('[Engine] Rust embedding generation failed, using fallback');
        await this.generateFallbackEmbeddings(indexedFile);
      }
    } else {
      await this.generateFallbackEmbeddings(indexedFile);
    }

    this.indexedFiles.set(filePath, indexedFile);
  }

  /**
   * Generate embeddings using JavaScript fallback
   */
  private async generateFallbackEmbeddings(file: IndexedFile): Promise<void> {
    // Simple fallback embedding generation
    // In production, this would use actual ML models
    const tokens = file.content.toLowerCase().split(/\s+/);
    const embedding = new Array(384).fill(0);
    
    // Simple hash-based embedding for demonstration
    for (let i = 0; i < tokens.length && i < 384; i++) {
      const token = tokens[i];
      embedding[i % 384] += this.simpleHash(token) / 1000000;
    }

    file.embeddings = {
      [EngineType.SEMANTIC]: embedding,
      [EngineType.HYBRID]: embedding.map(x => x * 0.8),
      [EngineType.NEURAL]: embedding.map(x => Math.tanh(x))
    };
  }

  /**
   * Simple hash function for fallback embeddings
   */
  private simpleHash(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash);
  }

  /**
   * Set up file watcher for automatic index updates
   */
  private async setupFileWatcher(
    directory: string,
    extensions: string[],
    excludePatterns: string[],
    maxFileSize: number
  ): Promise<void> {
    if (this.watchers.has(directory)) {
      this.watchers.get(directory)?.close();
    }

    try {
      // Dynamic import of chokidar - may not be installed
      const chokidarModule = await import('chokidar').catch(() => null);
      if (!chokidarModule) {
        console.warn('[Engine] chokidar not installed, file watching disabled');
        return;
      }
      
      const watcher = chokidarModule.default.watch(directory, {
        ignored: excludePatterns,
        persistent: true,
        ignoreInitial: true
      });

      watcher
        .on('add', async (filePath: string) => {
          if (this.shouldIndexFile(filePath, extensions)) {
            try {
              await this.indexFile(filePath, maxFileSize);
              console.log('[Engine] Indexed new file:', filePath);
            } catch (error) {
              console.error('[Engine] Failed to index new file:', filePath, error);
            }
          }
        })
        .on('change', async (filePath: string) => {
          if (this.shouldIndexFile(filePath, extensions)) {
            try {
              await this.indexFile(filePath, maxFileSize);
              console.log('[Engine] Reindexed changed file:', filePath);
            } catch (error) {
              console.error('[Engine] Failed to reindex file:', filePath, error);
            }
          }
        })
        .on('unlink', (filePath: string) => {
          this.indexedFiles.delete(filePath);
          console.log('[Engine] Removed file from index:', filePath);
        });

      this.watchers.set(directory, watcher);
    } catch (error) {
      console.warn('[Engine] File watching not available:', error);
    }
  }

  /**
   * Check if file should be indexed based on extension
   */
  private shouldIndexFile(filePath: string, extensions: string[]): boolean {
    const ext = extname(filePath);
    return extensions.includes(ext);
  }

  /**
   * Detect programming language from file extension
   */
  private detectLanguage(filePath: string): string {
    const ext = extname(filePath).toLowerCase();
    const languageMap: Record<string, string> = {
      '.ts': 'typescript',
      '.js': 'javascript',
      '.py': 'python',
      '.rs': 'rust',
      '.cpp': 'cpp',
      '.c': 'c',
      '.h': 'c',
      '.hpp': 'cpp',
      '.java': 'java',
      '.go': 'go',
      '.md': 'markdown',
      '.txt': 'text'
    };
    
    return languageMap[ext] || 'unknown';
  }

  /**
   * Execute parallel search across all engines
   */
  async parallelSearch(
    query: string,
    options: {
      maxResults?: number;
      scoreThreshold?: number;
      engines?: EngineType[];
    } = {}
  ): Promise<ParallelSearchResults> {
    const {
      maxResults = 10,
      scoreThreshold = 0.1,
      engines = [EngineType.EXACT, EngineType.SEMANTIC, EngineType.HYBRID, EngineType.NEURAL]
    } = options;

    const startTime = Date.now();
    
    // Execute searches in parallel
    const searchPromises = engines.map(async (engine): Promise<SearchResult> => {
      const engineStartTime = Date.now();
      
      try {
        const results = await this.searchWithEngine(engine, query, maxResults, scoreThreshold);
        return {
          engine,
          results,
          executionTime: Date.now() - engineStartTime
        };
      } catch (error) {
        return {
          engine,
          results: [],
          executionTime: Date.now() - engineStartTime,
          error: error instanceof Error ? error.message : String(error)
        };
      }
    });

    const results = await Promise.all(searchPromises);

    return {
      query,
      results,
      totalTime: Date.now() - startTime
    };
  }

  /**
   * Search with a specific engine
   */
  private async searchWithEngine(
    engine: EngineType,
    query: string,
    maxResults: number,
    scoreThreshold: number
  ): Promise<SearchResult['results']> {
    const files = Array.from(this.indexedFiles.values());
    
    switch (engine) {
      case EngineType.EXACT:
        return this.exactSearch(files, query, maxResults);
      
      case EngineType.SEMANTIC:
        return this.semanticSearch(files, query, maxResults, scoreThreshold);
      
      case EngineType.HYBRID:
        return this.hybridSearch(files, query, maxResults, scoreThreshold);
      
      case EngineType.NEURAL:
        return this.neuralSearch(files, query, maxResults, scoreThreshold);
      
      default:
        throw new Error(`Unsupported engine type: ${engine}`);
    }
  }

  /**
   * Exact text search
   */
  private async exactSearch(
    files: IndexedFile[],
    query: string,
    maxResults: number
  ): Promise<SearchResult['results']> {
    const results: SearchResult['results'] = [];
    const queryLower = query.toLowerCase();

    for (const file of files) {
      const matches: Array<{line: number; content: string; score: number}> = [];
      
      const lines = file.content.split('\n');
      lines.forEach((line, index) => {
        if (line.toLowerCase().includes(queryLower)) {
          matches.push({
            line: index + 1,
            content: line.trim(),
            score: 1.0
          });
        }
      });

      if (matches.length > 0) {
        results.push({
          file,
          score: matches.length / lines.length, // Score based on match density
          matches
        });
      }
    }

    return results
      .sort((a, b) => b.score - a.score)
      .slice(0, maxResults);
  }

  /**
   * Semantic search using embeddings
   */
  private async semanticSearch(
    files: IndexedFile[],
    query: string,
    maxResults: number,
    scoreThreshold: number
  ): Promise<SearchResult['results']> {
    // Generate query embedding
    let queryEmbedding: number[];
    
    if (this.rustBridge) {
      queryEmbedding = await this.rustBridge.generateSemanticEmbedding(query);
    } else {
      // Fallback query embedding
      const tokens = query.toLowerCase().split(/\s+/);
      queryEmbedding = new Array(384).fill(0);
      for (let i = 0; i < tokens.length && i < 384; i++) {
        queryEmbedding[i % 384] += this.simpleHash(tokens[i]) / 1000000;
      }
    }

    const results: SearchResult['results'] = [];

    for (const file of files) {
      const fileEmbedding = file.embeddings?.[EngineType.SEMANTIC];
      if (!fileEmbedding) continue;

      const similarity = this.cosineSimilarity(queryEmbedding, fileEmbedding);
      
      if (similarity >= scoreThreshold) {
        results.push({
          file,
          score: similarity,
          matches: [] // Semantic search doesn't provide line-level matches
        });
      }
    }

    return results
      .sort((a, b) => b.score - a.score)
      .slice(0, maxResults);
  }

  /**
   * Hybrid search combining exact and semantic
   */
  private async hybridSearch(
    files: IndexedFile[],
    query: string,
    maxResults: number,
    scoreThreshold: number
  ): Promise<SearchResult['results']> {
    const exactResults = await this.exactSearch(files, query, maxResults * 2);
    const semanticResults = await this.semanticSearch(files, query, maxResults * 2, scoreThreshold * 0.5);

    // Combine and rerank results
    const combinedResults = new Map<string, SearchResult['results'][0]>();

    // Add exact matches with higher weight
    for (const result of exactResults) {
      combinedResults.set(result.file.path, {
        ...result,
        score: result.score * 0.7 // Exact match weight
      });
    }

    // Add or boost semantic matches
    for (const result of semanticResults) {
      const existing = combinedResults.get(result.file.path);
      if (existing) {
        existing.score += result.score * 0.3; // Semantic boost weight
      } else {
        combinedResults.set(result.file.path, {
          ...result,
          score: result.score * 0.3 // Pure semantic weight
        });
      }
    }

    return Array.from(combinedResults.values())
      .filter(result => result.score >= scoreThreshold)
      .sort((a, b) => b.score - a.score)
      .slice(0, maxResults);
  }

  /**
   * Neural search using advanced embeddings
   */
  private async neuralSearch(
    files: IndexedFile[],
    query: string,
    maxResults: number,
    scoreThreshold: number
  ): Promise<SearchResult['results']> {
    // For now, neural search is similar to semantic but with different embeddings
    // In production, this would use specialized neural models
    let queryEmbedding: number[];
    
    if (this.rustBridge) {
      queryEmbedding = await this.rustBridge.generateNeuralEmbedding(query);
    } else {
      // Fallback neural embedding (tanh-transformed semantic)
      const tokens = query.toLowerCase().split(/\s+/);
      const baseEmbedding = new Array(384).fill(0);
      for (let i = 0; i < tokens.length && i < 384; i++) {
        baseEmbedding[i % 384] += this.simpleHash(tokens[i]) / 1000000;
      }
      queryEmbedding = baseEmbedding.map(x => Math.tanh(x));
    }

    const results: SearchResult['results'] = [];

    for (const file of files) {
      const fileEmbedding = file.embeddings?.[EngineType.NEURAL];
      if (!fileEmbedding) continue;

      const similarity = this.cosineSimilarity(queryEmbedding, fileEmbedding);
      
      if (similarity >= scoreThreshold) {
        results.push({
          file,
          score: similarity,
          matches: []
        });
      }
    }

    return results
      .sort((a, b) => b.score - a.score)
      .slice(0, maxResults);
  }

  /**
   * Calculate cosine similarity between two vectors
   */
  private cosineSimilarity(a: number[], b: number[]): number {
    if (a.length !== b.length) return 0;

    let dotProduct = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
      dotProduct += a[i] * b[i];
      normA += a[i] * a[i];
      normB += b[i] * b[i];
    }

    if (normA === 0 || normB === 0) return 0;
    
    return dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
  }

  /**
   * Update specific files in the index
   */
  async updateIndex(filePaths: string[]): Promise<void> {
    const maxFileSize = 1024 * 1024; // 1MB
    
    for (const filePath of filePaths) {
      try {
        await this.indexFile(filePath, maxFileSize);
        console.log('[Engine] Updated index for:', filePath);
      } catch (error) {
        const errorMsg = `Failed to update ${filePath}: ${error instanceof Error ? error.message : String(error)}`;
        this.indexStatus.errors.push(errorMsg);
        console.error('[Engine]', errorMsg);
        throw error;
      }
    }
  }

  /**
   * Get current indexing status
   */
  getStatus(): IndexStatus {
    return { ...this.indexStatus };
  }

  /**
   * Cleanup resources
   */
  async cleanup(): Promise<void> {
    // Close all file watchers
    for (const watcher of this.watchers.values()) {
      await watcher.close();
    }
    this.watchers.clear();

    // Clear indexes
    this.indexedFiles.clear();

    console.log('[Engine] Cleanup completed');
  }
}