#!/bin/bash

# Script to download the nomic-embed-code GGUF model
# This is a 4.3GB file that provides real semantic embeddings

MODEL_DIR="./src/model"
MODEL_FILE="nomic-embed-code.Q4_K_M.gguf"
MODEL_PATH="$MODEL_DIR/$MODEL_FILE"
MODEL_URL="https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_K_M.gguf"

echo "🔍 Checking for GGUF model..."

# Create model directory if it doesn't exist
mkdir -p "$MODEL_DIR"

# Check if model already exists
if [ -f "$MODEL_PATH" ]; then
    echo "✅ Model already exists: $MODEL_PATH"
    echo "📊 Size: $(du -h "$MODEL_PATH" | cut -f1)"
    exit 0
fi

echo "❌ Model not found. Downloading..."
echo "📦 URL: $MODEL_URL"
echo "💾 Target: $MODEL_PATH"
echo "⚠️  This is a 4.3GB download and will take time"

# Download with curl (with resume support)
echo "🔄 Starting download..."
curl -L -C - -o "$MODEL_PATH" "$MODEL_URL"

# Verify download
if [ -f "$MODEL_PATH" ]; then
    echo "✅ Download completed!"
    echo "📊 Downloaded size: $(du -h "$MODEL_PATH" | cut -f1)"
    
    # Basic file validation
    if [ $(stat -f%z "$MODEL_PATH" 2>/dev/null || stat -c%s "$MODEL_PATH" 2>/dev/null) -lt 1000000000 ]; then
        echo "❌ WARNING: File seems too small, download may be incomplete"
        exit 1
    fi
    
    echo "🎉 Model ready for use!"
else
    echo "❌ Download failed!"
    exit 1
fi