#!/usr/bin/env python3
"""
MCP RAG Indexer Server
A Model Context Protocol server for RAG-based code indexing and search

MCP 2025 Features:
- Tools: index_project, search_code, list_projects
- Resources: project files, chunks, and search results
- Prompts: code_review, documentation, refactor_suggestions, similar_code, api_usage

Example Prompt Usage:
- code_review: "Review this function for bugs and improvements"
- documentation: "Generate docs for this class based on similar code"
- api_usage: "Show me how requests.get is used in this project"
"""

import asyncio
import argparse
import json
import logging
import sys
import os
import sqlite3
from typing import Any, Dict, List, Optional
from pathlib import Path
from urllib.parse import urlparse, parse_qs

# Add current directory to Python path for local imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Import MCP SDK (will gracefully fail if not available)
try:
    from mcp import server, types
    from mcp.server import stdio
    from mcp.server.stdio import stdio_server
    from mcp.types import TextContent, Tool, Resource, Prompt, PromptMessage, PromptArgument, GetPromptResult
    MCP_AVAILABLE = True
except ImportError as e:
    logging.warning(f"MCP SDK not available: {e}")
    MCP_AVAILABLE = False

# Import local modules (will create minimal versions if they don't exist)
try:
    from model_manager import get_model_manager, load_embedding_model
except ImportError:
    logging.warning("model_manager not available, using mock")
    def get_model_manager():
        return None
    def load_embedding_model():
        return None

# Import indexer and query components
try:
    from indexer_universal import UniversalIndexer
    from query_universal import UniversalQueryEngine
except ImportError:
    logging.warning("indexer_universal or query_universal not available, using mock")
    UniversalIndexer = None
    UniversalQueryEngine = None

# Version information
VERSION = "1.0.0"
DESCRIPTION = "MCP RAG Indexer - Universal code indexing and search"


