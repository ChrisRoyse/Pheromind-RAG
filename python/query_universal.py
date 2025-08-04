#!/usr/bin/env python3
"""
Universal Code Query System for MCP RAG
Searches indexed code using various methods: semantic, keyword, regex
"""

import argparse
import json
import os
import sys
import sqlite3
import re
import time
import logging
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
import hashlib

# Add current directory to Python path for local imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Version information
VERSION = "1.0.0"

# Try to import sentence transformers for semantic search
try:
    from sentence_transformers import SentenceTransformer
    import numpy as np
    SEMANTIC_SEARCH_AVAILABLE = True
except ImportError:
    SEMANTIC_SEARCH_AVAILABLE = False
    logging.warning("sentence-transformers not available - semantic search disabled")


class QueryResult:
    """Represents a single query result"""
    
    def __init__(self, content: str, file_path: str, start_line: int, 
                 end_line: int, chunk_type: str, name: Optional[str] = None,
                 score: float = 0.0, highlights: Optional[List[Dict]] = None):
        self.content = content
        self.file_path = file_path
        self.start_line = start_line
        self.end_line = end_line
        self.chunk_type = chunk_type
        self.name = name
        self.score = score
        self.highlights = highlights or []
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert result to dictionary for JSON output"""
        return {
            'content': self.content,
            'file_path': self.file_path, 
            'start_line': self.start_line,
            'end_line': self.end_line,
            'chunk_type': self.chunk_type,
            'name': self.name,
            'score': self.score,
            'highlights': self.highlights
        }


class UniversalQueryEngine:
    """Universal query engine for code search"""
    
    def __init__(self, index_path: str, log_level: str = "info"):
        self.index_path = Path(index_path)
        self.db_path = self.index_path / 'code_index.db'
        self.embeddings_path = self.index_path / 'embeddings.npy'
        self.embeddings_meta_path = self.index_path / 'embeddings_meta.json'
        
        # Setup logging
        self.setup_logging(log_level)
        self.logger = logging.getLogger(__name__)
        
        # Initialize components
        self.model = None
        self.embeddings = None
        self.embeddings_meta = None
        
        # Validate index
        self._validate_index()
        
        # Load semantic search components if available
        if SEMANTIC_SEARCH_AVAILABLE:
            self._load_semantic_components()
    
    def setup_logging(self, log_level: str):
        """Setup logging configuration"""
        level_map = {
            'debug': logging.DEBUG,
            'info': logging.INFO, 
            'warning': logging.WARNING,
            'error': logging.ERROR
        }
        
        level = level_map.get(log_level.lower(), logging.INFO)
        logging.basicConfig(level=level, format='%(levelname)s: %(message)s')
    
    def _validate_index(self):
        """Validate that the index exists and is readable"""
        if not self.index_path.exists():
            raise FileNotFoundError(f"Index directory not found: {self.index_path}")
        
        if not self.db_path.exists():
            raise FileNotFoundError(f"Database not found: {self.db_path}")
        
        # Test database connection
        try:
            with sqlite3.connect(self.db_path) as conn:
                cursor = conn.execute("SELECT COUNT(*) FROM code_chunks")
                count = cursor.fetchone()[0]
                self.logger.info(f"Index contains {count} code chunks")
        except Exception as e:
            raise RuntimeError(f"Cannot read database: {e}")
    
    def _load_semantic_components(self):
        """Load semantic search model and embeddings"""
        try:
            # Load or create embedding model
            self.model = SentenceTransformer('all-MiniLM-L6-v2')
            self.logger.info("Loaded sentence transformer model")
            
            # Load embeddings if they exist
            if self.embeddings_path.exists() and self.embeddings_meta_path.exists():
                self.embeddings = np.load(self.embeddings_path)
                with open(self.embeddings_meta_path) as f:
                    self.embeddings_meta = json.load(f)
                self.logger.info(f"Loaded {len(self.embeddings)} cached embeddings")
            else:
                self.logger.info("No cached embeddings found - will generate on demand")
                
        except Exception as e:
            self.logger.warning(f"Failed to load semantic components: {e}")
            self.model = None
    
    def semantic_search(self, query: str, limit: int = 10, 
                       file_type: Optional[str] = None,
                       chunk_type: Optional[str] = None) -> List[QueryResult]:
        """Perform semantic search using embeddings"""
        if not self.model:
            raise RuntimeError("Semantic search not available - missing sentence-transformers")
        
        start_time = time.time()
        
        # Get all chunks from database
        chunks = self._get_chunks(file_type=file_type, chunk_type=chunk_type)
        
        if not chunks:
            return []
        
        # Generate embeddings if not cached
        if self.embeddings is None or self.embeddings_meta is None:
            self._generate_embeddings(chunks)
        
        # Encode query
        query_embedding = self.model.encode([query])
        
        # Calculate similarities
        if len(self.embeddings) != len(chunks):
            # Embeddings don't match current chunks, regenerate
            self._generate_embeddings(chunks)
        
        similarities = np.dot(query_embedding, self.embeddings.T).flatten()
        
        # Get top results
        top_indices = np.argsort(similarities)[::-1][:limit]
        
        results = []
        for idx in top_indices:
            if idx < len(chunks):
                chunk = chunks[idx]
                score = float(similarities[idx])
                
                result = QueryResult(
                    content=chunk['content'],
                    file_path=chunk['file_path'],
                    start_line=chunk['start_line'],
                    end_line=chunk['end_line'],
                    chunk_type=chunk['chunk_type'],
                    name=chunk['name'],
                    score=score
                )
                results.append(result)
        
        query_time = time.time() - start_time
        self.logger.info(f"Semantic search completed in {query_time:.2f}s")
        
        return results
    
    def keyword_search(self, query: str, limit: int = 10,
                      file_type: Optional[str] = None,
                      chunk_type: Optional[str] = None,
                      context: int = 0, highlight: bool = False) -> List[QueryResult]:
        """Perform exact keyword search"""
        start_time = time.time()
        
        # Build SQL query
        sql = "SELECT * FROM code_chunks WHERE content LIKE ?"
        params = [f"%{query}%"]
        
        if file_type:
            sql += " AND file_path LIKE ?"
            params.append(f"%.{file_type}")
        
        if chunk_type:
            sql += " AND chunk_type = ?"
            params.append(chunk_type)
        
        sql += " ORDER BY file_path, start_line LIMIT ?"
        params.append(limit)
        
        results = []
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(sql, params)
            
            for row in cursor:
                content = row['content']
                
                # Add context if requested
                if context > 0:
                    content = self._add_context(row, context)
                
                # Generate highlights if requested
                highlights = []
                if highlight:
                    highlights = self._generate_highlights(content, query)
                
                # Simple scoring based on match frequency
                score = content.lower().count(query.lower()) / len(content.split())
                
                result = QueryResult(
                    content=content,
                    file_path=row['file_path'],
                    start_line=row['start_line'],
                    end_line=row['end_line'],
                    chunk_type=row['chunk_type'],
                    name=row['name'],
                    score=score,
                    highlights=highlights
                )
                results.append(result)
        
        query_time = time.time() - start_time
        self.logger.info(f"Keyword search completed in {query_time:.2f}s")
        
        return results
    
    def regex_search(self, pattern: str, limit: int = 10,
                    file_type: Optional[str] = None,
                    chunk_type: Optional[str] = None) -> List[QueryResult]:
        """Perform regex pattern search"""
        start_time = time.time()
        
        try:
            regex = re.compile(pattern, re.MULTILINE | re.IGNORECASE)
        except re.error as e:
            raise ValueError(f"Invalid regex pattern: {e}")
        
        chunks = self._get_chunks(file_type=file_type, chunk_type=chunk_type)
        results = []
        
        for chunk in chunks:
            matches = list(regex.finditer(chunk['content']))
            if matches:
                # Score based on number of matches
                score = len(matches) / len(chunk['content'].split())
                
                result = QueryResult(
                    content=chunk['content'],
                    file_path=chunk['file_path'],
                    start_line=chunk['start_line'],
                    end_line=chunk['end_line'],
                    chunk_type=chunk['chunk_type'],
                    name=chunk['name'],
                    score=score
                )
                results.append(result)
            
            if len(results) >= limit:
                break
        
        # Sort by score
        results.sort(key=lambda x: x.score, reverse=True)
        
        query_time = time.time() - start_time
        self.logger.info(f"Regex search completed in {query_time:.2f}s")
        
        return results[:limit]
    
    def _get_chunks(self, file_type: Optional[str] = None,
                   chunk_type: Optional[str] = None) -> List[Dict[str, Any]]:
        """Get chunks from database with optional filtering"""
        sql = "SELECT * FROM code_chunks"
        params = []
        conditions = []
        
        if file_type:
            conditions.append("file_path LIKE ?")
            params.append(f"%.{file_type}")
        
        if chunk_type:
            conditions.append("chunk_type = ?")
            params.append(chunk_type)
        
        if conditions:
            sql += " WHERE " + " AND ".join(conditions)
        
        sql += " ORDER BY file_path, start_line"
        
        chunks = []
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(sql, params)
            
            for row in cursor:
                chunks.append({
                    'content': row['content'],
                    'file_path': row['file_path'],
                    'start_line': row['start_line'],
                    'end_line': row['end_line'],
                    'chunk_type': row['chunk_type'],
                    'name': row['name']
                })
        
        return chunks
    
    def _generate_embeddings(self, chunks: List[Dict[str, Any]]):
        """Generate embeddings for chunks"""
        if not self.model:
            return
        
        self.logger.info(f"Generating embeddings for {len(chunks)} chunks...")
        
        # Extract content for embedding
        contents = [chunk['content'] for chunk in chunks]
        
        # Generate embeddings
        self.embeddings = self.model.encode(contents)
        
        # Save embeddings and metadata
        self.embeddings_meta = {
            'chunk_count': len(chunks),
            'model_name': 'all-MiniLM-L6-v2',
            'generated_at': time.time(),
            'chunk_hashes': [hashlib.md5(c.encode()).hexdigest() for c in contents]
        }
        
        # Save to disk
        try:
            np.save(self.embeddings_path, self.embeddings)
            with open(self.embeddings_meta_path, 'w') as f:
                json.dump(self.embeddings_meta, f)
            self.logger.info("Cached embeddings to disk")
        except Exception as e:
            self.logger.warning(f"Failed to cache embeddings: {e}")
    
    def _add_context(self, chunk_row, context_lines: int) -> str:
        """Add context lines around a chunk"""
        # For now, just return the original content
        # In a more sophisticated implementation, we would read the original file
        return chunk_row['content']
    
    def _generate_highlights(self, content: str, query: str) -> List[Dict[str, Any]]:
        """Generate highlight information for matches"""
        highlights = []
        
        # Find all occurrences of the query
        pattern = re.escape(query)
        for match in re.finditer(pattern, content, re.IGNORECASE):
            highlights.append({
                'start': match.start(),
                'end': match.end(),
                'text': match.group()
            })
        
        return highlights


def create_argument_parser():
    """Create command line argument parser"""
    parser = argparse.ArgumentParser(
        description="Universal Code Query System for MCP RAG",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument('--version', action='version', version=f'Universal Query v{VERSION}')
    
    # Query methods (mutually exclusive)
    query_group = parser.add_mutually_exclusive_group(required=True)
    query_group.add_argument('--search', type=str, metavar='QUERY',
                            help='Perform semantic search with natural language query')
    query_group.add_argument('--keyword', type=str, metavar='KEYWORD',
                            help='Perform exact keyword search')
    query_group.add_argument('--regex', type=str, metavar='PATTERN',
                            help='Perform regex pattern search')
    
    # Required index path
    parser.add_argument('--index', type=str, metavar='PATH', required=True,
                       help='Path to the code index directory')
    
    # Filter options
    parser.add_argument('--file-type', type=str, metavar='EXT',
                       help='Filter results by file extension (e.g., py, js)')
    parser.add_argument('--chunk-type', type=str, metavar='TYPE',
                       help='Filter results by chunk type (function, class, etc.)')
    
    # Output options
    parser.add_argument('--limit', type=int, default=10, metavar='N',
                       help='Maximum number of results to return (default: 10)')
    parser.add_argument('--format', choices=['json', 'text'], default='json',
                       help='Output format (default: json)')
    
    # Search enhancements
    parser.add_argument('--context', type=int, default=0, metavar='N',
                       help='Number of context lines to include around matches')
    parser.add_argument('--highlight', action='store_true',
                       help='Highlight matching terms in results')
    
    # Logging
    parser.add_argument('--log-level', choices=['debug', 'info', 'warning', 'error'],
                       default='info', help='Set logging level')
    
    return parser


def format_results_json(results: List[QueryResult], query: str, query_time: float) -> str:
    """Format results as JSON"""
    output = {
        'query': query,
        'total_results': len(results),
        'query_time': query_time,
        'results': [result.to_dict() for result in results]
    }
    return json.dumps(output, indent=2)


def format_results_text(results: List[QueryResult], query: str, query_time: float) -> str:
    """Format results as plain text"""
    lines = []
    lines.append(f"Query: {query}")
    lines.append(f"Results: {len(results)} found in {query_time:.2f}s")
    lines.append("")
    
    for i, result in enumerate(results, 1):
        lines.append(f"{i}. {result.file_path}:{result.start_line}-{result.end_line} (score: {result.score:.3f})")
        if result.name:
            lines.append(f"   {result.chunk_type}: {result.name}")
        
        # Show first few lines of content
        content_lines = result.content.split('\\n')[:3]
        for line in content_lines:
            lines.append(f"   {line.strip()}")
        
        if len(result.content.split('\\n')) > 3:
            lines.append("   ...")
        
        lines.append("")
    
    return '\\n'.join(lines)


def main():
    """Main entry point"""
    parser = create_argument_parser()
    args = parser.parse_args()
    
    try:
        # Validate query
        query = args.search or args.keyword or args.regex
        if not query or not query.strip():
            print("Error: query cannot be empty", file=sys.stderr)
            return 1
        
        # Initialize query engine
        engine = UniversalQueryEngine(args.index, args.log_level)
        
        # Perform search
        start_time = time.time()
        
        if args.search:
            results = engine.semantic_search(
                query, 
                limit=args.limit,
                file_type=args.file_type,
                chunk_type=args.chunk_type
            )
        elif args.keyword:
            results = engine.keyword_search(
                query,
                limit=args.limit,
                file_type=args.file_type,
                chunk_type=args.chunk_type,
                context=args.context,
                highlight=args.highlight
            )
        elif args.regex:
            results = engine.regex_search(
                query,
                limit=args.limit,
                file_type=args.file_type,
                chunk_type=args.chunk_type
            )
        
        query_time = time.time() - start_time
        
        # Format and output results
        if args.format == 'json':
            output = format_results_json(results, query, query_time)
        else:
            output = format_results_text(results, query, query_time)
        
        print(output)
        
    except FileNotFoundError as e:
        print(f"Error: index not found - {e}", file=sys.stderr)
        return 1
    except RuntimeError as e:
        print(f"Error: database error - {e}", file=sys.stderr) 
        return 1
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    
    return 0


if __name__ == '__main__':
    sys.exit(main())