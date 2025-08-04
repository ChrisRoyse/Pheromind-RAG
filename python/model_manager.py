"""
Offline Model Manager for MCP RAG Indexer
Handles loading and management of pre-bundled ML models
"""

import os
import sys
import logging
from pathlib import Path
from typing import Optional, Dict, Any
import json

# Add current directory to Python path for local imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

logger = logging.getLogger(__name__)


class OfflineModelManager:
    """Manages pre-bundled ML models for offline operation"""
    
    def __init__(self, package_root: Optional[Path] = None):
        """
        Initialize the offline model manager
        
        Args:
            package_root: Root directory of the package (auto-detected if None)
        """
        if package_root is None:
            # Auto-detect package root (go up from python/ to package root)
            self.package_root = Path(__file__).parent.parent
        else:
            self.package_root = Path(package_root)
            
        self.models_dir = self.package_root / 'models'
        self.cache = {}
        
        # Set environment variables for offline operation
        self._setup_offline_environment()
        
        logger.info(f"Offline model manager initialized with models dir: {self.models_dir}")
    
    def _setup_offline_environment(self):
        """Set environment variables to enable offline model loading"""
        model_cache_dirs = [
            str(self.models_dir),
            str(self.models_dir / 'transformers'),
            str(self.models_dir / 'sentence_transformers')
        ]
        
        # Set cache directories
        os.environ['TRANSFORMERS_CACHE'] = str(self.models_dir)
        os.environ['SENTENCE_TRANSFORMERS_HOME'] = str(self.models_dir)
        os.environ['HF_HOME'] = str(self.models_dir)
        
        # Enable offline mode
        os.environ['HF_HUB_OFFLINE'] = '1'
        os.environ['TRANSFORMERS_OFFLINE'] = '1'
        
        # Performance optimizations
        os.environ['TOKENIZERS_PARALLELISM'] = 'false'
        os.environ['OMP_NUM_THREADS'] = '1'
        
        logger.debug(f"Set model cache directories: {model_cache_dirs}")
    
    def is_model_available(self, model_name: str) -> bool:
        """
        Check if a model is available locally
        
        Args:
            model_name: Name of the model (e.g., 'sentence-transformers/all-MiniLM-L6-v2')
            
        Returns:
            True if model is available locally
        """
        model_path = self.models_dir / model_name
        
        if not model_path.exists():
            logger.debug(f"Model directory not found: {model_path}")
            return False
        
        # Check for essential files
        essential_files = ['config.json', 'pytorch_model.bin']
        for file_name in essential_files:
            if not (model_path / file_name).exists():
                logger.debug(f"Essential model file missing: {model_path / file_name}")
                return False
        
        return True
    
    def load_sentence_transformer(self, model_name: str = 'sentence-transformers/all-MiniLM-L6-v2'):
        """
        Load a sentence transformer model from local files
        
        Args:
            model_name: Name of the sentence transformer model
            
        Returns:
            SentenceTransformer model instance
            
        Raises:
            ImportError: If sentence-transformers is not available
            FileNotFoundError: If model files are not found
        """
        if model_name in self.cache:
            logger.debug(f"Returning cached model: {model_name}")
            return self.cache[model_name]
        
        try:
            from sentence_transformers import SentenceTransformer
        except ImportError as e:
            logger.error("sentence-transformers not available")
            raise ImportError(
                "sentence-transformers is required for embedding models. "
                "Please install it with: pip install sentence-transformers"
            ) from e
        
        model_path = self.models_dir / model_name
        
        if self.is_model_available(model_name):
            logger.info(f"Loading sentence transformer from local files: {model_path}")
            try:
                model = SentenceTransformer(str(model_path))
                self.cache[model_name] = model
                return model
            except Exception as e:
                logger.warning(f"Failed to load local model {model_name}: {e}")
                # Fall through to online fallback
        
        # Fallback to online download if local files not available
        logger.info(f"Downloading sentence transformer model: {model_name}")
        try:
            model = SentenceTransformer(model_name)
            self.cache[model_name] = model
            return model
        except Exception as e:
            logger.error(f"Failed to load model {model_name}: {e}")
            raise FileNotFoundError(
                f"Model {model_name} not found locally and download failed. "
                f"Please ensure the model is bundled or internet is available."
            ) from e
    
    def load_tokenizer(self, model_name: str):
        """
        Load a tokenizer from local files
        
        Args:
            model_name: Name of the model
            
        Returns:
            Tokenizer instance
        """
        try:
            from transformers import AutoTokenizer
        except ImportError as e:
            logger.error("transformers not available")
            raise ImportError(
                "transformers is required for tokenizers. "
                "Please install it with: pip install transformers"
            ) from e
        
        model_path = self.models_dir / model_name
        
        if self.is_model_available(model_name):
            logger.info(f"Loading tokenizer from local files: {model_path}")
            try:
                tokenizer = AutoTokenizer.from_pretrained(str(model_path), local_files_only=True)
                return tokenizer
            except Exception as e:
                logger.warning(f"Failed to load local tokenizer {model_name}: {e}")
        
        # Fallback to online
        logger.info(f"Downloading tokenizer: {model_name}")
        try:
            tokenizer = AutoTokenizer.from_pretrained(model_name)
            return tokenizer
        except Exception as e:
            logger.error(f"Failed to load tokenizer {model_name}: {e}")
            raise FileNotFoundError(
                f"Tokenizer {model_name} not found locally and download failed."
            ) from e
    
    def get_model_info(self, model_name: str) -> Dict[str, Any]:
        """
        Get information about a local model
        
        Args:
            model_name: Name of the model
            
        Returns:
            Dictionary with model information
        """
        model_path = self.models_dir / model_name
        
        if not model_path.exists():
            return {'available': False, 'error': 'Model directory not found'}
        
        info = {
            'available': self.is_model_available(model_name),
            'path': str(model_path),
            'files': []
        }
        
        try:
            # List all files in model directory
            for file_path in model_path.rglob('*'):
                if file_path.is_file():
                    relative_path = file_path.relative_to(model_path)
                    file_size = file_path.stat().st_size
                    info['files'].append({
                        'name': str(relative_path),
                        'size': file_size,
                        'size_mb': round(file_size / (1024 * 1024), 2)
                    })
            
            # Load config if available
            config_path = model_path / 'config.json'
            if config_path.exists():
                try:
                    with open(config_path, 'r', encoding='utf-8') as f:
                        info['config'] = json.load(f)
                except Exception as e:
                    info['config_error'] = str(e)
            
            # Calculate total size
            total_size = sum(f['size'] for f in info['files'])
            info['total_size'] = total_size
            info['total_size_mb'] = round(total_size / (1024 * 1024), 2)
            
        except Exception as e:
            info['error'] = str(e)
        
        return info
    
    def list_available_models(self) -> Dict[str, Dict[str, Any]]:
        """
        List all available models in the models directory
        
        Returns:
            Dictionary mapping model names to their info
        """
        if not self.models_dir.exists():
            logger.warning(f"Models directory not found: {self.models_dir}")
            return {}
        
        models = {}
        
        try:
            # Look for model directories
            for model_org_dir in self.models_dir.iterdir():
                if model_org_dir.is_dir() and not model_org_dir.name.startswith('.'):
                    # Check if it's a direct model or organization directory
                    config_path = model_org_dir / 'config.json'
                    
                    if config_path.exists():
                        # Direct model directory
                        model_name = model_org_dir.name
                        models[model_name] = self.get_model_info(model_name)
                    else:
                        # Organization directory, check subdirectories
                        for model_dir in model_org_dir.iterdir():
                            if model_dir.is_dir():
                                model_config = model_dir / 'config.json'
                                if model_config.exists():
                                    model_name = f"{model_org_dir.name}/{model_dir.name}"
                                    models[model_name] = self.get_model_info(model_name)
        
        except Exception as e:
            logger.error(f"Error listing models: {e}")
        
        return models
    
    def get_default_embedding_model(self):
        """
        Get the default embedding model
        
        Returns:
            SentenceTransformer model instance
        """
        default_models = [
            'sentence-transformers/all-MiniLM-L6-v2',
            'sentence-transformers/all-mpnet-base-v2'
        ]
        
        for model_name in default_models:
            if self.is_model_available(model_name):
                logger.info(f"Using default embedding model: {model_name}")
                return self.load_sentence_transformer(model_name)
        
        # If no bundled models available, try to download the smallest one
        logger.warning("No bundled models found, attempting to download default model")
        return self.load_sentence_transformer(default_models[0])
    
    def clear_cache(self):
        """Clear the model cache"""
        self.cache.clear()
        logger.info("Model cache cleared")
    
    def get_cache_info(self) -> Dict[str, Any]:
        """
        Get information about cached models
        
        Returns:
            Dictionary with cache information
        """
        return {
            'cached_models': list(self.cache.keys()),
            'cache_size': len(self.cache),
            'models_dir': str(self.models_dir),
            'models_dir_exists': self.models_dir.exists()
        }


