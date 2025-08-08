#!/usr/bin/env python3
"""
Integration Test Verification Script
Verifies that integration tests are properly configured and can be discovered.
"""

import os
import subprocess
import sys
from pathlib import Path

def run_command(cmd, timeout=60):
    """Run a command and return its output."""
    try:
        result = subprocess.run(
            cmd, 
            shell=True, 
            capture_output=True, 
            text=True, 
            timeout=timeout,
            cwd=Path(__file__).parent.parent
        )
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "Command timed out"

def check_cargo_toml():
    """Check Cargo.toml for integration test configuration."""
    cargo_path = Path(__file__).parent.parent / "Cargo.toml"
    if not cargo_path.exists():
        return False, "Cargo.toml not found"
    
    with open(cargo_path, 'r') as f:
        content = f.read()
    
    # Check for feature flags
    has_full_system = "full-system" in content
    has_test_integration = "test-integration" in content
    
    return True, {
        "full_system_feature": has_full_system,
        "test_integration_feature": has_test_integration,
    }

def check_test_files():
    """Check for integration test files."""
    tests_dir = Path(__file__).parent.parent / "tests"
    if not tests_dir.exists():
        return False, "Tests directory not found"
    
    test_files = list(tests_dir.glob("*integration*.rs"))
    all_test_files = list(tests_dir.glob("*.rs"))
    
    return True, {
        "integration_test_files": [f.name for f in test_files],
        "total_test_files": len(all_test_files),
        "integration_count": len(test_files)
    }

def attempt_test_discovery():
    """Attempt to discover tests using various methods."""
    results = {}
    
    # Method 1: Basic test discovery
    code, stdout, stderr = run_command("cargo test --help", 10)
    if code == 0:
        results["cargo_available"] = True
    else:
        results["cargo_available"] = False
        results["cargo_error"] = stderr
    
    # Method 2: Try to list tests with features
    if results.get("cargo_available", False):
        code, stdout, stderr = run_command("cargo test --features full-system -- --list", 30)
        results["full_system_test_list"] = {
            "exit_code": code,
            "stdout_length": len(stdout),
            "stderr_length": len(stderr),
            "has_output": len(stdout) > 0 or len(stderr) > 0
        }
        
        if code != 0 and "permission denied" in stderr.lower():
            results["permission_issue"] = True
        
        # Method 3: Try without features for comparison
        code2, stdout2, stderr2 = run_command("cargo test -- --list", 30)
        results["no_features_test_list"] = {
            "exit_code": code2,
            "stdout_length": len(stdout2),
            "stderr_length": len(stderr2),
            "has_output": len(stdout2) > 0 or len(stderr2) > 0
        }
    
    return results

def main():
    """Main verification function."""
    print("=== Integration Test Verification ===\n")
    
    # Check 1: Cargo.toml configuration
    print("1. Checking Cargo.toml configuration...")
    success, config_info = check_cargo_toml()
    if success:
        print(f"   [OK] Configuration found: {config_info}")
    else:
        print(f"   [ERROR] Configuration error: {config_info}")
    print()
    
    # Check 2: Test files
    print("2. Checking test files...")
    success, test_info = check_test_files()
    if success:
        print(f"   [OK] Test files found: {test_info}")
    else:
        print(f"   [ERROR] Test files error: {test_info}")
    print()
    
    # Check 3: Test discovery
    print("3. Attempting test discovery...")
    discovery_results = attempt_test_discovery()
    print(f"   Discovery results: {discovery_results}")
    print()
    
    # Summary
    print("=== VERIFICATION SUMMARY ===")
    
    has_config = config_info.get("full_system_feature", False) if isinstance(config_info, dict) else False
    has_tests = test_info.get("integration_count", 0) > 0 if isinstance(test_info, dict) else False
    cargo_works = discovery_results.get("cargo_available", False)
    
    print(f"Feature flags configured: {'[OK]' if has_config else '[FAIL]'}")
    print(f"Integration test files found: {'[OK]' if has_tests else '[FAIL]'}")  
    print(f"Cargo commands work: {'[OK]' if cargo_works else '[FAIL]'}")
    
    if discovery_results.get("permission_issue", False):
        print("[WARNING] Permission/build issues detected")
    
    if has_config and has_tests and cargo_works:
        print("\n[SUCCESS] Integration tests appear to be properly configured!")
        return 0
    else:
        print(f"\n[FAILURE] Integration test configuration incomplete or non-functional")
        return 1

if __name__ == "__main__":
    sys.exit(main())