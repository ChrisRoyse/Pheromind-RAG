use neon::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use rayon::prelude::*;
use anyhow::{Result, Context};
use tokenizers::Tokenizer;

mod embedding;
mod faiss_index;
mod onnx_models;

use embedding::{EmbeddingGenerator, EmbeddingType};
use faiss_index::FaissIndexManager;
use onnx_models::OnnxModelManager;

/// Global state for the Rust bridge
struct BridgeState {
    embedding_generator: Arc<Mutex<EmbeddingGenerator>>,
    faiss_manager: Arc<Mutex<FaissIndexManager>>,
    onnx_manager: Arc<Mutex<OnnxModelManager>>,
    initialized: bool,
}

impl BridgeState {
    fn new() -> Self {
        Self {
            embedding_generator: Arc::new(Mutex::new(EmbeddingGenerator::new())),
            faiss_manager: Arc::new(Mutex::new(FaissIndexManager::new())),
            onnx_manager: Arc::new(Mutex::new(OnnxModelManager::new())),
            initialized: false,
        }
    }
}

static mut BRIDGE_STATE: Option<BridgeState> = None;

fn get_bridge_state() -> &'static mut BridgeState {
    unsafe {
        if BRIDGE_STATE.is_none() {
            BRIDGE_STATE = Some(BridgeState::new());
        }
        BRIDGE_STATE.as_mut().unwrap()
    }
}

/// Initialize the Rust bridge with model paths
fn initialize_bridge(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let config = cx.argument::<JsObject>(0)?;
    
    // Extract configuration
    let model_path: String = config
        .get(&mut cx, "modelPath")?
        .downcast_or_throw::<JsString, _>(&mut cx)?
        .value(&mut cx);
    
    let tokenizer_path: String = config
        .get(&mut cx, "tokenizerPath")?
        .downcast_or_throw::<JsString, _>(&mut cx)?
        .value(&mut cx);

    let use_gpu = config
        .get(&mut cx, "useGpu")
        .ok()
        .and_then(|v| v.downcast::<JsBoolean, _>(&mut cx).ok())
        .map(|b| b.value(&mut cx))
        .unwrap_or(false);

    let state = get_bridge_state();
    
    // Initialize components
    let result = (|| -> Result<()> {
        let mut embedding_gen = state.embedding_generator.lock().unwrap();
        embedding_gen.initialize(&model_path, &tokenizer_path, use_gpu)
            .context("Failed to initialize embedding generator")?;

        let mut onnx_manager = state.onnx_manager.lock().unwrap();
        onnx_manager.initialize(&model_path, use_gpu)
            .context("Failed to initialize ONNX manager")?;

        let mut faiss_manager = state.faiss_manager.lock().unwrap();
        faiss_manager.initialize()
            .context("Failed to initialize FAISS manager")?;

        Ok(())
    })();

    match result {
        Ok(_) => {
            state.initialized = true;
            Ok(cx.boolean(true))
        }
        Err(e) => {
            let error_msg = format!("Initialization failed: {}", e);
            cx.throw_error(error_msg)
        }
    }
}

/// Generate semantic embedding for text
fn generate_semantic_embedding(mut cx: FunctionContext) -> JsResult<JsArray> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let state = get_bridge_state();
    
    if !state.initialized {
        return cx.throw_error("Bridge not initialized. Call initializeBridge first.");
    }

    let embedding = {
        let embedding_gen = state.embedding_generator.lock().unwrap();
        embedding_gen.generate_embedding(&text, EmbeddingType::Semantic)
            .map_err(|e| format!("Failed to generate semantic embedding: {}", e))
    };

    match embedding {
        Ok(vec) => {
            let js_array = JsArray::new(&mut cx, vec.len() as u32);
            for (i, value) in vec.iter().enumerate() {
                let js_num = cx.number(*value as f64);
                js_array.set(&mut cx, i as u32, js_num)?;
            }
            Ok(js_array)
        }
        Err(e) => cx.throw_error(e)
    }
}

/// Generate hybrid embedding for text
fn generate_hybrid_embedding(mut cx: FunctionContext) -> JsResult<JsArray> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let state = get_bridge_state();
    
    if !state.initialized {
        return cx.throw_error("Bridge not initialized. Call initializeBridge first.");
    }

    let embedding = {
        let embedding_gen = state.embedding_generator.lock().unwrap();
        embedding_gen.generate_embedding(&text, EmbeddingType::Hybrid)
            .map_err(|e| format!("Failed to generate hybrid embedding: {}", e))
    };

    match embedding {
        Ok(vec) => {
            let js_array = JsArray::new(&mut cx, vec.len() as u32);
            for (i, value) in vec.iter().enumerate() {
                let js_num = cx.number(*value as f64);
                js_array.set(&mut cx, i as u32, js_num)?;
            }
            Ok(js_array)
        }
        Err(e) => cx.throw_error(e)
    }
}

