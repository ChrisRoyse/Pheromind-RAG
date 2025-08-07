# Embed Search System - Project Overview

## Purpose
A production-ready, high-performance embedding search system for code repositories using state-of-the-art semantic search with real AI embeddings. Designed for MCP (Model Context Protocol) integration with LLMs.

## Tech Stack
- **Primary Language**: Rust
- **Build System**: Cargo (Rust package manager)
- **Node.js Dependencies**: 
  - better-sqlite3 (database operations)
  - claude-flow (SPARC methodology and swarm orchestration)
  - sqlite3 (SQLite integration)

## Key Features
- 3-Chunk Context: Returns code with surrounding context (above + target + below chunks)
- Single Embedding Model: Uses all-MiniLM-L6-v2 for semantic search
- Git-Based Updates: Monitors file changes via git status for incremental updates
- MCP Integration: Full Model Context Protocol server for LLM integration
- Multiple search backends: BM25, Tantivy, and ML embeddings with LanceDB

## Performance Targets
- Search Accuracy: 85% success rate (finds relevant result in top 5)
- Search Latency: <500ms including embedding generation
- Memory Usage: <2GB (single model)
- Startup Time: <30 seconds
- Index Updates: <1s to detect changes via git

## Development Status
Currently in active development with focus on:
- Regex-based code chunker with multi-language support
- Three-chunk context expander
- MiniLM embeddings integration
- LanceDB vector storage
- Tantivy full-text search with fuzzy matching