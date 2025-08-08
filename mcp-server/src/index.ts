#!/usr/bin/env node

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ErrorCode,
  ListToolsRequestSchema,
  McpError,
} from '@modelcontextprotocol/sdk/types.js';
import { EmbeddingEngine } from './engine.js';
import { 
  indexDirectoryTool,
  parallelSearchTool,
  updateIndexTool,
  getStatusTool,
  type ToolHandler 
} from './tools.js';

/**
 * MCP Server for hybrid embedding search
 * Provides tools for indexing codebases and executing parallel searches across 4 engine types
 */
class EmbeddingMCPServer {
  private server: Server;
  private engine: EmbeddingEngine;
  private toolHandlers: Map<string, ToolHandler> = new Map();

  constructor() {
    this.server = new Server(
      {
        name: 'embedding-search-server',
        version: '1.0.0',
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.engine = new EmbeddingEngine();
    this.setupTools();
    this.setupHandlers();
  }

  /**
   * Register all MCP tools
   */
  private setupTools(): void {
    // Register tool definitions
    this.toolHandlers.set('index_directory', indexDirectoryTool);
    this.toolHandlers.set('parallel_search', parallelSearchTool);
    this.toolHandlers.set('update_index', updateIndexTool);
    this.toolHandlers.set('get_status', getStatusTool);
  }

  /**
   * Setup MCP request handlers
   */
  private setupHandlers(): void {
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      return {
        tools: Array.from(this.toolHandlers.values()).map(handler => handler.definition)
      };
    });

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;
      
      const handler = this.toolHandlers.get(name);
      if (!handler) {
        throw new McpError(
          ErrorCode.MethodNotFound,
          `Tool not found: ${name}`
        );
      }

      try {
        const result = await handler.execute(args || {}, this.engine);
        return {
          content: [
            {
              type: 'text',
              text: typeof result === 'string' ? result : JSON.stringify(result, null, 2)
            }
          ]
        };
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        throw new McpError(
          ErrorCode.InternalError,
          `Tool execution failed: ${errorMessage}`
        );
      }
    });

    // Error handler
    this.server.onerror = (error) => {
      console.error('[MCP Server Error]:', error);
    };
  }

  /**
   * Start the MCP server
   */
  async start(): Promise<void> {
    const transport = new StdioServerTransport();
    
    await this.server.connect(transport);
    console.error('[MCP Server] Embedding search server started');
  }

  /**
   * Cleanup resources
   */
  async cleanup(): Promise<void> {
    await this.engine.cleanup();
    await this.server.close();
  }
}

// Handle process termination
process.on('SIGINT', async () => {
  console.error('[MCP Server] Shutting down...');
  process.exit(0);
});

process.on('SIGTERM', async () => {
  console.error('[MCP Server] Shutting down...');
  process.exit(0);
});

// Start server
if (import.meta.url === `file://${process.argv[1]}`) {
  const server = new EmbeddingMCPServer();
  server.start().catch(console.error);
}