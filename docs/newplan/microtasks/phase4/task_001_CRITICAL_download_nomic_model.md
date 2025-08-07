# Task 001 - CRITICAL: Download Nomic Embed Text Model

## Priority: CRITICAL
## Estimated Time: 10 minutes
## Dependencies: None

## Objective
Download the Nomic embed-text-v1.5 model in GGUF format to enable embedding generation.

## Current Issue
- Model file missing or incorrect path
- Embedding generation fails without proper model
- Need GGUF format for Candle compatibility

## Tasks
1. **Download model file** (5 min)
   ```bash
   # Create models directory
   mkdir -p models
   
   # Download Nomic embed-text-v1.5 GGUF model
   curl -L -o models/nomic-embed-text-v1.5-Q4_K_M.gguf \
     "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5-Q4_K_M.gguf"
   ```

2. **Verify download** (3 min)
   ```bash
   # Check file exists and size
   ls -la models/nomic-embed-text-v1.5-Q4_K_M.gguf
   
   # Should be ~267MB
   du -h models/nomic-embed-text-v1.5-Q4_K_M.gguf
   ```

3. **Update environment** (2 min)
   - Set MODEL_PATH environment variable
   - Document model location in config

## Success Criteria
- [ ] Model file downloaded (267MB)
- [ ] File integrity verified
- [ ] Path configured correctly
- [ ] No download corruption

## Files to Modify
- `models/` directory (create)
- Environment configuration

## Validation
```bash
# Verify GGUF magic number
hexdump -C models/nomic-embed-text-v1.5-Q4_K_M.gguf | head -1
# Should show: GGUF magic bytes
```

## Next Task
→ Task 002: Verify model file format and metadata