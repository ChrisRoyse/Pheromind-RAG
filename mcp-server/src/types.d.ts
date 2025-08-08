// Type declarations for dynamic imports

declare module '../../mcp-bridge/index.node' {
  export function initializeBridge(config: {
    modelPath: string;
    tokenizerPath: string;
    useGpu: boolean;
  }): boolean;
  
  export function generateSemanticEmbedding(text: string): number[];
  export function generateHybridEmbedding(text: string): number[];
  export function generateNeuralEmbedding(text: string): number[];
  export function batchGenerateEmbeddings(texts: string[], type: string): number[][];
  export function similaritySearch(queryEmbedding: number[], k: number, threshold: number): Array<{id: number, score: number}>;
  export function addToIndex(embeddings: number[][], ids: number[]): boolean;
  export function getBridgeStats(): any;
  export function cleanupBridge(): boolean;
}

declare module 'chokidar' {
  interface FSWatcher {
    on(event: 'add', listener: (path: string) => void): FSWatcher;
    on(event: 'change', listener: (path: string) => void): FSWatcher;
    on(event: 'unlink', listener: (path: string) => void): FSWatcher;
    close(): Promise<void>;
  }
  
  interface WatchOptions {
    ignored?: string | string[];
    persistent?: boolean;
    ignoreInitial?: boolean;
  }
  
  function watch(path: string, options?: WatchOptions): FSWatcher;
  
  export default { watch };
}