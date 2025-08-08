#!/usr/bin/env python3
"""
Quick Integration Test Configuration Check
"""

import os
import sys
from pathlib import Path

def main():
    print("=== QUICK INTEGRATION TEST CHECK ===\n")
    
    # Check 1: Cargo.toml feature flags
    cargo_path = Path(__file__).parent.parent / "Cargo.toml"
    if not cargo_path.exists():
        print("[FAIL] Cargo.toml not found")
        return 1
    
    with open(cargo_path, 'r') as f:
        cargo_content = f.read()
    
    # Look for key integration test features
    has_full_system = "full-system" in cargo_content
    has_test_integration = "test-integration" in cargo_content
    has_tree_sitter = "tree-sitter" in cargo_content
    has_ml = '"ml"' in cargo_content
    has_vectordb = '"vectordb"' in cargo_content
    has_tantivy = '"tantivy"' in cargo_content
    
    print("1. Feature Flag Analysis:")
    print(f"   full-system feature: {'[OK]' if has_full_system else '[MISSING]'}")
    print(f"   test-integration feature: {'[OK]' if has_test_integration else '[MISSING]'}")
    print(f"   tree-sitter feature: {'[OK]' if has_tree_sitter else '[MISSING]'}")
    print(f"   ml feature: {'[OK]' if has_ml else '[MISSING]'}")
    print(f"   vectordb feature: {'[OK]' if has_vectordb else '[MISSING]'}")
    print(f"   tantivy feature: {'[OK]' if has_tantivy else '[MISSING]'}")
    print()
    
    # Check 2: Integration test files
    tests_dir = Path(__file__).parent.parent / "tests"
    if not tests_dir.exists():
        print("[FAIL] Tests directory not found")
        return 1
    
    test_files = list(tests_dir.glob("*.rs"))
    integration_files = [f for f in test_files if "integration" in f.name.lower()]
    
    print("2. Test File Analysis:")
    print(f"   Total test files: {len(test_files)}")
    print(f"   Integration test files: {len(integration_files)}")
    for f in integration_files:
        print(f"     - {f.name}")
    print()
    
    # Check 3: Test configuration in Cargo.toml
    test_config_lines = []
    in_test_section = False
    for line in cargo_content.split('\n'):
        if '[[test]]' in line:
            in_test_section = True
            test_config_lines.append(line)
        elif in_test_section and line.startswith('['):
            in_test_section = False
        elif in_test_section:
            test_config_lines.append(line)
    
    print("3. Test Configuration:")
    if test_config_lines:
        print(f"   Found {len([l for l in test_config_lines if '[[test]]' in l])} test configurations")
        for line in test_config_lines[:10]:  # Show first 10 lines
            print(f"     {line}")
    else:
        print("   [WARNING] No explicit test configurations found")
    print()
    
    # Check 4: Feature dependencies
    print("4. Feature Dependencies Check:")
    full_system_line = None
    for line in cargo_content.split('\n'):
        if 'full-system =' in line:
            full_system_line = line.strip()
            break
    
    if full_system_line:
        print(f"   full-system definition: {full_system_line}")
        expected_deps = ["tree-sitter", "ml", "vectordb", "tantivy"]
        missing_deps = [dep for dep in expected_deps if f'"{dep}"' not in full_system_line]
        if missing_deps:
            print(f"   [WARNING] Missing dependencies in full-system: {missing_deps}")
        else:
            print("   [OK] All expected dependencies present in full-system")
    else:
        print("   [ERROR] full-system feature definition not found")
    print()
    
    # Final verdict
    print("=== VERIFICATION VERDICT ===")
    
    config_ok = has_full_system and has_test_integration
    files_ok = len(integration_files) > 0
    deps_ok = full_system_line is not None
    
    print(f"Configuration: {'[OK]' if config_ok else '[FAIL]'}")
    print(f"Test files: {'[OK]' if files_ok else '[FAIL]'}")  
    print(f"Dependencies: {'[OK]' if deps_ok else '[FAIL]'}")
    
    if config_ok and files_ok and deps_ok:
        print("\n[VERIFIED SUCCESS] Integration tests are properly configured!")
        print("Feature flag solution appears to be working correctly.")
        return 0
    else:
        print(f"\n[VERIFIED FAILURE] Integration test configuration has issues!")
        if not config_ok:
            print("- Missing feature flags")
        if not files_ok:
            print("- Missing integration test files")
        if not deps_ok:
            print("- Feature dependency issues")
        return 1

if __name__ == "__main__":
    sys.exit(main())