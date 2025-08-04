/**
 * TDD Tests for Cache Management Functionality
 * These tests MUST FAIL initially - we're implementing cache_manager.py to make them pass
 */

const path = require('path');
const fs = require('fs').promises;
const { spawn } = require('child_process');
const tmp = require('tmp');

// Set longer timeout for cache operations
jest.setTimeout(60000);

describe('Cache Management Functionality (TDD)', () => {
  let tempCacheDir;
  let cacheManagerScript;
  
  beforeAll(() => {
    cacheManagerScript = path.join(__dirname, '..', 'python', 'cache_manager.py');
  });
  
  beforeEach(async () => {
    // Create temporary cache directory
    tempCacheDir = tmp.dirSync({ unsafeCleanup: true });
  });
  
  afterEach(() => {
    if (tempCacheDir) {
      tempCacheDir.removeCallback();
    }
  });

  describe('Cache Manager Module Existence', () => {
    test('MUST FAIL: cache_manager.py should exist', async () => {
      const exists = await fileExists(cacheManagerScript);
      expect(exists).toBe(true);
    });

    test('MUST FAIL: cache manager should have CLI interface', async () => {
      const result = await runPythonScript([cacheManagerScript, '--help']);
      expect(result.code).toBe(0);
      expect(result.stdout).toContain('usage');
      expect(result.stdout).toContain('cache');
    });
  });

  describe('Cache Initialization and Structure', () => {
    test('MUST FAIL: should initialize cache directory structure', async () => {
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--init'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
      expect(output.cache_path).toBeDefined();
      
      // Should create subdirectories
      const embeddingsDir = path.join(tempCacheDir.name, 'embeddings');
      const modelsDir = path.join(tempCacheDir.name, 'models');
      const indexesDir = path.join(tempCacheDir.name, 'indexes');
      
      expect(await fileExists(embeddingsDir)).toBe(true);
      expect(await fileExists(modelsDir)).toBe(true);
      expect(await fileExists(indexesDir)).toBe(true);
      
      // Should create metadata file
      const metadataFile = path.join(tempCacheDir.name, 'cache_metadata.json');
      expect(await fileExists(metadataFile)).toBe(true);
    });

    test('MUST FAIL: should create cache metadata with correct structure', async () => {
      await runPythonScript([cacheManagerScript, '--cache-path', tempCacheDir.name, '--init']);
      
      const metadataFile = path.join(tempCacheDir.name, 'cache_metadata.json');
      const content = await fs.readFile(metadataFile, 'utf-8');
      const metadata = JSON.parse(content);
      
      expect(metadata).toHaveProperty('version');
      expect(metadata).toHaveProperty('created_at');
      expect(metadata).toHaveProperty('last_updated');
      expect(metadata).toHaveProperty('cache_size');
      expect(metadata).toHaveProperty('entries');
      expect(Array.isArray(metadata.entries)).toBe(true);
    });
  });

  describe('Embedding Cache Management', () => {
    beforeEach(async () => {
      // Initialize cache
      await runPythonScript([cacheManagerScript, '--cache-path', tempCacheDir.name, '--init']);
    });

    test('MUST FAIL: should store embeddings in cache', async () => {
      const testData = {
        project: 'test-project',
        file: 'main.py',
        chunk_hash: 'abc123',
        embedding: [0.1, 0.2, 0.3, 0.4, 0.5]
      };
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-embedding',
        '--data', JSON.stringify(testData)
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
      expect(output.cache_key).toBeDefined();
      
      // Should create embedding file
      const embeddingFile = path.join(tempCacheDir.name, 'embeddings', `${testData.chunk_hash}.npy`);
      expect(await fileExists(embeddingFile)).toBe(true);
    });

    test('MUST FAIL: should retrieve embeddings from cache', async () => {
      // First store an embedding
      const testData = {
        project: 'test-project',
        file: 'main.py',
        chunk_hash: 'def456',
        embedding: [0.9, 0.8, 0.7, 0.6, 0.5]
      };
      
      await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-embedding',
        '--data', JSON.stringify(testData)
      ]);
      
      // Then retrieve it
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--get-embedding',
        '--chunk-hash', testData.chunk_hash
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
      expect(output.embedding).toBeDefined();
      expect(Array.isArray(output.embedding)).toBe(true);
      expect(output.embedding).toEqual(testData.embedding);
    });

    test('MUST FAIL: should handle missing embeddings gracefully', async () => {
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--get-embedding',
        '--chunk-hash', 'nonexistent'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(false);
      expect(output.error).toContain('not found');
    });
  });

  describe('Model Cache Management', () => {
    beforeEach(async () => {
      await runPythonScript([cacheManagerScript, '--cache-path', tempCacheDir.name, '--init']);
    });

    test('MUST FAIL: should store model files in cache', async () => {
      // Create a fake model file
      const modelData = 'fake model content';
      const modelFile = path.join(tempCacheDir.name, 'temp_model.bin');
      await fs.writeFile(modelFile, modelData);
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-model',
        '--model-name', 'test-model',
        '--model-file', modelFile
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
      expect(output.model_path).toBeDefined();
      
      // Should create model in cache
      const cachedModelFile = path.join(tempCacheDir.name, 'models', 'test-model.bin');
      expect(await fileExists(cachedModelFile)).toBe(true);
      
      const cachedContent = await fs.readFile(cachedModelFile, 'utf-8');
      expect(cachedContent).toBe(modelData);
    });

    test('MUST FAIL: should list cached models', async () => {
      // Store a model first
      const modelData = 'model data';
      const modelFile = path.join(tempCacheDir.name, 'temp_model.bin');
      await fs.writeFile(modelFile, modelData);
      
      await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-model',
        '--model-name', 'test-model',
        '--model-file', modelFile
      ]);
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--list-models'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.models).toBeDefined();
      expect(Array.isArray(output.models)).toBe(true);
      expect(output.models.length).toBeGreaterThan(0);
      
      const model = output.models.find(m => m.name === 'test-model');
      expect(model).toBeDefined();
      expect(model).toHaveProperty('size');
      expect(model).toHaveProperty('cached_at');
    });
  });

  describe('Index Cache Management', () => {
    beforeEach(async () => {
      await runPythonScript([cacheManagerScript, '--cache-path', tempCacheDir.name, '--init']);
    });

    test('MUST FAIL: should cache project indexes', async () => {
      // Create a fake index
      const indexData = { chunks: 100, files: 10 };
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-index',
        '--project-name', 'test-project',
        '--index-data', JSON.stringify(indexData)
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
      
      // Should create index file
      const indexFile = path.join(tempCacheDir.name, 'indexes', 'test-project.json');
      expect(await fileExists(indexFile)).toBe(true);
      
      const cachedIndex = JSON.parse(await fs.readFile(indexFile, 'utf-8'));
      expect(cachedIndex.chunks).toBe(100);
      expect(cachedIndex.files).toBe(10);
    });

    test('MUST FAIL: should retrieve cached indexes', async () => {
      // Store an index first
      const indexData = { chunks: 50, files: 5, version: '1.0' };
      
      await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-index',
        '--project-name', 'retrieve-test',
        '--index-data', JSON.stringify(indexData)
      ]);
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--get-index',
        '--project-name', 'retrieve-test'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
      expect(output.index_data).toBeDefined();
      expect(output.index_data.chunks).toBe(50);
      expect(output.index_data.files).toBe(5);
    });
  });

  describe('Cache Statistics and Management', () => {
    beforeEach(async () => {
      await runPythonScript([cacheManagerScript, '--cache-path', tempCacheDir.name, '--init']);
    });

    test('MUST FAIL: should provide cache statistics', async () => {
      // Add some data to cache
      const embedding = { chunk_hash: 'stat1', embedding: [0.1, 0.2] };
      await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-embedding',
        '--data', JSON.stringify(embedding)
      ]);
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--stats'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output).toHaveProperty('total_size');
      expect(output).toHaveProperty('embeddings_count');
      expect(output).toHaveProperty('models_count');
      expect(output).toHaveProperty('indexes_count');
      expect(output.embeddings_count).toBeGreaterThan(0);
    });

    test('MUST FAIL: should clean expired cache entries', async () => {
      // Store an embedding
      const embedding = { chunk_hash: 'expire1', embedding: [0.1] };
      await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-embedding',
        '--data', JSON.stringify(embedding)
      ]);
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--cleanup',
        '--max-age', '0'  // Everything is expired
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
      expect(output.cleaned_count).toBeGreaterThan(0);
    });

    test('MUST FAIL: should limit cache size by removing old entries', async () => {
      // Store multiple embeddings
      for (let i = 0; i < 5; i++) {
        const embedding = { chunk_hash: `size${i}`, embedding: [i] };
        await runPythonScript([
          cacheManagerScript,
          '--cache-path', tempCacheDir.name,
          '--store-embedding',
          '--data', JSON.stringify(embedding)
        ]);
      }
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--enforce-size-limit',
        '--max-size', '100'  // Very small limit in bytes
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
      expect(output).toHaveProperty('removed_count');
    });
  });

  describe('Cache Validation and Integrity', () => {
    beforeEach(async () => {
      await runPythonScript([cacheManagerScript, '--cache-path', tempCacheDir.name, '--init']);
    });

    test('MUST FAIL: should validate cache integrity', async () => {
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--validate'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.valid).toBe(true);
      expect(output).toHaveProperty('checks_performed');
      expect(output).toHaveProperty('issues_found');
      expect(Array.isArray(output.checks_performed)).toBe(true);
    });

    test('MUST FAIL: should detect corrupted cache files', async () => {
      // Store an embedding first
      const embedding = { chunk_hash: 'corrupt1', embedding: [0.1] };
      await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-embedding',
        '--data', JSON.stringify(embedding)
      ]);
      
      // Corrupt the embedding file
      const embeddingFile = path.join(tempCacheDir.name, 'embeddings', 'corrupt1.npy');
      await fs.writeFile(embeddingFile, 'corrupted data');
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--validate'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.valid).toBe(false);
      expect(output.issues_found).toBeGreaterThan(0);
    });

    test('MUST FAIL: should repair corrupted cache metadata', async () => {
      // Corrupt the metadata file
      const metadataFile = path.join(tempCacheDir.name, 'cache_metadata.json');
      await fs.writeFile(metadataFile, 'invalid json');
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--repair'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
      expect(output.repairs_made).toBeGreaterThan(0);
      
      // Metadata should be valid again
      const repairedContent = await fs.readFile(metadataFile, 'utf-8');
      expect(() => JSON.parse(repairedContent)).not.toThrow();
    });
  });

  describe('Performance and Concurrency', () => {
    beforeEach(async () => {
      await runPythonScript([cacheManagerScript, '--cache-path', tempCacheDir.name, '--init']);
    });

    test('MUST FAIL: should handle concurrent cache operations', async () => {
      // Start multiple cache operations concurrently
      const promises = [];
      
      for (let i = 0; i < 5; i++) {
        const embedding = { chunk_hash: `concurrent${i}`, embedding: [i] };
        const promise = runPythonScript([
          cacheManagerScript,
          '--cache-path', tempCacheDir.name,
          '--store-embedding',
          '--data', JSON.stringify(embedding)
        ]);
        promises.push(promise);
      }
      
      const results = await Promise.all(promises);
      
      // All operations should succeed
      for (const result of results) {
        expect(result.code).toBe(0);
        const output = JSON.parse(result.stdout);
        expect(output.success).toBe(true);
      }
      
      // All embeddings should be stored
      const statsResult = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--stats'
      ]);
      
      const stats = JSON.parse(statsResult.stdout);
      expect(stats.embeddings_count).toBe(5);
    });

    test('MUST FAIL: should handle large cache operations efficiently', async () => {
      // Create large embedding
      const largeEmbedding = Array.from({length: 1000}, (_, i) => i * 0.001);
      const embedding = { chunk_hash: 'large1', embedding: largeEmbedding };
      
      const startTime = Date.now();
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--store-embedding',
        '--data', JSON.stringify(embedding)
      ]);
      
      const duration = Date.now() - startTime;
      
      expect(result.code).toBe(0);
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
    });
  });

  describe('Error Handling', () => {
    test('MUST FAIL: should handle invalid cache directory', async () => {
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', '"/invalid/path"',
        '--stats'
      ]);
      
      expect(result.code).toBe(1);
      expect(result.stderr).toContain('cache directory');
    });

    test('MUST FAIL: should handle disk full scenarios', async () => {
      // This is hard to test reliably, so we'll simulate by setting a very small limit
      await runPythonScript([cacheManagerScript, '--cache-path', tempCacheDir.name, '--init']);
      
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--enforce-size-limit',
        '--max-size', '1'  // 1 byte limit
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.success).toBe(true);
    });
  });

  describe('Output Formats', () => {
    beforeEach(async () => {
      await runPythonScript([cacheManagerScript, '--cache-path', tempCacheDir.name, '--init']);
    });

    test('MUST FAIL: should support JSON output format', async () => {
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--stats',
        '--format', 'json'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output).toHaveProperty('total_size');
    });

    test('MUST FAIL: should support plain text output format', async () => {
      const result = await runPythonScript([
        cacheManagerScript,
        '--cache-path', tempCacheDir.name,
        '--stats',
        '--format', 'text'
      ]);
      
      expect(result.code).toBe(0);
      
      // Should be plain text (not JSON)
      expect(() => JSON.parse(result.stdout)).toThrow();
      expect(result.stdout).toContain('Cache Statistics');
    });
  });
});

// Helper Functions
async function fileExists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function runPythonScript(args, options = {}) {
  return new Promise((resolve, reject) => {
    // Use bundled Python runtime
    const { PlatformManager } = require('../lib/platform');
    const pythonExe = PlatformManager.getPythonExecutable();
    
    const child = spawn(pythonExe, args, {
      stdio: 'pipe',
      timeout: 30000,
      ...options
    });

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    child.on('exit', (code) => {
      resolve({ code, stdout, stderr });
    });

    child.on('error', reject);

    setTimeout(() => {
      child.kill();
      reject(new Error('Script timeout'));
    }, 30000);
  });
}