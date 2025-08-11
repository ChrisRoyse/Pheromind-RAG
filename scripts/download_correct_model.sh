#!/bin/bash

# Script to download the CORRECT nomic-embed-code embedding model
# The current model is Qwen2.5-Coder (generative), not an embedding model

MODEL_DIR="./src/model"
OLD_FILE="nomic-embed-code.Q4_K_M.gguf"
NEW_FILE="nomic-embed-text-v1.5.Q4_K_M.gguf"
MODEL_URL="https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_K_M.gguf"

echo "🚨 CRITICAL: Current model is WRONG!"
echo "   Current: Qwen2.5-Coder 7B (text generation)"
echo "   Need: nomic-embed-text-v1.5 (embeddings)"

# Backup the wrong model
if [ -f "$MODEL_DIR/$OLD_FILE" ]; then
    echo "📦 Backing up incorrect model..."
    mv "$MODEL_DIR/$OLD_FILE" "$MODEL_DIR/$OLD_FILE.backup"
fi

echo "🔄 Downloading CORRECT embedding model..."
echo "📦 URL: $MODEL_URL"
echo "💾 Target: $MODEL_DIR/$NEW_FILE"

# Download the correct model
curl -L -C - -o "$MODEL_DIR/$NEW_FILE" "$MODEL_URL"

# Update symlink to point to correct model
cd "$MODEL_DIR"
ln -sf "$NEW_FILE" "nomic-embed-code.Q4_K_M.gguf"

echo "✅ Downloaded correct embedding model!"
echo "📊 Size: $(du -h "$MODEL_DIR/$NEW_FILE" | cut -f1)"

# Verify it's actually an embedding model
python3 ../scripts/verify_model.py "$MODEL_DIR/$NEW_FILE"