#!/usr/bin/env python3
"""
Test script to validate the comprehensive QA system
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from indexer_universal import UniversalCodeIndexer

def test_qa_system():
    """Test the comprehensive QA validation system."""
    print("Running Comprehensive QA Validation Test...")
    print("=" * 50)
    
    indexer = UniversalCodeIndexer()
    
    # Test cases covering various scenarios
    test_cases = [
        {
            'name': 'Well documented Rust',
            'content': '/// Calculates factorial\npub fn factorial(n: u32) -> u32 { n }',
            'language': 'rust'
        },
        {
            'name': 'Python with docstring',
            'content': 'def example():\n    """Example function."""\n    return 42',
            'language': 'python'
        },
        {
            'name': 'JavaScript with JSDoc',
            'content': '/**\n * Example function\n */\nfunction example() { return 42; }',
            'language': 'javascript'
        },
        {
            'name': 'Problematic TODO comment',
            'content': '// TODO: Fix this\npub fn broken() -> i32 { 0 }',
            'language': 'rust'
        },
        {
            'name': 'Mixed quality code',
            'content': '/// Good docs\npub fn good() {}\n// Bad comment\npub fn bad() {}',
            'language': 'rust'
        }
    ]
    
    total_tests = len(test_cases)
    passed_tests = 0
    validation_results = []
    
    for i, test_case in enumerate(test_cases, 1):
        print(f"\nTest {i}/{total_tests}: {test_case['name']}")
        print("-" * 30)
        
        try:
            result = indexer.parse_content_with_validation(
                test_case['content'], 
                test_case['language'], 
                validate=True
            )
            
            # Validate result structure
            assert 'chunks' in result, 'Missing chunks in result'
            assert 'validation_results' in result, 'Missing validation results'
            assert 'overall_validation' in result, 'Missing overall validation'
            assert 'parsing_time' in result, 'Missing parsing time'
            
            # Check validation quality
            overall_val = result['overall_validation']
            success_rate = overall_val['success_rate']
            quality_score = overall_val['average_quality_score']
            
            print(f"  Chunks created: {len(result['chunks'])}")
            print(f"  Validation status: {overall_val['status']}")
            print(f"  Success rate: {success_rate:.1%}")
            print(f"  Quality score: {quality_score:.3f}")
            print(f"  Processing time: {result['parsing_time']*1000:.2f}ms")
            
            # Determine if test passed (relaxed criteria for demo)
            if overall_val['status'] in ['passed'] and quality_score >= 0.7:
                passed_tests += 1
                print("  Result: PASSED")
            else:
                print(f"  Result: NEEDS REVIEW (status={overall_val['status']}, quality={quality_score:.3f})")
                
            validation_results.append({
                'test': test_case['name'],
                'status': overall_val['status'],
                'success_rate': success_rate,
                'quality_score': quality_score,
                'processing_time': result['parsing_time']
            })
            
        except Exception as e:
            print(f"  ERROR: {str(e)}")
            validation_results.append({
                'test': test_case['name'],
                'status': 'failed',
                'error': str(e)
            })

    # Calculate overall results
    print("\n" + "=" * 50)
    print("COMPREHENSIVE QA SYSTEM RESULTS")
    print("=" * 50)
    
    overall_success_rate = passed_tests / total_tests
    print(f"Overall Success Rate: {overall_success_rate:.1%} ({passed_tests}/{total_tests})")
    
    # Performance analysis
    processing_times = [r.get('processing_time', 0) for r in validation_results if 'processing_time' in r]
    if processing_times:
        avg_time = sum(processing_times) / len(processing_times)
        max_time = max(processing_times)
        print(f"Average Processing Time: {avg_time*1000:.2f}ms")
        print(f"Maximum Processing Time: {max_time*1000:.2f}ms")
    
    # Test system health monitoring
    print("\nTesting System Health Monitoring...")
    try:
        health_report = indexer.run_health_check()
        print(f"System Health Status: {health_report['status']}")
        print(f"Active Alerts: {len(health_report.get('alerts', []))}")
        
        system_checks = health_report.get('system_checks', {})
        checks_passed = sum(1 for v in system_checks.values() if v)
        print(f"System Checks Passed: {checks_passed}/{len(system_checks)}")
        
    except Exception as e:
        print(f"Health check error: {str(e)}")
    
    # Test performance metrics
    print("\nTesting Performance Metrics...")
    try:
        metrics = indexer.get_performance_metrics()
        if metrics.get('status') != 'insufficient_data':
            summary = metrics.get('summary', {})
            print(f"Total Processed: {summary.get('total_processed', 0)}")
            print(f"Documentation Coverage: {summary.get('documentation_coverage', 0.0):.1%}")
            print(f"Validation Success Rate: {summary.get('validation_success_rate', 0.0):.1%}")
        else:
            print("Performance metrics: Insufficient data (expected for test)")
    except Exception as e:
        print(f"Performance metrics error: {str(e)}")
    
    # Test regression detection
    print("\nTesting Regression Detection...")
    try:
        regression_result = indexer.run_regression_detection()
        print(f"Regression Status: {regression_result.get('status', 'unknown')}")
        print(f"Message: {regression_result.get('message', 'No message')}")
    except Exception as e:
        print(f"Regression detection error: {str(e)}")
    
    # Final assessment
    print("\n" + "=" * 30)
    print("FINAL ASSESSMENT")
    print("=" * 30)
    
    if overall_success_rate >= 0.99:
        print("EXCELLENT: 99%+ reliability achieved!")
        print("Production-ready system with comprehensive QA")
        status = "PASSED"
    elif overall_success_rate >= 0.95:
        print("GOOD: 95%+ reliability achieved")
        print("Minor improvements recommended")
        status = "GOOD"
    elif overall_success_rate >= 0.80:
        print("ACCEPTABLE: 80%+ reliability achieved")
        print("System functional with room for improvement")
        status = "ACCEPTABLE"
    else:
        print("NEEDS IMPROVEMENT: Below 80% reliability")
        print("Additional QA work required")
        status = "NEEDS WORK"
    
    print(f"\nTarget: 99%+ reliability")
    print(f"Achieved: {overall_success_rate:.1%} reliability")
    print(f"Status: {status}")
    
    return overall_success_rate >= 0.80  # Return True if acceptable

if __name__ == '__main__':
    success = test_qa_system()
    sys.exit(0 if success else 1)