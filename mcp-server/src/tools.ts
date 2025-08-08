import { Tool } from '@modelcontextprotocol/sdk/types.js';
import { EmbeddingEngine, EngineType, ParallelSearchResults, IndexStatus } from './engine.js';

/**
 * Tool handler interface
 */
export interface ToolHandler {
  definition: Tool;
  execute: (args: Record<string, any>, engine: EmbeddingEngine) => Promise<any>;
}

/**
 * Validation helper
 */
function validateRequired(args: Record<string, any>, field: string, type: string): void {
  if (!(field in args)) {
    throw new Error(`Missing required parameter: ${field}`);
  }
  if (typeof args[field] !== type) {
    throw new Error(`Parameter ${field} must be of type ${type}, got ${typeof args[field]}`);
  }
}

/**
 * Tool: Index Directory
 * Indexes a codebase directory with optional file watching
 */
export const indexDirectoryTool: ToolHandler = {
  definition: {
    name: 'index_directory',
    description: 'Index a codebase directory for search with optional file watching',
    inputSchema: {
      type: 'object',
      properties: {
        directory: {
          type: 'string',
          description: 'Path to the directory to index'
        },
        extensions: {
          type: 'array',
          items: {
            type: 'string'
          },
          description: 'File extensions to index (default: .ts, .js, .py, .rs, .cpp, .h, .md, .txt)',
          default: ['.ts', '.js', '.py', '.rs', '.cpp', '.h', '.md', '.txt']
        },
        watchFiles: {
          type: 'boolean',
          description: 'Enable file watching for automatic index updates',
          default: false
        },
        excludePatterns: {
          type: 'array',
          items: {
            type: 'string'
          },
          description: 'Glob patterns for files/directories to exclude',
          default: ['node_modules/**', '.git/**', 'dist/**', 'build/**']
        },
        maxFileSize: {
          type: 'integer',
          description: 'Maximum file size in bytes to index (default: 1MB)',
          default: 1048576
        }
      },
      required: ['directory']
    }
  },

  async execute(args: Record<string, any>, engine: EmbeddingEngine): Promise<string> {
    validateRequired(args, 'directory', 'string');

    const options = {
      extensions: args.extensions || ['.ts', '.js', '.py', '.rs', '.cpp', '.h', '.md', '.txt'],
      watchFiles: args.watchFiles || false,
      excludePatterns: args.excludePatterns || ['node_modules/**', '.git/**', 'dist/**', 'build/**'],
      maxFileSize: args.maxFileSize || 1048576
    };

    // Validate arrays
    if (!Array.isArray(options.extensions)) {
      throw new Error('extensions must be an array of strings');
    }
    if (!Array.isArray(options.excludePatterns)) {
      throw new Error('excludePatterns must be an array of strings');
    }

    const startTime = Date.now();
    
    try {
      await engine.indexDirectory(args.directory, options);
      const endTime = Date.now();
      const status = engine.getStatus();
      
      const result = {
        success: true,
        directory: args.directory,
        filesIndexed: status.filesIndexed,
        totalFiles: status.totalFiles,
        executionTime: endTime - startTime,
        watchingEnabled: options.watchFiles,
        errors: status.errors
      };

      return `Directory indexing completed successfully.\n\n${JSON.stringify(result, null, 2)}`;
    } catch (error) {
      const errorResult = {
        success: false,
        directory: args.directory,
        error: error instanceof Error ? error.message : String(error),
        executionTime: Date.now() - startTime
      };
      
      return `Directory indexing failed.\n\n${JSON.stringify(errorResult, null, 2)}`;
    }
  }
};

/**
 * Tool: Parallel Search
 * Execute search across all 4 engines in parallel
 */
