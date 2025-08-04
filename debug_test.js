const tmp = require('tmp');
const fs = require('fs').promises;
const { spawn } = require('child_process');
const path = require('path');

function getFileExtension(language) {
  const extensions = {
    rust: 'rs',
    python: 'py',
    javascript: 'js',
    typescript: 'ts',
    java: 'java',
    cpp: 'cpp',
    c: 'c'
  };
  
  return extensions[language] || 'txt';
}

async function testIndexer() {
  const rustCode = `/// A spiking cortical column with TTFS dynamics.
/// This struct represents a biologically-inspired cortical column
/// that processes temporal information using time-to-first-spike encoding.
pub struct SpikingCorticalColumn {
    /// The current activation level of the column
    activation_level: f64,
    /// Threshold for spike generation  
    spike_threshold: f64,
}`;

  // Create temporary file exactly like test_utils.js does
  const language = 'rust';
  const extension = getFileExtension(language);
  console.log('Language:', language, 'Extension:', extension);
  
  // Create temp file with proper extension
  const tempFile = tmp.fileSync({ postfix: `.${extension}` });
  console.log('Temp file:', tempFile.name);
  console.log('File extension:', path.extname(tempFile.name));
  await fs.writeFile(tempFile.name, rustCode);
  
  // Test the indexer
  const pythonPath = path.join(process.cwd(), 'python', 'indexer_universal.py');
  console.log('Python path:', pythonPath);
  console.log('Command:', ['python', pythonPath, 'index', tempFile.name]);
  
  return new Promise((resolve, reject) => {
    const child = spawn('python', [pythonPath, 'index', tempFile.name], {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: process.cwd()
    });
    
    let stdout = '';
    let stderr = '';
    
    child.stdout.on('data', (data) => stdout += data.toString());
    child.stderr.on('data', (data) => stderr += data.toString());
    
    child.on('close', (code) => {
      console.log('Exit code:', code);
      console.log('Stdout length:', stdout.length);
      console.log('Stderr length:', stderr.length);
      if (stderr) console.log('Stderr:', stderr);
      
      if (code !== 0) {
        reject(new Error(`Failed with code ${code}: ${stderr}`));
      } else {
        try {
          console.log('Raw stdout:', JSON.stringify(stdout));
          if (stdout.trim()) {
            const result = JSON.parse(stdout);
            console.log('Result chunks count:', result.chunks.length);
            resolve(result);
          } else {
            console.log('Empty stdout!');
            resolve({chunks: []});
          }
        } catch (e) {
          console.log('Parse error:', e.message);
          console.log('Raw stdout:', stdout);
          reject(e);
        }
      }
      
      // Cleanup
      tempFile.removeCallback();
    });
    
    child.on('error', (error) => {
      console.log('Spawn error:', error.message);
      reject(error);
    });
  });
}

testIndexer().then(result => {
  console.log('Success! Found', result.chunks.length, 'chunks');
  if (result.chunks.length > 0) {
    console.log('First chunk has_documentation:', result.chunks[0].has_documentation);
    console.log('First chunk confidence:', result.chunks[0].confidence);
  }
}).catch(err => {
  console.error('Error:', err.message);
});