#!/bin/bash

# BM25 Stress Test Runner
# Runs comprehensive BM25 implementation validation

echo "================================================================"
echo "          BM25 STRESS TEST SUITE - FUNDAMENTAL FLAW DETECTION  "
echo "================================================================"
echo ""
echo "This suite tests 10 critical implementation flaws:"
echo "  1. Incremental Update Impossibility"
echo "  2. Tokenization Catastrophe"  
echo "  3. Memory Explosion"
echo "  4. Persistence Failure"
echo "  5. Length Bias Exposure"
echo "  6. Mathematical Edge Cases"
echo "  7. Unicode Tokenization Destruction"
echo "  8. Concurrency Corruption"
echo "  9. Stop Word Singularity"
echo "  10. Vocabulary Overflow"
echo ""
echo "Starting comprehensive stress test execution..."
echo ""

# Run the BM25 stress tests with full output
cargo test --test bm25_stress_tests --features core -- --nocapture

echo ""
echo "================================================================"
echo "                          ANALYSIS COMPLETE"
echo "================================================================"
echo ""
echo "For detailed analysis, see: docs/bm25_stress_test_results.md"
echo "For test documentation, see: docs/bm25_stress_test_documentation.md"
echo ""