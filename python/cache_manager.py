#!/usr/bin/env python3
"""
Cache Manager for MCP RAG Indexer
Handles caching of embeddings, models, and project indexes
"""

import argparse
import json
import os
import shutil
import sys
import time
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional, Any

import numpy as np


class CacheManager:
    """Manages cache for embeddings, models, and indexes"""
    
    def __init__(self, cache_path: str):
        self.cache_path = Path(cache_path)
        self.embeddings_dir = self.cache_path / 'embeddings'
        self.models_dir = self.cache_path / 'models'
        self.indexes_dir = self.cache_path / 'indexes'
        self.metadata_file = self.cache_path / 'cache_metadata.json'
    
    def init_cache(self) -> Dict[str, Any]:
        """Initialize cache directory structure"""
        try:
            # Create directories
            self.cache_path.mkdir(parents=True, exist_ok=True)
            self.embeddings_dir.mkdir(exist_ok=True)
            self.models_dir.mkdir(exist_ok=True)
            self.indexes_dir.mkdir(exist_ok=True)
            
            # Create metadata
            metadata = {
                'version': '1.0.0',
                'created_at': datetime.now().isoformat(),
                'last_updated': datetime.now().isoformat(),
                'cache_size': 0,
                'entries': []
            }
            
            with open(self.metadata_file, 'w') as f:
                json.dump(metadata, f, indent=2)
            
            return {
                'success': True,
                'cache_path': str(self.cache_path)
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def store_embedding(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Store embedding in cache"""
        try:
            chunk_hash = data['chunk_hash']
            embedding = np.array(data['embedding'])
            
            # Save embedding as numpy array
            embedding_file = self.embeddings_dir / f"{chunk_hash}.npy"
            np.save(embedding_file, embedding)
            
            # Update metadata
            self._update_metadata('embedding', {
                'chunk_hash': chunk_hash,
                'project': data.get('project', ''),
                'file': data.get('file', ''),
                'size': embedding_file.stat().st_size,
                'cached_at': datetime.now().isoformat()
            })
            
            return {
                'success': True,
                'cache_key': chunk_hash
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def get_embedding(self, chunk_hash: str) -> Dict[str, Any]:
        """Retrieve embedding from cache"""
        try:
            embedding_file = self.embeddings_dir / f"{chunk_hash}.npy"
            
            if not embedding_file.exists():
                return {
                    'success': False,
                    'error': f'Embedding {chunk_hash} not found'
                }
            
            embedding = np.load(embedding_file)
            
            return {
                'success': True,
                'embedding': embedding.tolist()
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def store_model(self, model_name: str, model_file: str) -> Dict[str, Any]:
        """Store model file in cache"""
        try:
            source_path = Path(model_file)
            target_path = self.models_dir / f"{model_name}.bin"
            
            # Copy model file
            shutil.copy2(source_path, target_path)
            
            # Update metadata
            self._update_metadata('model', {
                'name': model_name,
                'size': target_path.stat().st_size,
                'cached_at': datetime.now().isoformat()
            })
            
            return {
                'success': True,
                'model_path': str(target_path)
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def list_models(self) -> Dict[str, Any]:
        """List all cached models"""
        try:
            models = []
            
            for model_file in self.models_dir.glob("*.bin"):
                stat = model_file.stat()
                models.append({
                    'name': model_file.stem,
                    'size': stat.st_size,
                    'cached_at': datetime.fromtimestamp(stat.st_mtime).isoformat()
                })
            
            return {
                'models': models
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def store_index(self, project_name: str, index_data: Dict[str, Any]) -> Dict[str, Any]:
        """Store project index in cache"""
        try:
            index_file = self.indexes_dir / f"{project_name}.json"
            
            with open(index_file, 'w') as f:
                json.dump(index_data, f, indent=2)
            
            # Update metadata
            self._update_metadata('index', {
                'project_name': project_name,
                'size': index_file.stat().st_size,
                'cached_at': datetime.now().isoformat()
            })
            
            return {
                'success': True
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def get_index(self, project_name: str) -> Dict[str, Any]:
        """Retrieve project index from cache"""
        try:
            index_file = self.indexes_dir / f"{project_name}.json"
            
            if not index_file.exists():
                return {
                    'success': False,
                    'error': f'Index for project {project_name} not found'
                }
            
            with open(index_file, 'r') as f:
                index_data = json.load(f)
            
            return {
                'success': True,
                'index_data': index_data
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def get_stats(self) -> Dict[str, Any]:
        """Get cache statistics"""
        try:
            # Validate cache directory exists
            if not self.cache_path.exists():
                return {
                    'success': False,
                    'error': f'cache directory {self.cache_path} does not exist'
                }
            
            total_size = 0
            embeddings_count = 0
            models_count = 0
            indexes_count = 0
            
            # Count embeddings
            if self.embeddings_dir.exists():
                for embedding_file in self.embeddings_dir.glob("*.npy"):
                    total_size += embedding_file.stat().st_size
                    embeddings_count += 1
            
            # Count models
            if self.models_dir.exists():
                for model_file in self.models_dir.glob("*.bin"):
                    total_size += model_file.stat().st_size
                    models_count += 1
            
            # Count indexes
            if self.indexes_dir.exists():
                for index_file in self.indexes_dir.glob("*.json"):
                    total_size += index_file.stat().st_size
                    indexes_count += 1
            
            return {
                'total_size': total_size,
                'embeddings_count': embeddings_count,
                'models_count': models_count,
                'indexes_count': indexes_count
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def cleanup(self, max_age_days: int = 30) -> Dict[str, Any]:
        """Clean up expired cache entries"""
        try:
            cutoff_time = datetime.now() - timedelta(days=max_age_days)
            cleaned_count = 0
            
            # Clean embeddings
            for embedding_file in self.embeddings_dir.glob("*.npy"):
                if datetime.fromtimestamp(embedding_file.stat().st_mtime) < cutoff_time:
                    embedding_file.unlink()
                    cleaned_count += 1
            
            # Clean models
            for model_file in self.models_dir.glob("*.bin"):
                if datetime.fromtimestamp(model_file.stat().st_mtime) < cutoff_time:
                    model_file.unlink()
                    cleaned_count += 1
            
            # Clean indexes
            for index_file in self.indexes_dir.glob("*.json"):
                if datetime.fromtimestamp(index_file.stat().st_mtime) < cutoff_time:
                    index_file.unlink()
                    cleaned_count += 1
            
            return {
                'success': True,
                'cleaned_count': cleaned_count
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def enforce_size_limit(self, max_size_bytes: int) -> Dict[str, Any]:
        """Enforce cache size limit by removing oldest entries"""
        try:
            current_size = self.get_stats()['total_size']
            
            if current_size <= max_size_bytes:
                return {
                    'success': True,
                    'removed_count': 0
                }
            
            # Get all files with timestamps
            files_with_time = []
            
            for embedding_file in self.embeddings_dir.glob("*.npy"):
                files_with_time.append((embedding_file, embedding_file.stat().st_mtime))
            
            for model_file in self.models_dir.glob("*.bin"):
                files_with_time.append((model_file, model_file.stat().st_mtime))
            
            for index_file in self.indexes_dir.glob("*.json"):
                files_with_time.append((index_file, index_file.stat().st_mtime))
            
            # Sort by modification time (oldest first)
            files_with_time.sort(key=lambda x: x[1])
            
            removed_count = 0
            current_size = self.get_stats()['total_size']
            
            for file_path, _ in files_with_time:
                if current_size <= max_size_bytes:
                    break
                
                file_size = file_path.stat().st_size
                file_path.unlink()
                current_size -= file_size
                removed_count += 1
            
            return {
                'success': True,
                'removed_count': removed_count
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def validate(self) -> Dict[str, Any]:
        """Validate cache integrity"""
        try:
            checks_performed = []
            issues_found = 0
            
            # Check directory structure
            checks_performed.append("Directory structure")
            if not all([
                self.cache_path.exists(),
                self.embeddings_dir.exists(),
                self.models_dir.exists(),
                self.indexes_dir.exists()
            ]):
                issues_found += 1
            
            # Check metadata file
            checks_performed.append("Metadata file")
            if not self.metadata_file.exists():
                issues_found += 1
            else:
                try:
                    with open(self.metadata_file, 'r') as f:
                        json.load(f)
                except json.JSONDecodeError:
                    issues_found += 1
            
            # Validate embedding files
            checks_performed.append("Embedding files")
            for embedding_file in self.embeddings_dir.glob("*.npy"):
                try:
                    np.load(embedding_file)
                except Exception:
                    issues_found += 1
            
            # Validate index files
            checks_performed.append("Index files")
            for index_file in self.indexes_dir.glob("*.json"):
                try:
                    with open(index_file, 'r') as f:
                        json.load(f)
                except json.JSONDecodeError:
                    issues_found += 1
            
            return {
                'valid': issues_found == 0,
                'checks_performed': checks_performed,
                'issues_found': issues_found
            }
            
        except Exception as e:
            return {
                'valid': False,
                'error': str(e)
            }
    
    def repair(self) -> Dict[str, Any]:
        """Repair corrupted cache files"""
        try:
            repairs_made = 0
            
            # Repair metadata file
            if not self.metadata_file.exists() or not self._is_valid_json(self.metadata_file):
                self.init_cache()
                repairs_made += 1
            
            # Remove corrupted embedding files
            for embedding_file in self.embeddings_dir.glob("*.npy"):
                try:
                    np.load(embedding_file)
                except Exception:
                    embedding_file.unlink()
                    repairs_made += 1
            
            # Remove corrupted index files
            for index_file in self.indexes_dir.glob("*.json"):
                if not self._is_valid_json(index_file):
                    index_file.unlink()
                    repairs_made += 1
            
            return {
                'success': True,
                'repairs_made': repairs_made
            }
            
        except Exception as e:
            return {
                'success': False,
                'error': str(e)
            }
    
    def _update_metadata(self, entry_type: str, entry_data: Dict[str, Any]):
        """Update cache metadata"""
        try:
            if self.metadata_file.exists():
                with open(self.metadata_file, 'r') as f:
                    metadata = json.load(f)
            else:
                metadata = {
                    'version': '1.0.0',
                    'created_at': datetime.now().isoformat(),
                    'entries': []
                }
            
            # Add entry
            entry_data['type'] = entry_type
            metadata['entries'].append(entry_data)
            metadata['last_updated'] = datetime.now().isoformat()
            
            # Update cache size
            metadata['cache_size'] = self.get_stats().get('total_size', 0)
            
            with open(self.metadata_file, 'w') as f:
                json.dump(metadata, f, indent=2)
                
        except Exception:
            pass  # Non-critical operation
    
    def _is_valid_json(self, file_path: Path) -> bool:
        """Check if file contains valid JSON"""
        try:
            with open(file_path, 'r') as f:
                json.load(f)
            return True
        except (json.JSONDecodeError, OSError):
            return False


def format_output_json(data: Dict[str, Any]) -> str:
    """Format output as JSON"""
    return json.dumps(data, indent=2)


def format_output_text(data: Dict[str, Any]) -> str:
    """Format output as plain text"""
    if 'total_size' in data:
        # Stats output
        return f"""Cache Statistics:
Total Size: {data['total_size']:,} bytes
Embeddings: {data['embeddings_count']}
Models: {data['models_count']}
Indexes: {data['indexes_count']}"""
    else:
        # Generic output
        return str(data)


def main():
    parser = argparse.ArgumentParser(description='MCP RAG Cache Manager')
    parser.add_argument('--cache-path', required=True, help='Cache directory path')
    parser.add_argument('--format', choices=['json', 'text'], default='json', help='Output format')
    
    # Operations
    parser.add_argument('--init', action='store_true', help='Initialize cache')
    parser.add_argument('--store-embedding', action='store_true', help='Store embedding')
    parser.add_argument('--get-embedding', action='store_true', help='Get embedding')
    parser.add_argument('--store-model', action='store_true', help='Store model')
    parser.add_argument('--list-models', action='store_true', help='List models')
    parser.add_argument('--store-index', action='store_true', help='Store index')
    parser.add_argument('--get-index', action='store_true', help='Get index')
    parser.add_argument('--stats', action='store_true', help='Get statistics')
    parser.add_argument('--cleanup', action='store_true', help='Cleanup old entries')
    parser.add_argument('--enforce-size-limit', action='store_true', help='Enforce size limit')
    parser.add_argument('--validate', action='store_true', help='Validate cache')
    parser.add_argument('--repair', action='store_true', help='Repair cache')
    
    # Parameters
    parser.add_argument('--data', help='JSON data for operations')
    parser.add_argument('--chunk-hash', help='Chunk hash for embedding operations')
    parser.add_argument('--model-name', help='Model name')
    parser.add_argument('--model-file', help='Model file path')
    parser.add_argument('--project-name', help='Project name')
    parser.add_argument('--index-data', help='Index data as JSON')
    parser.add_argument('--max-age', type=int, default=30, help='Max age in days for cleanup')
    parser.add_argument('--max-size', type=int, help='Max cache size in bytes')
    
    args = parser.parse_args()
    
    if not args.cache_path:
        print("Error: --cache-path is required", file=sys.stderr)
        sys.exit(1)
    
    cache_manager = CacheManager(args.cache_path)
    
    try:
        if args.init:
            result = cache_manager.init_cache()
        elif args.store_embedding:
            if not args.data:
                print("Error: --data required for store-embedding", file=sys.stderr)
                sys.exit(1)
            data = json.loads(args.data)
            result = cache_manager.store_embedding(data)
        elif args.get_embedding:
            if not args.chunk_hash:
                print("Error: --chunk-hash required for get-embedding", file=sys.stderr)
                sys.exit(1)
            result = cache_manager.get_embedding(args.chunk_hash)
        elif args.store_model:
            if not args.model_name or not args.model_file:
                print("Error: --model-name and --model-file required for store-model", file=sys.stderr)
                sys.exit(1)
            result = cache_manager.store_model(args.model_name, args.model_file)
        elif args.list_models:
            result = cache_manager.list_models()
        elif args.store_index:
            if not args.project_name or not args.index_data:
                print("Error: --project-name and --index-data required for store-index", file=sys.stderr)
                sys.exit(1)
            index_data = json.loads(args.index_data)
            result = cache_manager.store_index(args.project_name, index_data)
        elif args.get_index:
            if not args.project_name:
                print("Error: --project-name required for get-index", file=sys.stderr)
                sys.exit(1)
            result = cache_manager.get_index(args.project_name)
        elif args.stats:
            result = cache_manager.get_stats()
        elif args.cleanup:
            result = cache_manager.cleanup(args.max_age)
        elif args.enforce_size_limit:
            if not args.max_size:
                print("Error: --max-size required for enforce-size-limit", file=sys.stderr)
                sys.exit(1)
            result = cache_manager.enforce_size_limit(args.max_size)
        elif args.validate:
            result = cache_manager.validate()
        elif args.repair:
            result = cache_manager.repair()
        else:
            parser.print_help()
            sys.exit(0)
        
        # Format and output result
        if isinstance(result, dict) and result.get('success') is False:
            # For errors, output to stderr unless it's a "not found" case (graceful handling)
            error_msg = result.get('error', '').lower()
            if 'not found' in error_msg:
                # Graceful handling - output to stdout
                if args.format == 'json':
                    print(format_output_json(result))
                else:
                    print(format_output_text(result))
            else:
                # Actual error - output to stderr
                if args.format == 'json':
                    print(format_output_json(result), file=sys.stderr)
                else:
                    print(format_output_text(result), file=sys.stderr)
                sys.exit(1)
        else:
            # Success case - output to stdout
            if args.format == 'json':
                print(format_output_json(result))
            else:
                print(format_output_text(result))
            
    except Exception as e:
        error_result = {'success': False, 'error': str(e)}
        if args.format == 'json':
            print(format_output_json(error_result))
        else:
            print(f"Error: {str(e)}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()