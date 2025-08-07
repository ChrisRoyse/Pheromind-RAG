#!/bin/bash

# Verification script to ensure no fallbacks or workarounds exist in the codebase
# All code must either work correctly or fail with proper error messages

echo "=== Fallback/Workaround Verification Script ==="
echo "Checking for patterns that indicate fallbacks or error swallowing..."
echo ""

FOUND_ISSUES=0

# Check for unwrap_or with default values (except in tests and logging)
echo "Checking for unwrap_or with defaults..."
if grep -r "unwrap_or(0\|unwrap_or(false\|unwrap_or(true\|unwrap_or(\"\"" src/ --include="*.rs" | grep -v "// OK:" | grep -v "test"; then
    echo "❌ Found unwrap_or with default values (potential fallbacks)"
    FOUND_ISSUES=1
else
    echo "✅ No problematic unwrap_or patterns found"
fi
echo ""

# Check for .ok() that might swallow errors (except in tests)
echo "Checking for .ok() that might swallow errors..."
if grep -r "\.ok()" src/ --include="*.rs" | grep -v "ok_or" | grep -v "// OK:" | grep -v "test"; then
    echo "❌ Found .ok() calls that might be swallowing errors"
    FOUND_ISSUES=1
else
    echo "✅ No problematic .ok() patterns found"
fi
echo ""

# Check for let _ = patterns that ignore results
echo "Checking for let _ = patterns that ignore results..."
if grep -r "let _ =" src/ --include="*.rs" | grep -v "// OK:" | grep -v "test" | grep -v "mod tests"; then
    echo "❌ Found let _ = patterns that might be ignoring errors"
    FOUND_ISSUES=1
else
    echo "✅ No problematic let _ = patterns found"
fi
echo ""

# Check for fallback/workaround/placeholder/simulate keywords in comments
echo "Checking for fallback/workaround keywords in implementation code..."
if grep -ri "fallback\|workaround\|placeholder\|simulate\|mock\|stub\|dummy\|fake" src/ --include="*.rs" | grep -v "// No fallback" | grep -v "// Require" | grep -v "test" | grep -v "// OK:"; then
    echo "⚠️  Found potential fallback/workaround references (review needed)"
    # Not setting FOUND_ISSUES here as these might be legitimate comments
fi
echo ""

# Check for Default::default() usage (could indicate fallback values)
echo "Checking for Default::default() usage..."
if grep -r "Default::default()" src/ --include="*.rs" | grep -v "// OK:" | grep -v "test"; then
    echo "⚠️  Found Default::default() usage (review if these are fallbacks)"
fi
echo ""

# Check for if-let patterns that might be hiding errors
echo "Checking for potentially problematic if-let Ok patterns..."
if grep -r "if let Ok(_)" src/ --include="*.rs" | grep -v "// OK:" | grep -v "test"; then
    echo "⚠️  Found if let Ok(_) patterns that might be hiding errors"
fi
echo ""

# Summary
echo "=== Verification Summary ==="
if [ $FOUND_ISSUES -eq 0 ]; then
    echo "✅ No critical fallback patterns detected!"
    echo "All code should either work correctly or fail with proper error messages."
else
    echo "❌ Found fallback patterns that need to be addressed!"
    echo "Please review and fix the issues above."
    exit 1
fi