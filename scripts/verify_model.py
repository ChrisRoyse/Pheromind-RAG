#!/usr/bin/env python3

"""
Verify GGUF model file and extract metadata
"""

import sys
import struct

def read_gguf_metadata(file_path):
    """Extract metadata from GGUF file"""
    try:
        with open(file_path, 'rb') as f:
            # Read GGUF magic
            magic = f.read(4)
            if magic != b'GGUF':
                print(f"❌ NOT A GGUF FILE: {magic}")
                return False
            
            # Read version
            version = struct.unpack('<I', f.read(4))[0]
            print(f"📋 GGUF Version: {version}")
            
            # Read tensor count
            tensor_count = struct.unpack('<Q', f.read(8))[0]
            print(f"📋 Tensor Count: {tensor_count}")
            
            # Read metadata count
            metadata_count = struct.unpack('<Q', f.read(8))[0]
            print(f"📋 Metadata Count: {metadata_count}")
            
            # Read some metadata
            for i in range(min(20, metadata_count)):
                # Read key length
                key_len = struct.unpack('<Q', f.read(8))[0]
                if key_len > 1000:  # Sanity check
                    break
                    
                # Read key
                key = f.read(key_len).decode('utf-8', errors='ignore')
                
                # Read value type
                value_type = struct.unpack('<I', f.read(4))[0]
                
                # Read value based on type (simplified)
                if value_type == 8:  # String
                    str_len = struct.unpack('<Q', f.read(8))[0]
                    if str_len < 1000:
                        value = f.read(str_len).decode('utf-8', errors='ignore')
                        print(f"   {key}: {value}")
                    else:
                        f.seek(str_len, 1)  # Skip
                elif value_type == 4:  # Uint32
                    value = struct.unpack('<I', f.read(4))[0]
                    print(f"   {key}: {value}")
                elif value_type == 6:  # Float32
                    value = struct.unpack('<f', f.read(4))[0]
                    print(f"   {key}: {value}")
                else:
                    # Skip unknown types
                    if value_type == 9:  # Array - skip for now
                        array_type = struct.unpack('<I', f.read(4))[0]
                        array_len = struct.unpack('<Q', f.read(8))[0]
                        # Skip array data (simplified)
                        break
                    else:
                        break
            
            return True
            
    except Exception as e:
        print(f"❌ Error reading GGUF: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python3 verify_model.py <gguf_file>")
        sys.exit(1)
    
    file_path = sys.argv[1]
    print(f"🔍 Analyzing: {file_path}")
    
    if read_gguf_metadata(file_path):
        print("✅ GGUF file is valid")
    else:
        print("❌ GGUF file analysis failed")