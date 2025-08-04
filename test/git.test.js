/**
 * TDD Tests for Git Integration Functionality
 * These tests MUST FAIL initially - we're implementing git_tracker.py to make them pass
 */

const path = require('path');
const fs = require('fs').promises;
const { spawn } = require('child_process');
const tmp = require('tmp');

// Set longer timeout for git operations
jest.setTimeout(60000);

describe('Git Integration Functionality (TDD)', () => {
  let tempProjectDir;
  let gitTrackerScript;
  let indexPath;
  
  beforeAll(() => {
    gitTrackerScript = path.join(__dirname, '..', 'python', 'git_tracker.py');
  });
  
  beforeEach(async () => {
    // Create temporary test project with git
    tempProjectDir = tmp.dirSync({ unsafeCleanup: true });
    indexPath = path.join(tempProjectDir.name, '.mcp_index');
    
    // Initialize git repository
    await runGitCommand(['init'], tempProjectDir.name);
    await runGitCommand(['config', 'user.email', 'test@example.com'], tempProjectDir.name);
    await runGitCommand(['config', 'user.name', 'Test User'], tempProjectDir.name);
    
    // Create test files and make initial commit
    await createTestProject(tempProjectDir.name);
    await runGitCommand(['add', '.'], tempProjectDir.name);
    await runGitCommand(['commit', '-m', 'Initial commit'], tempProjectDir.name);
  });
  
  afterEach(() => {
    if (tempProjectDir) {
      tempProjectDir.removeCallback();
    }
  });

  describe('Git Tracker Module Existence', () => {
    test('MUST FAIL: git_tracker.py should exist', async () => {
      const exists = await fileExists(gitTrackerScript);
      expect(exists).toBe(true);
    });

    test('MUST FAIL: git tracker should have CLI interface', async () => {
      const result = await runPythonScript([gitTrackerScript, '--help']);
      expect(result.code).toBe(0);
      expect(result.stdout).toContain('usage');
      expect(result.stdout).toContain('git');
    });
  });

  describe('Git Status and Change Detection', () => {
    test('MUST FAIL: should detect git repository', async () => {
      const result = await runPythonScript([
        gitTrackerScript,
        '--check-repo',
        tempProjectDir.name
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.is_git_repo).toBe(true);
      expect(output.git_root).toBeDefined();
      expect(output.current_branch).toBeDefined();
    });

    test('MUST FAIL: should detect changed files since last commit', async () => {
      // Modify a file
      await fs.writeFile(
        path.join(tempProjectDir.name, 'main.py'),
        '# Modified file\nprint("Hello World")\n'
      );
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--status',
        tempProjectDir.name
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.modified_files).toBeDefined();
      expect(Array.isArray(output.modified_files)).toBe(true);
      expect(output.modified_files.length).toBeGreaterThan(0);
      
      // Should find the modified main.py
      const filenames = output.modified_files.map(f => path.basename(f));
      expect(filenames).toContain('main.py');
    });

    test('MUST FAIL: should detect newly added files', async () => {
      // Add a new file
      await fs.writeFile(
        path.join(tempProjectDir.name, 'new_file.py'),
        'def new_function():\n    return "new"'
      );
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--status',
        tempProjectDir.name
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.untracked_files).toBeDefined();
      expect(Array.isArray(output.untracked_files)).toBe(true);
      expect(output.untracked_files.length).toBeGreaterThan(0);
      
      // Should find the new file
      const filenames = output.untracked_files.map(f => path.basename(f));
      expect(filenames).toContain('new_file.py');
    });
  });

  describe('Commit History and Diffs', () => {
    test('MUST FAIL: should get commit history', async () => {
      // Make another commit
      await fs.writeFile(
        path.join(tempProjectDir.name, 'feature.py'),
        'def feature():\n    return "feature"'
      );
      await runGitCommand(['add', 'feature.py'], tempProjectDir.name);
      await runGitCommand(['commit', '-m', 'Add feature'], tempProjectDir.name);
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--history',
        tempProjectDir.name,
        '--limit', '5'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.commits).toBeDefined();
      expect(Array.isArray(output.commits)).toBe(true);
      expect(output.commits.length).toBeGreaterThan(0);
      
      // Should have commit info
      const commit = output.commits[0];
      expect(commit).toHaveProperty('hash');
      expect(commit).toHaveProperty('message');
      expect(commit).toHaveProperty('author');
      expect(commit).toHaveProperty('date');
    });

    test('MUST FAIL: should get diff between commits', async () => {
      // Get initial commit hash
      const logResult = await runGitCommand(['log', '--format=%H', '-n', '1'], tempProjectDir.name);
      const commitHash = logResult.stdout.trim();
      
      // Modify file and commit
      await fs.writeFile(
        path.join(tempProjectDir.name, 'main.py'),
        '# Modified\ndef main():\n    print("Updated")\n'
      );
      await runGitCommand(['add', 'main.py'], tempProjectDir.name);
      await runGitCommand(['commit', '-m', 'Update main'], tempProjectDir.name);
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--diff',
        commitHash,
        '--target', tempProjectDir.name
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.changes).toBeDefined();
      expect(Array.isArray(output.changes)).toBe(true);
      expect(output.changes.length).toBeGreaterThan(0);
      
      // Should show the change to main.py
      const mainChange = output.changes.find(c => c.file.endsWith('main.py'));
      expect(mainChange).toBeDefined();
      expect(mainChange.status).toBe('modified');
    });
  });

  describe('Branch Management', () => {
    test('MUST FAIL: should list all branches', async () => {
      // Create a new branch
      await runGitCommand(['checkout', '-b', 'feature-branch'], tempProjectDir.name);
      await runGitCommand(['checkout', 'master'], tempProjectDir.name);
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--branches',
        tempProjectDir.name
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.branches).toBeDefined();
      expect(Array.isArray(output.branches)).toBe(true);
      expect(output.branches.length).toBeGreaterThanOrEqual(2);
      
      // Should have both branches
      const branchNames = output.branches.map(b => b.name);
      expect(branchNames).toContain('master');
      expect(branchNames).toContain('feature-branch');
    });

    test('MUST FAIL: should detect current branch', async () => {
      const result = await runPythonScript([
        gitTrackerScript,
        '--check-repo',
        tempProjectDir.name
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.current_branch).toBe('master');
    });
  });

  describe('Smart Indexing Integration', () => {
    test('MUST FAIL: should identify files that need re-indexing based on git changes', async () => {
      // Create index first
      await fs.mkdir(indexPath, { recursive: true });
      await fs.writeFile(path.join(indexPath, 'last_indexed_commit.txt'), 'dummy-hash');
      
      // Modify files
      await fs.writeFile(
        path.join(tempProjectDir.name, 'main.py'),
        '# Updated\ndef main():\n    return "updated"'
      );
      await fs.writeFile(
        path.join(tempProjectDir.name, 'new.py'),
        'def new():\n    return "new"'
      );
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--reindex-needed',
        tempProjectDir.name,
        '--index-path', indexPath
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output.files_to_reindex).toBeDefined();
      expect(Array.isArray(output.files_to_reindex)).toBe(true);
      expect(output.files_to_reindex.length).toBeGreaterThan(0);
      
      // Should include modified and new files
      const filenames = output.files_to_reindex.map(f => path.basename(f));
      expect(filenames).toContain('main.py');
      expect(filenames).toContain('new.py');
    });

    test('MUST FAIL: should update index tracking with current commit hash', async () => {
      await fs.mkdir(indexPath, { recursive: true });
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--update-tracking',
        tempProjectDir.name,
        '--index-path', indexPath
      ]);
      
      expect(result.code).toBe(0);
      
      // Should create tracking file
      const trackingFile = path.join(indexPath, 'last_indexed_commit.txt');
      const exists = await fileExists(trackingFile);
      expect(exists).toBe(true);
      
      // Should contain current commit hash
      const content = await fs.readFile(trackingFile, 'utf-8');
      expect(content.trim().length).toBeGreaterThan(0);
      expect(content.trim()).toMatch(/^[a-f0-9]+$/); // Git hash format
    });
  });

  describe('File Filtering and Ignore Patterns', () => {
    test('MUST FAIL: should respect .gitignore patterns', async () => {
      // Create .gitignore
      await fs.writeFile(
        path.join(tempProjectDir.name, '.gitignore'),
        '*.log\n*.tmp\n__pycache__/\n'
      );
      
      // Create files that should be ignored
      await fs.writeFile(path.join(tempProjectDir.name, 'debug.log'), 'log content');
      await fs.writeFile(path.join(tempProjectDir.name, 'temp.tmp'), 'temp content');
      await fs.mkdir(path.join(tempProjectDir.name, '__pycache__'), { recursive: true });
      await fs.writeFile(path.join(tempProjectDir.name, '__pycache__', 'module.pyc'), 'bytecode');
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--status',
        tempProjectDir.name
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      
      // Should not include ignored files
      if (output.untracked_files) {
        const filenames = output.untracked_files.map(f => path.basename(f));
        expect(filenames).not.toContain('debug.log');
        expect(filenames).not.toContain('temp.tmp');
      }
    });

    test('MUST FAIL: should filter files by extension for indexing recommendations', async () => {
      // Create files with different extensions
      await fs.writeFile(path.join(tempProjectDir.name, 'code.py'), 'def code(): pass');
      await fs.writeFile(path.join(tempProjectDir.name, 'data.json'), '{}');
      await fs.writeFile(path.join(tempProjectDir.name, 'binary.exe'), 'binary content');
      await fs.writeFile(path.join(tempProjectDir.name, 'image.png'), 'fake png');
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--reindex-needed',
        tempProjectDir.name,
        '--code-only'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      
      if (output.files_to_reindex && output.files_to_reindex.length > 0) {
        // Should only include code files
        const extensions = output.files_to_reindex.map(f => path.extname(f));
        expect(extensions).toContain('.py');
        expect(extensions).not.toContain('.exe');
        expect(extensions).not.toContain('.png');
      }
    });
  });

  describe('Performance and Error Handling', () => {
    test('MUST FAIL: should handle non-git directories gracefully', async () => {
      // Create non-git directory
      const nonGitDir = tmp.dirSync({ unsafeCleanup: true });
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--check-repo',
        nonGitDir.name
      ]);
      
      expect(result.code).toBe(0); // Should not error
      
      const output = JSON.parse(result.stdout);
      expect(output.is_git_repo).toBe(false);
      
      nonGitDir.removeCallback();
    });

    test('MUST FAIL: should handle corrupted git repository', async () => {
      // Corrupt the .git directory
      await fs.rm(path.join(tempProjectDir.name, '.git', 'HEAD'), { force: true });
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--status',
        tempProjectDir.name
      ]);
      
      // Should handle gracefully (exit code 1 is acceptable for corrupted repo)
      expect([0, 1]).toContain(result.code);
      
      if (result.code === 1) {
        expect(result.stderr).toContain('git');
      }
    });

    test('MUST FAIL: should handle large repositories efficiently', async () => {
      // Create many files to simulate large repo
      const lotsOfFiles = [];
      for (let i = 0; i < 50; i++) {
        const filename = `file_${i}.py`;
        await fs.writeFile(
          path.join(tempProjectDir.name, filename),
          `# File ${i}\ndef function_${i}():\n    return ${i}`
        );
        lotsOfFiles.push(filename);
      }
      
      const startTime = Date.now();
      
      const result = await runPythonScript([
        gitTrackerScript,
        '--status',
        tempProjectDir.name
      ]);
      
      const duration = Date.now() - startTime;
      
      expect(result.code).toBe(0);
      expect(duration).toBeLessThan(10000); // Should complete within 10 seconds
      
      const output = JSON.parse(result.stdout);
      expect(output.untracked_files.length).toBeGreaterThanOrEqual(50);
    });
  });

  describe('Output Formats', () => {
    test('MUST FAIL: should support JSON output format', async () => {
      const result = await runPythonScript([
        gitTrackerScript,
        '--status',
        tempProjectDir.name,
        '--format', 'json'
      ]);
      
      expect(result.code).toBe(0);
      
      const output = JSON.parse(result.stdout);
      expect(output).toHaveProperty('modified_files');
      expect(output).toHaveProperty('untracked_files');
    });

    test('MUST FAIL: should support plain text output format', async () => {
      const result = await runPythonScript([
        gitTrackerScript,
        '--status',
        tempProjectDir.name,
        '--format', 'text'
      ]);
      
      expect(result.code).toBe(0);
      
      // Should be plain text (not JSON)
      expect(() => JSON.parse(result.stdout)).toThrow();
      expect(result.stdout).toContain('Status:');
    });
  });
});

// Helper Functions
async function createTestProject(projectPath) {
  // Create Python file
  await fs.writeFile(
    path.join(projectPath, 'main.py'),
    `#!/usr/bin/env python3
def main():
    print("Hello, World!")
    
if __name__ == '__main__':
    main()
`
  );
  
  // Create JavaScript file
  await fs.writeFile(
    path.join(projectPath, 'utils.js'),
    `function hello() {
    console.log("Hello from JavaScript!");
}

module.exports = { hello };
`
  );
  
  // Create README
  await fs.writeFile(
    path.join(projectPath, 'README.md'),
    `# Test Project

This is a test project for git integration.
`
  );
}

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
    const pythonExe = 'python';
    
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

async function runGitCommand(args, cwd) {
  return new Promise((resolve, reject) => {
    const child = spawn('git', args, {
      stdio: 'pipe',
      cwd: cwd,
      timeout: 10000
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
      reject(new Error('Git command timeout'));
    }, 10000);
  });
}