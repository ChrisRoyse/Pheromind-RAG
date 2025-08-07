#!/usr/bin/env python3
"""
Comprehensive Test Suite for Embed Search System
Tests all 4 parallel search methods without modifying source code
"""

import subprocess
import json
import time
import sys
from pathlib import Path
from typing import Dict, List, Any, Tuple
from datetime import datetime

class SearchMethodTester:
    """Test harness for the 4 parallel search methods"""
    
    def __init__(self):
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "system": "embed-search",
            "version": "0.1.0",
            "search_methods": {},
            "errors": [],
            "performance": {},
            "functionality": {}
        }
        self.project_root = Path.cwd()
        
    def run_command(self, cmd: List[str], timeout: int = 30) -> Tuple[int, str, str]:
        """Execute command and capture output"""
        try:
            result = subprocess.run(
                cmd, 
                capture_output=True, 
                text=True, 
                timeout=timeout,
                cwd=self.project_root
            )
            return result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired:
            return -1, "", f"Command timed out after {timeout} seconds"
        except Exception as e:
            return -1, "", str(e)
            
    def test_ripgrep_search(self) -> Dict[str, Any]:
        """Test Ripgrep-based search functionality"""
        print("\n🔍 Testing Ripgrep/Native Search...")
        results = {
            "name": "Ripgrep/Native Search",
            "status": "unknown",
            "tests": [],
            "performance": {}
        }
        
        # Test 1: Basic text search
        start = time.time()
        code, stdout, stderr = self.run_command(
            ["cargo", "run", "--", "search", "struct"]
        )
        elapsed = time.time() - start
        
        results["tests"].append({
            "name": "Basic text search",
            "query": "struct",
            "success": code == 0,
            "execution_time": elapsed,
            "output_lines": len(stdout.splitlines()) if stdout else 0,
            "error": stderr if code != 0 else None
        })
        
        # Test 2: Regex pattern search
        start = time.time()
        code, stdout, stderr = self.run_command(
            ["cargo", "run", "--", "search", "fn.*test"]
        )
        elapsed = time.time() - start
        
        results["tests"].append({
            "name": "Regex pattern search",
            "query": "fn.*test",
            "success": code == 0,
            "execution_time": elapsed,
            "output_lines": len(stdout.splitlines()) if stdout else 0
        })
        
        # Test 3: Case-insensitive search
        start = time.time()
        code, stdout, stderr = self.run_command(
            ["cargo", "run", "--", "search", "--ignore-case", "ERROR"]
        )
        elapsed = time.time() - start
        
        results["tests"].append({
            "name": "Case-insensitive search",
            "query": "ERROR (ignore case)",
            "success": code == 0,
            "execution_time": elapsed,
            "output_lines": len(stdout.splitlines()) if stdout else 0
        })
        
        # Calculate overall status
        successful_tests = sum(1 for t in results["tests"] if t["success"])
        results["status"] = "working" if successful_tests > len(results["tests"]) / 2 else "broken"
        results["success_rate"] = f"{successful_tests}/{len(results['tests'])}"
        results["average_time"] = sum(t["execution_time"] for t in results["tests"]) / len(results["tests"])
        
        return results
        
    def test_tantivy_search(self) -> Dict[str, Any]:
        """Test Tantivy full-text search"""
        print("\n📚 Testing Tantivy Full-Text Search...")
        results = {
            "name": "Tantivy Full-Text Search",
            "status": "unknown",
            "tests": [],
            "index_info": {}
        }
        
        # Check if index exists
        index_path = self.project_root / ".tantivy_index"
        if index_path.exists():
            results["index_info"]["exists"] = True
            results["index_info"]["size_mb"] = sum(
                f.stat().st_size for f in index_path.rglob("*") if f.is_file()
            ) / (1024 * 1024)
        else:
            results["index_info"]["exists"] = False
            
        # Test 1: Build with tantivy feature
        start = time.time()
        code, stdout, stderr = self.run_command(
            ["cargo", "build", "--features", "tantivy"],
            timeout=120
        )
        elapsed = time.time() - start
        
        results["tests"].append({
            "name": "Build with tantivy feature",
            "success": code == 0,
            "execution_time": elapsed,
            "error": stderr if code != 0 else None
        })
        
        if code == 0:
            # Test 2: Index creation
            start = time.time()
            code, stdout, stderr = self.run_command(
                ["cargo", "run", "--features", "tantivy", "--", "index", "."]
            )
            elapsed = time.time() - start
            
            results["tests"].append({
                "name": "Index creation",
                "success": code == 0,
                "execution_time": elapsed,
                "indexed_files": stdout.count("Indexed") if stdout else 0
            })
            
            # Test 3: Fuzzy search
            start = time.time()
            code, stdout, stderr = self.run_command(
                ["cargo", "run", "--features", "tantivy", "--", "search", "--fuzzy", "serch"]
            )
            elapsed = time.time() - start
            
            results["tests"].append({
                "name": "Fuzzy search",
                "query": "serch (fuzzy)",
                "success": code == 0,
                "execution_time": elapsed,
                "matches": len(stdout.splitlines()) if stdout else 0
            })
            
        # Calculate status
        successful_tests = sum(1 for t in results["tests"] if t["success"])
        results["status"] = "working" if successful_tests == len(results["tests"]) else "partial"
        results["success_rate"] = f"{successful_tests}/{len(results['tests'])}"
        
        return results
        
    def test_vector_embedding_search(self) -> Dict[str, Any]:
        """Test Vector/Embedding search"""
        print("\n🧠 Testing Vector/Embedding Search...")
        results = {
            "name": "Vector/Embedding Search",
            "status": "unknown",
            "tests": [],
            "model_info": {}
        }
        
        # Check for model files
        model_paths = [
            Path.home() / ".cache" / "huggingface",
            Path.home() / ".cache" / "nomic",
            self.project_root / "models"
        ]
        
        for path in model_paths:
            if path.exists():
                gguf_files = list(path.rglob("*.gguf"))
                if gguf_files:
                    results["model_info"]["model_found"] = True
                    results["model_info"]["model_path"] = str(gguf_files[0])
                    results["model_info"]["model_size_mb"] = gguf_files[0].stat().st_size / (1024 * 1024)
                    break
        else:
            results["model_info"]["model_found"] = False
            
        # Test 1: Build with ML features
        start = time.time()
        code, stdout, stderr = self.run_command(
            ["cargo", "build", "--features", "ml,vectordb"],
            timeout=180
        )
        elapsed = time.time() - start
        
        results["tests"].append({
            "name": "Build with ML features",
            "success": code == 0,
            "execution_time": elapsed,
            "error": stderr[:500] if code != 0 else None  # Truncate long errors
        })
        
        if code == 0:
            # Test 2: Embedding generation
            start = time.time()
            code, stdout, stderr = self.run_command(
                ["cargo", "run", "--features", "ml,vectordb", "--", "test"]
            )
            elapsed = time.time() - start
            
            results["tests"].append({
                "name": "Embedding generation test",
                "success": code == 0,
                "execution_time": elapsed
            })
            
            # Test 3: Semantic search
            start = time.time()
            code, stdout, stderr = self.run_command(
                ["cargo", "run", "--features", "ml,vectordb", "--", "search", "--semantic", "error handling"]
            )
            elapsed = time.time() - start
            
            results["tests"].append({
                "name": "Semantic similarity search",
                "query": "error handling",
                "success": code == 0,
                "execution_time": elapsed,
                "semantic_matches": stdout.count("similarity:") if stdout else 0
            })
            
        # Calculate status
        successful_tests = sum(1 for t in results["tests"] if t["success"])
        results["status"] = "working" if successful_tests == len(results["tests"]) else "not_working"
        results["success_rate"] = f"{successful_tests}/{len(results['tests'])}"
        
        return results
        
    def test_ast_symbol_search(self) -> Dict[str, Any]:
        """Test AST-based symbol search"""
        print("\n🌳 Testing AST/Symbol Search...")
        results = {
            "name": "AST/Symbol Search",
            "status": "unknown",
            "tests": [],
            "languages_supported": []
        }
        
        # Test 1: Build with tree-sitter
        start = time.time()
        code, stdout, stderr = self.run_command(
            ["cargo", "build", "--features", "tree-sitter"],
            timeout=120
        )
        elapsed = time.time() - start
        
        results["tests"].append({
            "name": "Build with tree-sitter",
            "success": code == 0,
            "execution_time": elapsed,
            "error": stderr[:500] if code != 0 else None
        })
        
        if code == 0:
            # Test 2: Symbol extraction
            start = time.time()
            code, stdout, stderr = self.run_command(
                ["cargo", "run", "--features", "tree-sitter", "--bin", "verify_symbols"]
            )
            elapsed = time.time() - start
            
            results["tests"].append({
                "name": "Symbol extraction",
                "success": code == 0,
                "execution_time": elapsed,
                "symbols_found": stdout.count("Symbol:") if stdout else 0
            })
            
            # Test 3: Symbol search
            start = time.time()
            code, stdout, stderr = self.run_command(
                ["cargo", "run", "--features", "tree-sitter", "--", "search", "--symbols", "struct"]
            )
            elapsed = time.time() - start
            
            results["tests"].append({
                "name": "Symbol search",
                "query": "struct",
                "success": code == 0,
                "execution_time": elapsed,
                "symbol_matches": len(stdout.splitlines()) if stdout else 0
            })
            
        # Check supported languages
        language_files = [
            "tree-sitter-rust", "tree-sitter-python", "tree-sitter-javascript",
            "tree-sitter-typescript", "tree-sitter-go", "tree-sitter-java",
            "tree-sitter-c", "tree-sitter-cpp", "tree-sitter-html",
            "tree-sitter-css", "tree-sitter-json", "tree-sitter-bash"
        ]
        
        for lang in language_files:
            if (self.project_root / "Cargo.toml").read_text().find(lang) != -1:
                results["languages_supported"].append(lang.replace("tree-sitter-", ""))
                
        # Calculate status
        successful_tests = sum(1 for t in results["tests"] if t["success"])
        results["status"] = "working" if successful_tests == len(results["tests"]) else "partial"
        results["success_rate"] = f"{successful_tests}/{len(results['tests'])}"
        
        return results
        
    def test_integration(self) -> Dict[str, Any]:
        """Test integration between all search methods"""
        print("\n🔗 Testing Integration...")
        results = {
            "name": "Integration Tests",
            "status": "unknown",
            "tests": [],
            "parallel_execution": {}
        }
        
        # Test 1: Unified search (all methods)
        start = time.time()
        code, stdout, stderr = self.run_command(
            ["cargo", "run", "--features", "full-system", "--", "search", "async"],
            timeout=60
        )
        elapsed = time.time() - start
        
        results["tests"].append({
            "name": "Unified search (all methods)",
            "query": "async",
            "success": code == 0,
            "execution_time": elapsed,
            "total_results": len(stdout.splitlines()) if stdout else 0,
            "error": stderr[:500] if code != 0 else None
        })
        
        # Test 2: Performance comparison
        queries = ["struct", "async fn", "Result", "impl", "test"]
        method_times = {}
        
        for query in queries:
            start = time.time()
            self.run_command(["cargo", "run", "--", "search", query])
            method_times[query] = time.time() - start
            
        results["parallel_execution"]["average_query_time"] = sum(method_times.values()) / len(method_times)
        results["parallel_execution"]["queries_tested"] = len(queries)
        
        # Test 3: Result fusion
        code, stdout, stderr = self.run_command(
            ["cargo", "test", "--features", "full-system", "fusion"]
        )
        
        results["tests"].append({
            "name": "Result fusion test",
            "success": code == 0,
            "fusion_working": "test result: ok" in stdout if stdout else False
        })
        
        # Calculate status
        successful_tests = sum(1 for t in results["tests"] if t["success"])
        results["status"] = "working" if successful_tests > 0 else "not_working"
        results["success_rate"] = f"{successful_tests}/{len(results['tests'])}"
        
        return results
        
    def generate_report(self):
        """Generate comprehensive analysis report"""
        print("\n" + "="*60)
        print("📊 COMPREHENSIVE SEARCH SYSTEM ANALYSIS")
        print("="*60)
        
        # Test all methods
        self.results["search_methods"]["ripgrep"] = self.test_ripgrep_search()
        self.results["search_methods"]["tantivy"] = self.test_tantivy_search()
        self.results["search_methods"]["vector_embedding"] = self.test_vector_embedding_search()
        self.results["search_methods"]["ast_symbol"] = self.test_ast_symbol_search()
        self.results["search_methods"]["integration"] = self.test_integration()
        
        # Summary statistics
        working_methods = sum(
            1 for m in self.results["search_methods"].values() 
            if m["status"] == "working"
        )
        partial_methods = sum(
            1 for m in self.results["search_methods"].values() 
            if m["status"] == "partial"
        )
        broken_methods = sum(
            1 for m in self.results["search_methods"].values() 
            if m["status"] in ["not_working", "broken"]
        )
        
        self.results["summary"] = {
            "total_methods": len(self.results["search_methods"]),
            "working": working_methods,
            "partial": partial_methods,
            "broken": broken_methods,
            "overall_health": "good" if working_methods >= 3 else "poor"
        }
        
        # Print summary
        print(f"\n✅ Working: {working_methods}/4 search methods")
        print(f"⚠️  Partial: {partial_methods}/4 search methods")
        print(f"❌ Broken: {broken_methods}/4 search methods")
        
        # Save JSON report
        report_path = self.project_root / "docs" / "search_analysis_report.json"
        report_path.parent.mkdir(exist_ok=True)
        with open(report_path, "w") as f:
            json.dump(self.results, f, indent=2)
        print(f"\n📄 Full report saved to: {report_path}")
        
        return self.results
        
if __name__ == "__main__":
    tester = SearchMethodTester()
    results = tester.generate_report()
    
    # Exit with appropriate code
    if results["summary"]["overall_health"] == "good":
        sys.exit(0)
    else:
        sys.exit(1)