/// Generate neural embedding for text
fn generate_neural_embedding(mut cx: FunctionContext) -> JsResult<JsArray> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let state = get_bridge_state();
    
    if !state.initialized {
        return cx.throw_error("Bridge not initialized. Call initializeBridge first.");
    }

    let embedding = {
        let embedding_gen = state.embedding_generator.lock().unwrap();
        embedding_gen.generate_embedding(&text, EmbeddingType::Neural)
            .map_err(|e| format!("Failed to generate neural embedding: {}", e))
    };

    match embedding {
        Ok(vec) => {
            let js_array = JsArray::new(&mut cx, vec.len() as u32);
            for (i, value) in vec.iter().enumerate() {
                let js_num = cx.number(*value as f64);
                js_array.set(&mut cx, i as u32, js_num)?;
            }
            Ok(js_array)
        }
        Err(e) => cx.throw_error(e)
    }
}

/// Batch generate embeddings for multiple texts
fn batch_generate_embeddings(mut cx: FunctionContext) -> JsResult<JsArray> {
    let texts_array = cx.argument::<JsArray>(0)?;
    let embedding_type_str = cx.argument::<JsString>(1)?.value(&mut cx);
    
    let state = get_bridge_state();
    
    if !state.initialized {
        return cx.throw_error("Bridge not initialized. Call initializeBridge first.");
    }

    // Parse embedding type
    let embedding_type = match embedding_type_str.as_str() {
        "semantic" => EmbeddingType::Semantic,
        "hybrid" => EmbeddingType::Hybrid,
        "neural" => EmbeddingType::Neural,
        _ => return cx.throw_error("Invalid embedding type. Use 'semantic', 'hybrid', or 'neural'")
    };

    // Extract texts
    let len = texts_array.len(&mut cx);
    let mut texts = Vec::with_capacity(len as usize);
    
    for i in 0..len {
        let text: Handle<JsString> = texts_array.get(&mut cx, i)?;
        texts.push(text.value(&mut cx));
    }

    // Generate embeddings in parallel
    let embeddings_result = {
        let embedding_gen = state.embedding_generator.lock().unwrap();
        texts.par_iter()
            .map(|text| embedding_gen.generate_embedding(text, embedding_type.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to generate batch embeddings: {}", e))
    };

    match embeddings_result {
        Ok(embeddings) => {
            let js_result = JsArray::new(&mut cx, embeddings.len() as u32);
            
            for (i, embedding) in embeddings.iter().enumerate() {
                let js_embedding = JsArray::new(&mut cx, embedding.len() as u32);
                for (j, value) in embedding.iter().enumerate() {
                    let js_num = cx.number(*value as f64);
                    js_embedding.set(&mut cx, j as u32, js_num)?;
                }
                js_result.set(&mut cx, i as u32, js_embedding)?;
            }
            
            Ok(js_result)
        }
        Err(e) => cx.throw_error(e)
    }
}

/// Perform similarity search using FAISS
fn similarity_search(mut cx: FunctionContext) -> JsResult<JsArray> {
    let query_embedding_array = cx.argument::<JsArray>(0)?;
    let k = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let threshold = cx.argument::<JsNumber>(2)?.value(&mut cx) as f32;
    
    let state = get_bridge_state();
    
    if !state.initialized {
        return cx.throw_error("Bridge not initialized. Call initializeBridge first.");
    }

    // Convert JS array to Vec<f32>
    let len = query_embedding_array.len(&mut cx);
    let mut query_embedding = Vec::with_capacity(len as usize);
    
    for i in 0..len {
        let val: Handle<JsNumber> = query_embedding_array.get(&mut cx, i)?;
        query_embedding.push(val.value(&mut cx) as f32);
    }

    // Perform search
    let search_result = {
        let faiss_manager = state.faiss_manager.lock().unwrap();
        faiss_manager.search(&query_embedding, k, threshold)
            .map_err(|e| format!("FAISS search failed: {}", e))
    };

    match search_result {
        Ok(results) => {
            let js_results = JsArray::new(&mut cx, results.len() as u32);
            
            for (i, (id, score)) in results.iter().enumerate() {
                let js_result = JsObject::new(&mut cx);
                
                let js_id = cx.number(*id as f64);
                js_result.set(&mut cx, "id", js_id)?;
                
                let js_score = cx.number(*score as f64);
                js_result.set(&mut cx, "score", js_score)?;
                
                js_results.set(&mut cx, i as u32, js_result)?;
            }
            
            Ok(js_results)
        }
        Err(e) => cx.throw_error(e)
    }
}

/// Add vectors to FAISS index
fn add_to_index(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let embeddings_array = cx.argument::<JsArray>(0)?;
    let ids_array = cx.argument::<JsArray>(1)?;
    
    let state = get_bridge_state();
    
    if !state.initialized {
        return cx.throw_error("Bridge not initialized. Call initializeBridge first.");
    }

    let embeddings_len = embeddings_array.len(&mut cx);
    let ids_len = ids_array.len(&mut cx);
    
    if embeddings_len != ids_len {
        return cx.throw_error("Embeddings and IDs arrays must have the same length");
    }

    // Extract embeddings and IDs
    let mut embeddings = Vec::new();
    let mut ids = Vec::new();
    
    for i in 0..embeddings_len {
        let embedding_array: Handle<JsArray> = embeddings_array.get(&mut cx, i)?;
        let embedding_len = embedding_array.len(&mut cx);
        let mut embedding = Vec::with_capacity(embedding_len as usize);
        
        for j in 0..embedding_len {
            let val: Handle<JsNumber> = embedding_array.get(&mut cx, j)?;
            embedding.push(val.value(&mut cx) as f32);
        }
        embeddings.push(embedding);
        
        let id: Handle<JsNumber> = ids_array.get(&mut cx, i)?;
        ids.push(id.value(&mut cx) as i64);
    }

    // Add to index
    let result = {
        let mut faiss_manager = state.faiss_manager.lock().unwrap();
        faiss_manager.add_vectors(&embeddings, &ids)
            .map_err(|e| format!("Failed to add vectors to index: {}", e))
    };

    match result {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(e)
    }
}

/// Get bridge statistics
fn get_bridge_stats(mut cx: FunctionContext) -> JsResult<JsObject> {
    let state = get_bridge_state();
    let stats = JsObject::new(&mut cx);
    
    // Basic stats
    let initialized = cx.boolean(state.initialized);
    stats.set(&mut cx, "initialized", initialized)?;
    
    // Memory usage stats (simplified)
    let memory_stats = JsObject::new(&mut cx);
    memory_stats.set(&mut cx, "allocated", cx.number(0.0))?; // Would need actual memory tracking
    stats.set(&mut cx, "memory", memory_stats)?;
    
    // FAISS stats
    let faiss_stats = {
        if state.initialized {
            let faiss_manager = state.faiss_manager.lock().unwrap();
            match faiss_manager.get_stats() {
                Ok(stats) => {
                    let js_stats = JsObject::new(&mut cx);
                    js_stats.set(&mut cx, "indexSize", cx.number(stats.index_size as f64))?;
                    js_stats.set(&mut cx, "totalVectors", cx.number(stats.total_vectors as f64))?;
                    js_stats
                }
                Err(_) => JsObject::new(&mut cx)
            }
        } else {
            JsObject::new(&mut cx)
        }
    };
    stats.set(&mut cx, "faiss", faiss_stats)?;
    
    Ok(stats)
}

/// Clean up resources
fn cleanup_bridge(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let state = get_bridge_state();
    
    // Clean up all managers
    {
        let mut embedding_gen = state.embedding_generator.lock().unwrap();
        let _ = embedding_gen.cleanup();
    }
    
    {
        let mut faiss_manager = state.faiss_manager.lock().unwrap();
        let _ = faiss_manager.cleanup();
    }
    
    {
        let mut onnx_manager = state.onnx_manager.lock().unwrap();
        let _ = onnx_manager.cleanup();
    }
    
    state.initialized = false;
    
    Ok(cx.boolean(true))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // Initialize logging
    env_logger::init();
    
    // Export functions to Node.js
    cx.export_function("initializeBridge", initialize_bridge)?;
    cx.export_function("generateSemanticEmbedding", generate_semantic_embedding)?;
    cx.export_function("generateHybridEmbedding", generate_hybrid_embedding)?;
    cx.export_function("generateNeuralEmbedding", generate_neural_embedding)?;
    cx.export_function("batchGenerateEmbeddings", batch_generate_embeddings)?;
    cx.export_function("similaritySearch", similarity_search)?;
    cx.export_function("addToIndex", add_to_index)?;
    cx.export_function("getBridgeStats", get_bridge_stats)?;
    cx.export_function("cleanupBridge", cleanup_bridge)?;
    
    Ok(())
}