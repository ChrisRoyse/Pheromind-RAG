/**
 * Python Process Wrapper for MCP RAG Indexer
 * Manages Python MCP server process with health monitoring and restart capability
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs').promises;
const EventEmitter = require('events');
const chalk = require('chalk');

const { PlatformManager } = require('./platform');
const { ConfigManager } = require('./config');

class MCPWrapper extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.options = {
      autoRestart: true,
      maxRestarts: 5,
      restartDelay: 2000,
      healthCheckInterval: 30000,
      processTimeout: 60000,
      logLevel: 'info',
      ...options
    };
    
    this.pythonProcess = null;
    this.isHealthy = false;
    this.isStarting = false;
    this.isStopping = false;
    this.restartCount = 0;
    this.lastStartTime = null;
    this.healthCheckTimer = null;
    this.configManager = new ConfigManager();
    
    // Bind event handlers
    this.handleProcessExit = this.handleProcessExit.bind(this);
    this.handleProcessError = this.handleProcessError.bind(this);
    this.performHealthCheck = this.performHealthCheck.bind(this);
  }
  
  /**
   * Start the MCP server process
   */
  async start() {
    if (this.isStarting || (this.pythonProcess && !this.pythonProcess.killed)) {
      throw new Error('Process is already starting or running');
    }
    
    this.isStarting = true;
    this.isStopping = false;
    
    try {
      console.log(chalk.gray('Starting MCP RAG Indexer server...'));
      
      // Validate environment
      await this.validateEnvironment();
      
      // Get configuration
      const config = await this.configManager.getEnvironmentConfig();
      
      // Prepare process arguments and environment
      const { command, args, env } = await this.prepareProcess(config);
      
      // Spawn Python process
      this.pythonProcess = spawn(command, args, {
        stdio: ['pipe', 'pipe', 'pipe'],
        env,
        cwd: path.join(__dirname, '..')
      });
      
      this.lastStartTime = Date.now();
      
      // Setup process event handlers
      this.setupProcessHandlers();
      
      // Wait for process to start
      await this.waitForStartup();
      
      // Start health monitoring
      this.startHealthMonitoring();
      
      this.isHealthy = true;
      this.isStarting = false;
      this.restartCount = 0;
      
      console.log(chalk.green('✓ MCP RAG Indexer server started successfully'));
      this.emit('started', { pid: this.pythonProcess.pid });
      
    } catch (error) {
      this.isStarting = false;
      this.cleanup();
      throw error;
    }
  }
  
  /**
   * Stop the MCP server process
   */
  async stop() {
    if (this.isStopping) {
      return;
    }
    
    this.isStopping = true;
    console.log(chalk.yellow('Stopping MCP RAG Indexer server...'));
    
    try {
      // Stop health monitoring
      this.stopHealthMonitoring();
      
      if (this.pythonProcess && !this.pythonProcess.killed) {
        // Try graceful shutdown first
        this.pythonProcess.kill('SIGTERM');
        
        // Wait for graceful shutdown
        await this.waitForShutdown(5000);
        
        // Force kill if still running
        if (!this.pythonProcess.killed) {
          console.log(chalk.yellow('Process did not exit gracefully, forcing shutdown...'));
          this.pythonProcess.kill('SIGKILL');
          await this.waitForShutdown(2000);
        }
      }
      
      this.cleanup();
      console.log(chalk.green('✓ MCP RAG Indexer server stopped'));
      this.emit('stopped');
      
    } catch (error) {
      console.error(chalk.red('Error stopping process:'), error.message);
      this.forceCleanup();
    } finally {
      this.isStopping = false;
    }
  }
  
  /**
   * Restart the MCP server process
   */
  async restart() {
    console.log(chalk.yellow('Restarting MCP RAG Indexer server...'));
    
    if (this.pythonProcess && !this.pythonProcess.killed) {
      await this.stop();
    }
    
    // Wait a bit before restarting
    await new Promise(resolve => setTimeout(resolve, this.options.restartDelay));
    
    await this.start();
  }
  
  /**
   * Get process status information
   */
  getStatus() {
    return {
      isRunning: Boolean(this.pythonProcess && !this.pythonProcess.killed),
      isHealthy: this.isHealthy,
      isStarting: this.isStarting,
      isStopping: this.isStopping,
      pid: this.pythonProcess ? this.pythonProcess.pid : null,
      restartCount: this.restartCount,
      lastStartTime: this.lastStartTime,
      uptime: this.lastStartTime ? Date.now() - this.lastStartTime : 0
    };
  }
  
  /**
   * Send input to the process
   */
  sendInput(data) {
    if (this.pythonProcess && this.pythonProcess.stdin && !this.pythonProcess.stdin.destroyed) {
      this.pythonProcess.stdin.write(data);
      return true;
    }
    return false;
  }
  
  /**
   * Validate the environment before starting
   */
  async validateEnvironment() {
    // Check Python runtime
    if (!await PlatformManager.validateRuntime()) {
      throw new Error('Python runtime not found or invalid');
    }
    
    // Check MCP server script
    const serverScript = path.join(__dirname, '..', 'python', 'mcp_rag_server.py');
    try {
      await fs.access(serverScript);
    } catch (error) {
      throw new Error(`MCP server script not found: ${serverScript}`);
    }
    
    // Initialize configuration directories
    await this.configManager.initializeDirectories();
  }
  
  /**
   * Prepare process command, arguments, and environment
   */
  async prepareProcess(config) {
    const pythonPath = PlatformManager.getPythonExecutable();
    const serverScript = path.join(__dirname, '..', 'python', 'mcp_rag_server.py');
    
    const args = [serverScript];
    
    // Add configuration arguments
    if (config.logLevel) {
      args.push('--log-level', config.logLevel);
    }
    
    // Set up environment
    const env = {
      ...process.env,
      // Model cache directories
      TRANSFORMERS_CACHE: path.join(__dirname, '..', 'models'),
      SENTENCE_TRANSFORMERS_HOME: path.join(__dirname, '..', 'models'),
      HF_HOME: path.join(__dirname, '..', 'models'),
      // Performance optimizations
      TOKENIZERS_PARALLELISM: 'false',
      OMP_NUM_THREADS: '1',
      // Python path
      PYTHONPATH: path.join(__dirname, '..', 'python'),
      // Offline mode
      HF_HUB_OFFLINE: '1',
      TRANSFORMERS_OFFLINE: '1',
      // Configuration
      MCP_RAG_CONFIG_DIR: this.configManager.getConfigDir(),
      MCP_RAG_LOG_LEVEL: config.logLevel
    };
    
    return { command: pythonPath, args, env };
  }
  
  /**
   * Setup process event handlers
   */
  setupProcessHandlers() {
    this.pythonProcess.on('exit', this.handleProcessExit);
    this.pythonProcess.on('error', this.handleProcessError);
    
    // Handle stdout/stderr
    this.pythonProcess.stdout.on('data', (data) => {
      const output = data.toString().trim();
      if (output) {
        this.emit('stdout', output);
        if (this.options.logLevel === 'debug') {
          console.log(chalk.gray('[STDOUT]'), output);
        }
      }
    });
    
    this.pythonProcess.stderr.on('data', (data) => {
      const output = data.toString().trim();
      if (output) {
        this.emit('stderr', output);
        
        // Log errors (but not all stderr, as some might be warnings/info)
        if (output.toLowerCase().includes('error') || output.toLowerCase().includes('exception')) {
          console.error(chalk.red('[STDERR]'), output);
        } else if (this.options.logLevel === 'debug') {
          console.log(chalk.yellow('[STDERR]'), output);
        }
      }
    });
  }
  
  /**
   * Handle process exit
   */
  handleProcessExit(code, signal) {
    this.isHealthy = false;
    this.stopHealthMonitoring();
    
    console.log(chalk.yellow(`MCP server process exited with code ${code}, signal ${signal}`));
    
    this.emit('exit', { code, signal });
    
    // Auto-restart if enabled and not intentionally stopped
    if (this.options.autoRestart && !this.isStopping && this.restartCount < this.options.maxRestarts) {
      this.restartCount++;
      console.log(chalk.yellow(`Auto-restarting (attempt ${this.restartCount}/${this.options.maxRestarts})...`));
      
      setTimeout(() => {
        this.start().catch((error) => {
          console.error(chalk.red('Auto-restart failed:'), error.message);
          this.emit('restart-failed', error);
        });
      }, this.options.restartDelay);
    } else if (this.restartCount >= this.options.maxRestarts) {
      console.error(chalk.red('Max restart attempts reached, giving up'));
      this.emit('max-restarts-reached');
    }
  }
  
  /**
   * Handle process error
   */
  handleProcessError(error) {
    this.isHealthy = false;
    console.error(chalk.red('MCP server process error:'), error.message);
    
    this.emit('error', error);
    
    if (error.code === 'ENOENT') {
      console.error(chalk.red('Python executable not found. Please check your installation.'));
    }
  }
  
  /**
   * Wait for process startup
   */
  waitForStartup() {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Process startup timeout'));
      }, this.options.processTimeout);
      
      const checkStartup = () => {
        if (this.pythonProcess && !this.pythonProcess.killed) {
          clearTimeout(timeout);
          resolve();
        }
      };
      
      // Check immediately and then periodically
      checkStartup();
      const interval = setInterval(() => {
        checkStartup();
        if (this.pythonProcess && !this.pythonProcess.killed) {
          clearInterval(interval);
        }
      }, 100);
      
      // Handle early exit
      if (this.pythonProcess) {
        this.pythonProcess.once('exit', (code, signal) => {
          clearTimeout(timeout);
          clearInterval(interval);
          reject(new Error(`Process exited early with code ${code}, signal ${signal}`));
        });
      }
    });
  }
  
  /**
   * Wait for process shutdown
   */
  waitForShutdown(timeout = 5000) {
    return new Promise((resolve) => {
      if (!this.pythonProcess || this.pythonProcess.killed) {
        resolve();
        return;
      }
      
      const timer = setTimeout(() => {
        resolve(); // Timeout, but don't reject
      }, timeout);
      
      this.pythonProcess.once('exit', () => {
        clearTimeout(timer);
        resolve();
      });
    });
  }
  
  /**
   * Start health monitoring
   */
  startHealthMonitoring() {
    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer);
    }
    
    this.healthCheckTimer = setInterval(this.performHealthCheck, this.options.healthCheckInterval);
  }
  
  /**
   * Stop health monitoring
   */
  stopHealthMonitoring() {
    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer);
      this.healthCheckTimer = null;
    }
  }
  
  /**
   * Perform health check
   */
  performHealthCheck() {
    if (!this.pythonProcess || this.pythonProcess.killed) {
      this.isHealthy = false;
      this.emit('unhealthy', 'Process not running');
      return;
    }
    
    // Basic health check - process is running
    // Could be extended to send ping/pong messages
    const wasHealthy = this.isHealthy;
    this.isHealthy = true;
    
    if (!wasHealthy) {
      this.emit('healthy');
    }
  }
  
  /**
   * Cleanup resources
   */
  cleanup() {
    this.stopHealthMonitoring();
    this.isHealthy = false;
    
    if (this.pythonProcess) {
      this.pythonProcess.removeAllListeners();
      this.pythonProcess = null;
    }
  }
  
  /**
   * Force cleanup (for emergency situations)
   */
  forceCleanup() {
    this.cleanup();
    
    if (this.pythonProcess && !this.pythonProcess.killed) {
      try {
        this.pythonProcess.kill('SIGKILL');
      } catch (error) {
        // Ignore errors during force cleanup
      }
    }
  }
}

module.exports = { MCPWrapper };