export const parallelSearchTool: ToolHandler = {
  definition: {
    name: 'parallel_search',
    description: 'Execute parallel search across all 4 embedding engines (exact, semantic, hybrid, neural)',
    inputSchema: {
      type: 'object',
      properties: {
        query: {
          type: 'string',
          description: 'Search query text'
        },
        maxResults: {
          type: 'integer',
          description: 'Maximum number of results per engine',
          default: 10,
          minimum: 1,
          maximum: 100
        },
        scoreThreshold: {
          type: 'number',
          description: 'Minimum similarity score for results',
          default: 0.1,
          minimum: 0,
          maximum: 1
        },
        engines: {
          type: 'array',
          items: {
            type: 'string',
            enum: ['exact', 'semantic', 'hybrid', 'neural']
          },
          description: 'Specific engines to use (default: all engines)',
          default: ['exact', 'semantic', 'hybrid', 'neural']
        }
      },
      required: ['query']
    }
  },

  async execute(args: Record<string, any>, engine: EmbeddingEngine): Promise<ParallelSearchResults> {
    validateRequired(args, 'query', 'string');

    const options = {
      maxResults: args.maxResults || 10,
      scoreThreshold: args.scoreThreshold || 0.1,
      engines: args.engines || [EngineType.EXACT, EngineType.SEMANTIC, EngineType.HYBRID, EngineType.NEURAL]
    };

    // Validate engines array
    if (!Array.isArray(options.engines)) {
      throw new Error('engines must be an array');
    }

    const validEngines = Object.values(EngineType);
    for (const engineType of options.engines) {
      if (!validEngines.includes(engineType as EngineType)) {
        throw new Error(`Invalid engine type: ${engineType}. Valid types: ${validEngines.join(', ')}`);
      }
    }

    // Validate numeric parameters
    if (options.maxResults < 1 || options.maxResults > 100) {
      throw new Error('maxResults must be between 1 and 100');
    }
    if (options.scoreThreshold < 0 || options.scoreThreshold > 1) {
      throw new Error('scoreThreshold must be between 0 and 1');
    }

    try {
      const results = await engine.parallelSearch(args.query, options);
      return results;
    } catch (error) {
      throw new Error(`Parallel search failed: ${error instanceof Error ? error.message : String(error)}`);
    }
  }
};

/**
 * Tool: Update Index
 * Update specific files in the index
 */
export const updateIndexTool: ToolHandler = {
  definition: {
    name: 'update_index',
    description: 'Update specific files in the search index',
    inputSchema: {
      type: 'object',
      properties: {
        filePaths: {
          type: 'array',
          items: {
            type: 'string'
          },
          description: 'Array of file paths to update in the index'
        }
      },
      required: ['filePaths']
    }
  },

  async execute(args: Record<string, any>, engine: EmbeddingEngine): Promise<string> {
    if (!('filePaths' in args)) {
      throw new Error('Missing required parameter: filePaths');
    }
    if (!Array.isArray(args.filePaths)) {
      throw new Error('filePaths must be an array of strings');
    }
    if (args.filePaths.length === 0) {
      throw new Error('filePaths array cannot be empty');
    }

    const startTime = Date.now();
    const results = {
      success: true,
      updatedFiles: [] as string[],
      errors: [] as string[]
    };

    for (const filePath of args.filePaths) {
      if (typeof filePath !== 'string') {
        results.errors.push(`Invalid file path type: ${typeof filePath}`);
        continue;
      }

      try {
        await engine.updateIndex([filePath]);
        results.updatedFiles.push(filePath);
      } catch (error) {
        const errorMsg = `Failed to update ${filePath}: ${error instanceof Error ? error.message : String(error)}`;
        results.errors.push(errorMsg);
      }
    }

    results.success = results.errors.length === 0;
    const executionTime = Date.now() - startTime;

    const response = {
      ...results,
      totalFiles: args.filePaths.length,
      executionTime
    };

    return `Index update ${results.success ? 'completed successfully' : 'completed with errors'}.\n\n${JSON.stringify(response, null, 2)}`;
  }
};

/**
 * Tool: Get Status  
 * Get current indexing status and system information
 */
export const getStatusTool: ToolHandler = {
  definition: {
    name: 'get_status',
    description: 'Get current indexing status and system information',
    inputSchema: {
      type: 'object',
      properties: {
        includeDetails: {
          type: 'boolean',
          description: 'Include detailed file information in the response',
          default: false
        }
      }
    }
  },

  async execute(args: Record<string, any>, engine: EmbeddingEngine): Promise<IndexStatus & { systemInfo?: any }> {
    const status = engine.getStatus();
    const includeDetails = args.includeDetails || false;

    let response: IndexStatus & { systemInfo?: any } = {
      ...status
    };

    if (includeDetails) {
      response.systemInfo = {
        nodeVersion: process.version,
        platform: process.platform,
        arch: process.arch,
        memoryUsage: process.memoryUsage(),
        uptime: process.uptime()
      };
    }

    // Add progress information if indexing
    if (status.isIndexing && status.startTime) {
      const elapsed = Date.now() - status.startTime;
      const progress = status.totalFiles > 0 ? status.filesIndexed / status.totalFiles : 0;
      
      if (progress > 0) {
        const estimatedTotal = elapsed / progress;
        response.estimatedCompletion = status.startTime + estimatedTotal;
      }
    }

    return response;
  }
};

/**
 * Export all tools as an array for easy registration
 */
export const allTools: ToolHandler[] = [
  indexDirectoryTool,
  parallelSearchTool,
  updateIndexTool,
  getStatusTool
];