# Global instance for easy importing
_global_manager = None


def get_model_manager() -> OfflineModelManager:
    """
    Get the global model manager instance
    
    Returns:
        OfflineModelManager instance
    """
    global _global_manager
    if _global_manager is None:
        _global_manager = OfflineModelManager()
    return _global_manager


def load_embedding_model(model_name: str = None):
    """
    Convenience function to load an embedding model
    
    Args:
        model_name: Name of the model to load (uses default if None)
        
    Returns:
        SentenceTransformer model instance
    """
    manager = get_model_manager()
    
    if model_name is None:
        return manager.get_default_embedding_model()
    else:
        return manager.load_sentence_transformer(model_name)


if __name__ == '__main__':
    # CLI interface for testing
    import argparse
    
    parser = argparse.ArgumentParser(description='Offline Model Manager')
    parser.add_argument('--list', action='store_true', help='List available models')
    parser.add_argument('--info', type=str, help='Get info about a specific model')
    parser.add_argument('--test', type=str, help='Test loading a model')
    parser.add_argument('--cache-info', action='store_true', help='Show cache information')
    
    args = parser.parse_args()
    
    # Setup logging
    logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')
    
    manager = get_model_manager()
    
    if args.list:
        models = manager.list_available_models()
        print(f"Found {len(models)} models:")
        for model_name, info in models.items():
            status = "✓" if info['available'] else "✗"
            size = info.get('total_size_mb', 0)
            print(f"  {status} {model_name} ({size:.1f} MB)")
    
    elif args.info:
        info = manager.get_model_info(args.info)
        print(f"Model: {args.info}")
        print(f"Available: {info['available']}")
        print(f"Path: {info['path']}")
        if 'total_size_mb' in info:
            print(f"Size: {info['total_size_mb']:.1f} MB")
        if 'files' in info:
            print(f"Files: {len(info['files'])}")
    
    elif args.test:
        try:
            print(f"Loading model: {args.test}")
            model = manager.load_sentence_transformer(args.test)
            print(f"✓ Successfully loaded: {type(model).__name__}")
            
            # Test encoding
            test_text = "This is a test sentence."
            embedding = model.encode(test_text)
            print(f"✓ Encoding test passed: {embedding.shape}")
            
        except Exception as e:
            print(f"✗ Failed to load model: {e}")
    
    elif args.cache_info:
        info = manager.get_cache_info()
        print("Cache Information:")
        for key, value in info.items():
            print(f"  {key}: {value}")
    
    else:
        print("Use --help for usage information")