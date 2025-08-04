#!/usr/bin/env python3
"""Debug script to test the indexer directly"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

from indexer_universal import UniversalCodeIndexer

def test_indexer():
    print("Creating indexer...")
    try:
        indexer = UniversalCodeIndexer()
        print("Indexer created successfully")
    except Exception as e:
        print(f"Error creating indexer: {e}")
        import traceback
        traceback.print_exc()
        return
    
    content = """/// Calculates the spike timing for a cortical column using TTFS encoding.
/// 
/// This function implements the time-to-first-spike algorithm for neural processing.
/// The calculation takes into account membrane potential, threshold dynamics, and
/// temporal encoding patterns typical of cortical columns.
/// 
/// # Arguments
/// 
/// * `membrane_voltage` - Current membrane voltage in millivolts
/// * `spike_threshold` - Threshold voltage for spike generation
/// * `time_constant` - Membrane time constant in milliseconds
/// 
/// # Returns
/// 
/// Returns the time to first spike in milliseconds, or None if no spike occurs
/// within the simulation window.
/// 
/// # Examples
/// 
/// ```rust
/// let spike_time = calculate_spike_timing(65.0, 70.0, 10.0);
/// assert!(spike_time.is_some());
/// ```
pub fn calculate_spike_timing(
    membrane_voltage: f64, 
    spike_threshold: f64, 
    time_constant: f64
) -> Option<f64> {
    if membrane_voltage >= spike_threshold {
        return Some(0.0);
    }
    Some((spike_threshold - membrane_voltage) / time_constant)
}"""
    
    print("Parsing content...")
    try:
        # Add debug step
        print("Content to parse:")
        print(repr(content))
        print("Language: rust")
        
        result = indexer.parse_content(content, 'rust', 'test.rs')
        print(f"Result: {result}")
        print(f"Number of chunks: {len(result)}")
        
        if result:
            for i, chunk in enumerate(result):
                print(f"Chunk {i}: {chunk}")
        else:
            print("No chunks returned!")
            
            # Let's debug step by step
            print("\nDebug: Testing chunking engine directly...")
            lines = content.split('\n')
            print(f"Lines: {lines}")
            
            # Test the SmartChunkingEngine directly
            chunking_engine = indexer.chunking_engine
            logical_units = chunking_engine._find_logical_units(lines, 'rust')
            print(f"Logical units found: {logical_units}")
            
    except Exception as e:
        print(f"Error during parsing: {e}")
        import traceback
        traceback.print_exc()

if __name__ == '__main__':
    test_indexer()