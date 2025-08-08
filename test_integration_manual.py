#!/usr/bin/env python3
"""
Manual integration test to analyze system flow without feature compilation issues.
This script traces the actual integration pathways in the embed-search system.
"""

import subprocess
import json
import os
from pathlib import Path

def analyze_cargo_features():
    """Analyze Cargo.toml to understand feature configuration"""
    print("🔧 ANALYZING CARGO FEATURES...")
    
    cargo_path = Path("Cargo.toml")
    if not cargo_path.exists():
        print("❌ Cargo.toml not found")
        return {}
    
    with open(cargo_path) as f:
        content = f.read()
    
    # Extract feature definitions
    features_section = False
    features = {}
    for line in content.split('\n'):
        if line.strip() == '[features]':
            features_section = True
            continue
        if features_section and line.startswith('['):
            break
        if features_section and '=' in line:
            key, value = line.split('=', 1)
            features[key.strip()] = value.strip().strip('"')
    
    print(f"✅ Found {len(features)} features:")
    for name, deps in features.items():
        print(f"   - {name}: {deps}")
    
    return features

def check_integration_dependencies():
    """Check which integration components are available"""
    print("\n🔍 CHECKING INTEGRATION DEPENDENCIES...")
    
    # Check for key files that indicate component availability
    components = {
        'UnifiedSearcher': 'src/search/unified.rs',
        'BM25Engine': 'src/search/bm25.rs', 
        'Fusion': 'src/search/fusion.rs',
        'Config': 'src/config/mod.rs',
        'SymbolIndexer': 'src/search/symbol_index.rs',
        'LanceDBStorage': 'src/storage/lancedb_storage.rs',
        'NomicEmbedder': 'src/embedding/nomic.rs',
        'TextSearcher': 'src/search/search_adapter.rs'
    }
    
    available = {}
    for component, file_path in components.items():
        if Path(file_path).exists():
            available[component] = True
            print(f"   ✅ {component} - Found at {file_path}")
        else:
            available[component] = False
            print(f"   ❌ {component} - Missing at {file_path}")
    
    return available

def trace_integration_flow():
    """Trace the integration flow through UnifiedSearcher"""
    print("\n🔄 TRACING INTEGRATION FLOW...")
    
    unified_path = Path("src/search/unified.rs")
    if not unified_path.exists():
        print("❌ UnifiedSearcher not found")
        return {}
    
    with open(unified_path) as f:
        content = f.read()
    
    # Find key integration points
    integration_points = {}
    
    # Check for async search method
    if 'async fn search(' in content:
        integration_points['main_search'] = True
        print("   ✅ Main async search method found")
    else:
        integration_points['main_search'] = False
        print("   ❌ Main async search method missing")
    
    # Check for component integration
    components = ['search_exact', 'search_semantic', 'search_symbols', 'search_bm25']
    for component in components:
        if f'async fn {component}(' in content:
            integration_points[component] = True
            print(f"   ✅ {component} method found")
        else:
            integration_points[component] = False
            print(f"   ❌ {component} method missing")
    
    # Check for feature gates
    feature_gates = content.count('#[cfg(feature')
    integration_points['feature_gates'] = feature_gates
    print(f"   📋 Found {feature_gates} feature gates")
    
    # Check for error handling
    if 'Result<' in content and 'anyhow::anyhow!' in content:
        integration_points['error_handling'] = True
        print("   ✅ Error handling implemented")
    else:
        integration_points['error_handling'] = False
        print("   ❌ Error handling missing")
    
    return integration_points

def check_test_integration():
    """Check integration test configuration"""
    print("\n🧪 CHECKING INTEGRATION TESTS...")
    
    tests_path = Path("tests")
    if not tests_path.exists():
        print("❌ Tests directory not found")
        return {}
    
    integration_tests = {}
    for test_file in tests_path.glob("*.rs"):
        print(f"   📝 Found test file: {test_file.name}")
        
        with open(test_file) as f:
            content = f.read()
        
        # Check for UnifiedSearcher usage
        if 'UnifiedSearcher' in content:
            integration_tests[test_file.stem] = {
                'uses_unified_searcher': True,
                'has_async_tests': '#[tokio::test]' in content,
                'has_feature_gates': '#[cfg(feature' in content,
                'test_count': content.count('#[tokio::test]') + content.count('#[test]')
            }
            print(f"     ✅ {test_file.stem} - Uses UnifiedSearcher")
        else:
            integration_tests[test_file.stem] = {'uses_unified_searcher': False}
            print(f"     ⏭️  {test_file.stem} - No UnifiedSearcher usage")
    
    return integration_tests

def analyze_broken_connections():
    """Analyze potential broken connections in the integration"""
    print("\n💔 ANALYZING BROKEN CONNECTIONS...")
    
    broken_connections = []
    
    # Check UnifiedSearcher implementation
    unified_path = Path("src/search/unified.rs")
    if unified_path.exists():
        with open(unified_path) as f:
            content = f.read()
        
        # Check for unused methods (dead code warnings indicate broken integration)
        if 'methods `search_bm25`' in content or 'search_bm25` are never used' in content:
            broken_connections.append("BM25 search method is implemented but never called")
        
        # Check for incomplete feature integration
        if '#[cfg(not(all(feature' in content:
            broken_connections.append("Feature gates preventing complete integration")
            
        # Check for unused fusion component
        if 'fusion: SimpleFusion' in content and 'fusion` is never read' in content:
            broken_connections.append("Fusion component is initialized but never used")
    
    # Check for missing implementations
    if not Path("src/search/bm25.rs").exists():
        broken_connections.append("BM25Engine referenced but implementation missing")
    
    if broken_connections:
        for connection in broken_connections:
            print(f"   💔 {connection}")
    else:
        print("   ✅ No obvious broken connections detected")
    
    return broken_connections

def main():
    print("=" * 80)
    print("EMBED-SEARCH INTEGRATION FLOW ANALYSIS")
    print("=" * 80)
    
    # Analyze system components
    features = analyze_cargo_features()
    components = check_integration_dependencies() 
    flow = trace_integration_flow()
    tests = check_test_integration()
    broken = analyze_broken_connections()
    
    # Generate summary report
    print("\n" + "=" * 80)
    print("INTEGRATION ANALYSIS SUMMARY")
    print("=" * 80)
    
    total_components = len(components)
    available_components = sum(1 for v in components.values() if v)
    
    print(f"📊 Component Availability: {available_components}/{total_components} components available")
    print(f"🔧 Feature Configuration: {len(features)} features defined")
    print(f"🔄 Integration Flow: {sum(1 for v in flow.values() if v == True)} integration points working")
    print(f"🧪 Integration Tests: {len(tests)} test files found")
    print(f"💔 Broken Connections: {len(broken)} issues detected")
    
    if broken:
        print("\n🚨 CRITICAL INTEGRATION ISSUES:")
        for issue in broken:
            print(f"   - {issue}")
    
    if available_components < total_components:
        print("\n⚠️  MISSING COMPONENTS:")
        for name, available in components.items():
            if not available:
                print(f"   - {name}")
    
    print("\n✅ ANALYSIS COMPLETE")

if __name__ == "__main__":
    main()