class RAGIndexerServer:
    """Main MCP RAG Indexer Server"""
    
    def __init__(self, log_level: str = "info"):
        self.log_level = log_level
        self.setup_logging()
        self.logger = logging.getLogger(__name__)
        
        # Initialize components
        self.model_manager = None
        self.embedding_model = None
        self.indexed_projects = {}
        self.server_app = None
        
        self.logger.info(f"Initializing MCP RAG Indexer Server v{VERSION}")
        
        # Initialize model manager if available
        try:
            self.model_manager = get_model_manager()
            self.logger.info("Model manager initialized")
        except Exception as e:
            self.logger.warning(f"Failed to initialize model manager: {e}")
    
    def setup_logging(self):
        """Setup logging configuration"""
        level_map = {
            'debug': logging.DEBUG,
            'info': logging.INFO,
            'warning': logging.WARNING,
            'error': logging.ERROR
        }
        
        level = level_map.get(self.log_level.lower(), logging.INFO)
        
        logging.basicConfig(
            level=level,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.StreamHandler(sys.stderr)
            ]
        )
    
    def get_tools(self) -> List[Tool]:
        """Return available MCP tools"""
        return [
            Tool(
                name="index_project",
                description="Index a code project for RAG search",
                inputSchema={
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the project directory to index"
                        },
                        "name": {
                            "type": "string", 
                            "description": "Optional name for the project"
                        }
                    },
                    "required": ["path"]
                }
            ),
            Tool(
                name="search_code",
                description="Search indexed code using natural language",
                inputSchema={
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language search query"
                        },
                        "project": {
                            "type": "string",
                            "description": "Optional project name to search within"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return",
                            "default": 10
                        }
                    },
                    "required": ["query"]
                }
            ),
            Tool(
                name="list_projects",
                description="List all indexed projects",
                inputSchema={
                    "type": "object",
                    "properties": {}
                }
            )
        ]
    
    def get_resources(self) -> List[Resource]:
        """Return available MCP resources"""
        resources = []
        
        # Add base resources for each indexed project
        for project_name, project_info in self.indexed_projects.items():
            base_uri = f"mcp-rag://project/{project_name}"
            
            resources.extend([
                Resource(
                    uri=f"{base_uri}/files",
                    name=f"Files in {project_name}",
                    description=f"List all indexed files in project {project_name}",
                    mimeType="application/json"
                ),
                Resource(
                    uri=f"{base_uri}/chunks",
                    name=f"Code chunks in {project_name}",
                    description=f"List all code chunks in project {project_name}",
                    mimeType="application/json"
                ),
                Resource(
                    uri=f"{base_uri}/search",
                    name=f"Search in {project_name}",
                    description=f"Search code in project {project_name} (use ?q=query parameter)",
                    mimeType="application/json"
                )
            ])
        
        return resources
    
    def get_prompts(self) -> List[Prompt]:
        """Return available MCP prompts"""
        return [
            Prompt(
                name="code_review",
                description="Review code using indexed codebase context to identify issues and suggest improvements",
                arguments=[
                    PromptArgument(
                        name="code",
                        description="The code to review",
                        required=True
                    ),
                    PromptArgument(
                        name="project",
                        description="Optional project name to search for context",
                        required=False
                    )
                ]
            ),
            Prompt(
                name="documentation",
                description="Generate documentation for code using examples from the indexed codebase",
                arguments=[
                    PromptArgument(
                        name="code",
                        description="The code to document",
                        required=True
                    ),
                    PromptArgument(
                        name="project",
                        description="Optional project name to search for similar functions",
                        required=False
                    )
                ]
            ),
            Prompt(
                name="refactor_suggestions",
                description="Suggest refactoring improvements using patterns from the indexed codebase",
                arguments=[
                    PromptArgument(
                        name="code",
                        description="The code to refactor",
                        required=True
                    ),
                    PromptArgument(
                        name="project",
                        description="Optional project name to search for patterns",
                        required=False
                    )
                ]
            ),
            Prompt(
                name="similar_code",
                description="Find similar code patterns and implementations in the indexed codebase",
                arguments=[
                    PromptArgument(
                        name="description",
                        description="Description of the code pattern to find",
                        required=True
                    ),
                    PromptArgument(
                        name="project",
                        description="Optional project name to search within",
                        required=False
                    )
                ]
            ),
            Prompt(
                name="api_usage",
                description="Find examples of how APIs or functions are used in the indexed codebase",
                arguments=[
                    PromptArgument(
                        name="api_name",
                        description="Name of the API, function, or class to find usage examples for",
                        required=True
                    ),
                    PromptArgument(
                        name="project",
                        description="Optional project name to search within",
                        required=False
                    )
                ]
            )
        ]
    
    def _get_project_index_path(self, project_name: str) -> Optional[Path]:
        """Get the index path for a project"""
        if project_name not in self.indexed_projects:
            return None
        
        project_info = self.indexed_projects[project_name]
        project_path = Path(project_info['path'])
        return project_path / '.mcp_index'
    
    def _get_project_files(self, project_name: str) -> List[Dict[str, Any]]:
        """Get list of all indexed files for a project"""
        index_path = self._get_project_index_path(project_name)
        if not index_path or not (index_path / 'code_index.db').exists():
            return []
        
        files = []
        try:
            db_path = index_path / 'code_index.db'
            with sqlite3.connect(db_path) as conn:
                conn.row_factory = sqlite3.Row
                cursor = conn.execute('''
                    SELECT DISTINCT file_path, COUNT(*) as chunk_count,
                           MAX(created_at) as last_indexed
                    FROM code_chunks 
                    GROUP BY file_path
                    ORDER BY file_path
                ''')
                
                for row in cursor:
                    files.append({
                        'file_path': row['file_path'],
                        'chunk_count': row['chunk_count'],
                        'last_indexed': row['last_indexed']
                    })
        except Exception as e:
            self.logger.error(f"Error getting files for project {project_name}: {e}")
        
        return files
    
    def _get_project_chunks(self, project_name: str, limit: int = 100) -> List[Dict[str, Any]]:
        """Get list of all code chunks for a project"""
        index_path = self._get_project_index_path(project_name)
        if not index_path or not (index_path / 'code_index.db').exists():
            return []
        
        chunks = []
        try:
            db_path = index_path / 'code_index.db'
            with sqlite3.connect(db_path) as conn:
                conn.row_factory = sqlite3.Row
                cursor = conn.execute('''
                    SELECT file_path, start_line, end_line, chunk_type, name, content
                    FROM code_chunks 
                    ORDER BY file_path, start_line
                    LIMIT ?
                ''', (limit,))
                
                for row in cursor:
                    chunks.append({
                        'file_path': row['file_path'],
                        'start_line': row['start_line'],
                        'end_line': row['end_line'],
                        'chunk_type': row['chunk_type'],
                        'name': row['name'],
                        'content': row['content'][:500] + ('...' if len(row['content']) > 500 else '')  # Truncate for listing
                    })
        except Exception as e:
            self.logger.error(f"Error getting chunks for project {project_name}: {e}")
        
        return chunks
    
    def _get_file_content(self, project_name: str, file_path: str) -> Optional[str]:
        """Get content of a specific file"""
        index_path = self._get_project_index_path(project_name)
        if not index_path or not (index_path / 'code_index.db').exists():
            return None
        
        try:
            db_path = index_path / 'code_index.db'
            with sqlite3.connect(db_path) as conn:
                conn.row_factory = sqlite3.Row
                cursor = conn.execute('''
                    SELECT content, start_line, end_line, chunk_type, name
                    FROM code_chunks 
                    WHERE file_path = ?
                    ORDER BY start_line
                ''', (file_path,))
                
                chunks = []
                for row in cursor:
                    chunks.append({
                        'content': row['content'],
                        'start_line': row['start_line'],
                        'end_line': row['end_line'],
                        'chunk_type': row['chunk_type'],
                        'name': row['name']
                    })
                
                if chunks:
                    return json.dumps({
                        'file_path': file_path,
                        'chunks': chunks
                    }, indent=2)
                    
        except Exception as e:
            self.logger.error(f"Error getting file content for {file_path}: {e}")
        
        return None
    
    def _search_project(self, project_name: str, query: str, limit: int = 10) -> List[Dict[str, Any]]:
        """Search within a specific project"""
        index_path = self._get_project_index_path(project_name)
        if not index_path or not (index_path / 'code_index.db').exists():
            return []
        
        results = []
        try:
            # Simple keyword search in database
            db_path = index_path / 'code_index.db'
            with sqlite3.connect(db_path) as conn:
                conn.row_factory = sqlite3.Row
                cursor = conn.execute('''
                    SELECT file_path, start_line, end_line, chunk_type, name, content
                    FROM code_chunks 
                    WHERE content LIKE ?
                    ORDER BY file_path, start_line
                    LIMIT ?
                ''', (f'%{query}%', limit))
                
                for row in cursor:
                    # Simple scoring based on query frequency
                    content = row['content']
                    score = content.lower().count(query.lower()) / len(content.split())
                    
                    results.append({
                        'file_path': row['file_path'],
                        'start_line': row['start_line'],
                        'end_line': row['end_line'],
                        'chunk_type': row['chunk_type'],
                        'name': row['name'],
                        'content': content,
                        'score': score
                    })
                
        except Exception as e:
            self.logger.error(f"Error searching project {project_name}: {e}")
        
        return results
    
    async def get_prompt(self, name: str, arguments: Dict[str, Any]) -> GetPromptResult:
        """Generate prompt content based on template and arguments"""
        try:
            if name == "code_review":
                code = arguments.get("code", "")
                project = arguments.get("project")
                
                # Search for related code patterns if project is specified
                context = ""
                if project and project in self.indexed_projects:
                    # Search for similar patterns
                    search_results = self._search_project(project, code[:100], limit=3)
                    if search_results:
                        context = "\n\nRelated code patterns found in the codebase:\n"
                        for i, result in enumerate(search_results, 1):
                            context += f"\n{i}. {result['file_path']}:{result['start_line']}-{result['end_line']}\n"
                            context += f"   {result['content'][:200]}...\n"
                
                prompt_text = f"""Review this code for potential issues, improvements, and consistency with existing patterns:

```
{code}
```
{context}

Please provide:
1. Code quality assessment
2. Potential bugs or issues
3. Performance improvements
4. Style and best practices recommendations
5. Consistency with patterns found in the codebase"""

                return GetPromptResult(
                    description="Code review with codebase context",
                    messages=[
                        PromptMessage(
                            role="user",
                            content=TextContent(type="text", text=prompt_text)
                        )
                    ]
                )
            
            elif name == "documentation":
                code = arguments.get("code", "")
                project = arguments.get("project")
                
                # Find similar functions for examples
                context = ""
                if project and project in self.indexed_projects:
                    search_results = self._search_project(project, "function def class", limit=3)
                    if search_results:
                        context = "\n\nSimilar documented functions in the codebase:\n"
                        for i, result in enumerate(search_results, 1):
                            context += f"\n{i}. {result['file_path']}:{result['start_line']}-{result['end_line']}\n"
                            context += f"   {result['content'][:200]}...\n"
                
                prompt_text = f"""Generate comprehensive documentation for this code:

```
{code}
```
{context}

Please provide:
1. Clear description of what the code does
2. Parameter descriptions (if applicable)
3. Return value description (if applicable)
4. Usage examples
5. Any important notes or warnings
6. Follow the documentation style used in similar functions shown above"""

                return GetPromptResult(
                    description="Documentation generation with codebase examples",
                    messages=[
                        PromptMessage(
                            role="user",
                            content=TextContent(type="text", text=prompt_text)
                        )
                    ]
                )
            
            elif name == "refactor_suggestions":
                code = arguments.get("code", "")
                project = arguments.get("project")
                
                # Find refactoring patterns in the codebase
                context = ""
                if project and project in self.indexed_projects:
                    search_results = self._search_project(project, "refactor improve optimize", limit=3)
                    if search_results:
                        context = "\n\nRefactoring patterns found in the codebase:\n"
                        for i, result in enumerate(search_results, 1):
                            context += f"\n{i}. {result['file_path']}:{result['start_line']}-{result['end_line']}\n"
                            context += f"   {result['content'][:200]}...\n"
                
                prompt_text = f"""Suggest refactoring improvements for this code:

```
{code}
```
{context}

Please provide:
1. Code structure improvements
2. Performance optimizations
3. Readability enhancements
4. Design pattern suggestions
5. Code duplication elimination
6. Consider the refactoring patterns used in the codebase shown above"""

                return GetPromptResult(
                    description="Refactoring suggestions with codebase patterns",
                    messages=[
                        PromptMessage(
                            role="user",
                            content=TextContent(type="text", text=prompt_text)
                        )
                    ]
                )
            
            elif name == "similar_code":
                description = arguments.get("description", "")
                project = arguments.get("project")
                
                # Search for similar code patterns
                context = ""
                if project and project in self.indexed_projects:
                    search_results = self._search_project(project, description, limit=5)
                    if search_results:
                        context = "\n\nSimilar code patterns found:\n"
                        for i, result in enumerate(search_results, 1):
                            context += f"\n{i}. {result['file_path']}:{result['start_line']}-{result['end_line']} (score: {result.get('score', 0):.3f})\n"
                            context += f"   Type: {result.get('chunk_type', 'unknown')}\n"
                            if result.get('name'):
                                context += f"   Name: {result['name']}\n"
                            context += f"   Code:\n   {result['content'][:300]}...\n"
                    else:
                        context = f"\n\nNo similar patterns found for '{description}' in the indexed codebase."
                
                all_results = []
                if not project:
                    # Search across all projects
                    for project_name in self.indexed_projects.keys():
                        project_results = self._search_project(project_name, description, limit=2)
                        for result in project_results:
                            result['project'] = project_name
                        all_results.extend(project_results)
                    
                    if all_results:
                        all_results.sort(key=lambda x: x.get('score', 0), reverse=True)
                        context = "\n\nSimilar code patterns found across projects:\n"
                        for i, result in enumerate(all_results[:5], 1):
                            context += f"\n{i}. {result['file_path']}:{result['start_line']}-{result['end_line']} (project: {result['project']}, score: {result.get('score', 0):.3f})\n"
                            context += f"   Type: {result.get('chunk_type', 'unknown')}\n"
                            if result.get('name'):
                                context += f"   Name: {result['name']}\n"
                            context += f"   Code:\n   {result['content'][:300]}...\n"
                
                prompt_text = f"""Find similar code patterns for: "{description}"
{context}

Based on the search results above, please:
1. Analyze the patterns found
2. Identify common implementation approaches
3. Highlight differences between implementations
4. Suggest best practices based on these examples
5. Recommend which approach might be most suitable for different scenarios"""

                return GetPromptResult(
                    description="Similar code pattern analysis",
                    messages=[
                        PromptMessage(
                            role="user",
                            content=TextContent(type="text", text=prompt_text)
                        )
                    ]
                )
            
            elif name == "api_usage":
                api_name = arguments.get("api_name", "")
                project = arguments.get("project")
                
                # Search for API usage examples
                context = ""
                if project and project in self.indexed_projects:
                    search_results = self._search_project(project, api_name, limit=5)
                    if search_results:
                        context = f"\n\nUsage examples of '{api_name}' found in {project}:\n"
                        for i, result in enumerate(search_results, 1):
                            context += f"\n{i}. {result['file_path']}:{result['start_line']}-{result['end_line']}\n"
                            context += f"   {result['content'][:300]}...\n"
                    else:
                        context = f"\n\nNo usage examples found for '{api_name}' in project '{project}'."
                
                all_results = []
                if not project:
                    # Search across all projects
                    for project_name in self.indexed_projects.keys():
                        project_results = self._search_project(project_name, api_name, limit=3)
                        for result in project_results:
                            result['project'] = project_name
                        all_results.extend(project_results)
                    
                    if all_results:
                        all_results.sort(key=lambda x: x.get('score', 0), reverse=True)
                        context = f"\n\nUsage examples of '{api_name}' found across projects:\n"
                        for i, result in enumerate(all_results[:5], 1):
                            context += f"\n{i}. {result['file_path']}:{result['start_line']}-{result['end_line']} (project: {result['project']})\n"
                            context += f"   {result['content'][:300]}...\n"
                
                prompt_text = f"""Show examples and usage patterns for API: "{api_name}"
{context}

Based on the usage examples found, please provide:
1. Common usage patterns
2. Parameter patterns and typical values
3. Return value handling
4. Error handling approaches
5. Best practices based on these examples
6. Integration patterns with other APIs or functions"""

                return GetPromptResult(
                    description="API usage examples and patterns",
                    messages=[
                        PromptMessage(
                            role="user",
                            content=TextContent(type="text", text=prompt_text)
                        )
                    ]
                )
            
            else:
                raise ValueError(f"Unknown prompt: {name}")
                
        except Exception as e:
            self.logger.error(f"Error generating prompt {name}: {e}")
            raise ValueError(f"Failed to generate prompt: {str(e)}")
    
    async def read_resource(self, uri: str) -> str:
        """Read and return resource content based on URI"""
        try:
            parsed_uri = urlparse(uri)
            
            if parsed_uri.scheme != "mcp-rag":
                raise ValueError(f"Unsupported URI scheme: {parsed_uri.scheme}")
            
            # For mcp-rag://project/name/type/..., urlparse treats "project" as netloc
            # We need to handle this specially
            if parsed_uri.netloc != "project":
                raise ValueError(f"Expected netloc 'project', got: {parsed_uri.netloc}")
            
            # Parse path: /{project_name}/{resource_type}/{optional_params}
            path_parts = [p for p in parsed_uri.path.split('/') if p]
            
            if len(path_parts) < 2:
                raise ValueError(f"Invalid resource path: {parsed_uri.path} - expected /{{name}}/{{type}}, got parts: {path_parts}")
            
            project_name = path_parts[0]
            resource_type = path_parts[1]
            
            if project_name not in self.indexed_projects:
                raise ValueError(f"Project not found: {project_name}")
            
            # Handle different resource types
            if resource_type == "files":
                files = self._get_project_files(project_name)
                return json.dumps({
                    'project': project_name,
                    'type': 'files',
                    'count': len(files),
                    'files': files
                }, indent=2)
                
            elif resource_type == "chunks":
                chunks = self._get_project_chunks(project_name)
                return json.dumps({
                    'project': project_name, 
                    'type': 'chunks',
                    'count': len(chunks),
                    'chunks': chunks
                }, indent=2)
                
            elif resource_type == "file" and len(path_parts) > 2:
                # Get specific file content: /{name}/file/{file_path}
                file_path = '/'.join(path_parts[2:])
                content = self._get_file_content(project_name, file_path)
                if content:
                    return content
                else:
                    raise ValueError(f"File not found: {file_path}")
                    
            elif resource_type == "search":
                # Parse query parameter
                query_params = parse_qs(parsed_uri.query)
                query = query_params.get('q', [''])[0]
                
                if not query.strip():
                    raise ValueError("Search query parameter 'q' is required")
                
                limit = int(query_params.get('limit', ['10'])[0])
                results = self._search_project(project_name, query, limit)
                
                return json.dumps({
                    'project': project_name,
                    'type': 'search',
                    'query': query,
                    'count': len(results),
                    'results': results
                }, indent=2)
                
            else:
                raise ValueError(f"Unknown resource type: {resource_type}")
                
        except Exception as e:
            self.logger.error(f"Error reading resource {uri}: {e}")
            raise ValueError(f"Failed to read resource: {str(e)}")
    
    async def handle_index_project(self, arguments: Dict[str, Any]) -> List[TextContent]:
        """Handle project indexing request"""
        path = arguments.get("path")
        name = arguments.get("name", Path(path).name if path else "unknown")
        
        if not path:
            return [TextContent(
                type="text",
                text="Error: path parameter is required"
            )]
        
        try:
            project_path = Path(path)
            if not project_path.exists():
                return [TextContent(
                    type="text",
                    text=f"Error: Path does not exist: {path}"
                )]
            
            if not project_path.is_dir():
                return [TextContent(
                    type="text", 
                    text=f"Error: Path is not a directory: {path}"
                )]
            
            # Create index directory
            index_path = project_path / '.mcp_index'
            index_path.mkdir(exist_ok=True)
            
            # Perform actual indexing if UniversalIndexer is available
            if UniversalIndexer:
                try:
                    indexer = UniversalIndexer()
                    result = indexer.index_project(
                        str(project_path), 
                        str(index_path),
                        incremental=True,
                        generate_embeddings=False
                    )
                    
                    # Store project info
                    self.indexed_projects[name] = {
                        "path": str(project_path.absolute()),
                        "indexed_at": str(Path(index_path / 'code_index.db').stat().st_mtime),
                        "file_count": result.get('indexed_files', 0),
                        "chunk_count": result.get('total_chunks', 0),
                        "status": "indexed"
                    }
                    
                    self.logger.info(f"Indexed project '{name}' at {path}")
                    
                    return [TextContent(
                        type="text",
                        text=f"Successfully indexed project '{name}' at {path}\n"
                             f"Indexed {result.get('indexed_files', 0)} files\n"
                             f"Generated {result.get('total_chunks', 0)} code chunks\n"
                             f"Index stored at {index_path}"
                    )]
                    
                except Exception as indexing_error:
                    self.logger.error(f"Indexing failed: {indexing_error}")
                    return [TextContent(
                        type="text",
                        text=f"Error during indexing: {str(indexing_error)}"
                    )]
            else:
                # Fallback: just record the project without actual indexing
                file_count = len(list(project_path.rglob("*.*")))
                self.indexed_projects[name] = {
                    "path": str(project_path.absolute()),
                    "indexed_at": "mock",
                    "file_count": file_count,
                    "chunk_count": 0,
                    "status": "mock_indexed"
                }
                
                return [TextContent(
                    type="text",
                    text=f"Mock indexed project '{name}' at {path}\n"
                         f"Found {file_count} files\n"
                         f"Note: UniversalIndexer not available - using mock indexing"
                )]
            
        except Exception as e:
            self.logger.error(f"Error indexing project: {e}")
            return [TextContent(
                type="text",
                text=f"Error indexing project: {str(e)}"
            )]
    
    async def handle_search_code(self, arguments: Dict[str, Any]) -> List[TextContent]:
        """Handle code search request"""
        query = arguments.get("query")
        project = arguments.get("project")
        limit = arguments.get("limit", 10)
        
        if not query:
            return [TextContent(
                type="text",
                text="Error: query parameter is required"
            )]
        
        try:
            if not self.indexed_projects:
                return [TextContent(
                    type="text",
                    text="No projects have been indexed yet. Use 'index_project' first."
                )]
            
            # Search in specified project or all projects
            all_results = []
            
            projects_to_search = [project] if project and project in self.indexed_projects else list(self.indexed_projects.keys())
            
            for project_name in projects_to_search:
                project_results = self._search_project(project_name, query, limit)
                for result in project_results:
                    result['project'] = project_name
                all_results.extend(project_results)
            
            # Sort by score and limit
            all_results.sort(key=lambda x: x.get('score', 0), reverse=True)
            final_results = all_results[:limit]
            
            if not final_results:
                return [TextContent(
                    type="text",
                    text=f"No results found for query: '{query}'"
                )]
            
            # Format results
            results_text = f"Search results for: '{query}'\n"
            results_text += f"Found {len(final_results)} results\n\n"
            
            for i, result in enumerate(final_results, 1):
                results_text += f"{i}. {result['file_path']}:{result['start_line']}-{result['end_line']}"
                if 'project' in result:
                    results_text += f" (project: {result['project']})"
                results_text += f" (score: {result.get('score', 0):.3f})\n"
                
                if result.get('name'):
                    results_text += f"   {result['chunk_type']}: {result['name']}\n"
                
                # Show first few lines of content
                content_lines = result['content'].split('\n')[:2]
                for line in content_lines:
                    if line.strip():
                        results_text += f"   {line.strip()}\n"
                
                if len(result['content'].split('\n')) > 2:
                    results_text += "   ...\n"
                results_text += "\n"
            
            return [TextContent(
                type="text", 
                text=results_text
            )]
            
        except Exception as e:
            self.logger.error(f"Error searching code: {e}")
            return [TextContent(
                type="text",
                text=f"Error searching code: {str(e)}"
            )]
    
    async def handle_list_projects(self, arguments: Dict[str, Any]) -> List[TextContent]:
        """Handle list projects request"""
        try:
            if not self.indexed_projects:
                return [TextContent(
                    type="text",
                    text="No projects have been indexed yet."
                )]
            
            projects_text = "Indexed Projects:\n\n"
            for name, info in self.indexed_projects.items():
                projects_text += f"• {name}\n"
                projects_text += f"  Path: {info['path']}\n"
                projects_text += f"  Files: {info['file_count']}\n"
                projects_text += f"  Status: {info['status']}\n\n"
            
            return [TextContent(
                type="text",
                text=projects_text
            )]
            
        except Exception as e:
            self.logger.error(f"Error listing projects: {e}")
            return [TextContent(
                type="text",
                text=f"Error listing projects: {str(e)}"
            )]
    
    def create_server(self):
        """Create and configure the MCP server"""
        if not MCP_AVAILABLE:
            raise RuntimeError("MCP SDK is not available")
        
        # Create server instance
        self.server_app = server.Server("mcp-rag-indexer")
        
        # Register tool handlers
        @self.server_app.call_tool()
        async def call_tool(name: str, arguments: dict) -> list[types.TextContent]:
            """Handle tool calls"""
            try:
                if name == "index_project":
                    return await self.handle_index_project(arguments)
                elif name == "search_code":
                    return await self.handle_search_code(arguments)
                elif name == "list_projects":
                    return await self.handle_list_projects(arguments)
                else:
                    return [TextContent(
                        type="text",
                        text=f"Unknown tool: {name}"
                    )]
            except Exception as e:
                self.logger.error(f"Error in tool call {name}: {e}")
                return [TextContent(
                    type="text", 
                    text=f"Error: {str(e)}"
                )]
        
        # Register tools list handler
        @self.server_app.list_tools()
        async def list_tools() -> list[types.Tool]:
            """Return available tools"""
            return self.get_tools()
        
        # Register resources list handler
        @self.server_app.list_resources()
        async def list_resources() -> list[types.Resource]:
            """Return available resources"""
            return self.get_resources()
        
        # Register resource read handler
        @self.server_app.read_resource()
        async def read_resource(uri: str) -> str:
            """Read and return resource content"""
            return await self.read_resource(uri)
        
        # Register prompts list handler
        @self.server_app.list_prompts()
        async def list_prompts() -> list[types.Prompt]:
            """Return available prompts"""
            return self.get_prompts()
        
        # Register prompt get handler
        @self.server_app.get_prompt()
        async def get_prompt(name: str, arguments: dict) -> types.GetPromptResult:
            """Get prompt content with arguments filled in"""
            return await self.get_prompt(name, arguments)
        
        return self.server_app
    
    async def run_stdio(self):
        """Run the server with stdio transport"""
        if not MCP_AVAILABLE:
            self.logger.error("Cannot run MCP server: MCP SDK not available")
            print("Error: MCP SDK not available", file=sys.stderr)
            return 1
        
        try:
            server_app = self.create_server()
            
            self.logger.info("Starting MCP RAG Indexer server with stdio transport")
            
            # Run the server
            async with stdio_server() as streams:
                await server_app.run(
                    streams[0], streams[1],
                    server.InitializationOptions(
                        server_name="mcp-rag-indexer",
                        server_version=VERSION,
                        capabilities=types.ServerCapabilities(
                            tools=types.ToolsCapability(),
                            resources=types.ResourcesCapability(subscribe=False),
                            prompts=types.PromptsCapability()
                        )
                    )
                )
                
        except Exception as e:
            self.logger.error(f"Server error: {e}")
            return 1
        
        return 0
    
    def run_cli(self, args):
        """Run CLI commands"""
        if args.version:
            print(f"MCP RAG Indexer v{VERSION}")
            print(DESCRIPTION)
            return 0
        
        # argparse handles --help automatically, so this is not needed
        
        # Default: run as MCP server
        if MCP_AVAILABLE:
            return asyncio.run(self.run_stdio())
        else:
            print("Error: MCP SDK not available. Cannot run as MCP server.", file=sys.stderr)
            print("Install with: pip install mcp", file=sys.stderr)
            return 1
    
    def shutdown(self):
        """Clean shutdown"""
        self.logger.info("Shutting down MCP RAG Indexer server")
        # Cleanup resources here
        pass


def create_argument_parser():
    """Create command line argument parser"""
    parser = argparse.ArgumentParser(
        description=DESCRIPTION,
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument(
        '--version',
        action='store_true',
        help='Show version information'
    )
    
    parser.add_argument(
        '--log-level',
        choices=['debug', 'info', 'warning', 'error'],
        default='info',
        help='Set logging level (default: info)'
    )
    
    parser.add_argument(
        '--help-mcp',
        action='store_true',
        help='Show MCP-specific help'
    )
    
    return parser


def main():
    """Main entry point"""
    parser = create_argument_parser()
    args = parser.parse_args()
    
    # Handle special case for --help (since argparse handles this automatically)
    # Note: argparse automatically handles --help, so this is not needed
    
    try:
        server = RAGIndexerServer(log_level=args.log_level)
        exit_code = server.run_cli(args)
        server.shutdown()
        sys.exit(exit_code)
        
    except KeyboardInterrupt:
        print("\nShutdown requested...", file=sys.stderr)
        sys.exit(0)
    except Exception as e:
        print(f"Fatal error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()