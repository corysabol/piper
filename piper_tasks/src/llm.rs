use anyhow::{Error as E, Result};
use hf_hub::{Repo, RepoType};
use kalosm::pipeline::TextGenerationPipeline;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokenizers::Tokenizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub model: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub prompt: String,
    pub config: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub text: String,
    pub tokens_used: usize,
}

#[derive(Debug, Clone)]
pub struct LlmEngine {
    pipeline: TextGenerationPipeline,
    tokenizer: Tokenizer,
    model_id: String,
}

impl LlmEngine {
    pub fn new(model_id: &str) -> Result<Self> {
        // Download model from Hugging Face Hub
        let api = hf_hub::api::sync::Api::new()?;
        let api = api.model(model_id.to_string());
        
        let model_path = api.get("model.safetensors")?;
        let tokenizer_path = api.get("tokenizer.json")?;
        
        // Initialize the pipeline and tokenizer
        let pipeline = TextGenerationPipeline::new(model_path)?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)?;
        
        Ok(Self {
            pipeline,
            tokenizer,
            model_id: model_id.to_string(),
        })
    }
    
    pub fn generate(&self, request: &LlmRequest) -> Result<LlmResponse> {
        let mut prompt = request.prompt.clone();
        
        // Add system prompt if provided
        if let Some(system) = &request.config.system_prompt {
            prompt = format!("{}\n\n{}", system, prompt);
        }
        
        // Count input tokens
        let input_tokens = self.tokenizer.encode(prompt.clone(), true)?;
        
        // Generate text
        let generation_config = kalosm::pipeline::TextGenerationConfig {
            temperature: request.config.temperature.unwrap_or(0.7),
            max_new_tokens: request.config.max_tokens.unwrap_or(512),
            ..Default::default()
        };
        
        let output = self.pipeline.generate(
            &[prompt],
            &generation_config,
        )?;
        
        let generated_text = output[0].clone();
        
        // Count output tokens
        let output_tokens = self.tokenizer.encode(generated_text.clone(), true)?;
        let tokens_used = input_tokens.get_ids().len() + output_tokens.get_ids().len();
        
        Ok(LlmResponse {
            text: generated_text,
            tokens_used,
        })
    }
}

/// Run LLM inference with the given arguments
pub fn run(args: &HashMap<String, String>) -> Result<String> {
    let model = args.get("model").unwrap_or(&"mistralai/Mistral-7B-Instruct-v0.2".to_string()).clone();
    let prompt = args.get("prompt").ok_or_else(|| E::msg("Prompt is required"))?;
    
    let system_prompt = args.get("system").cloned();
    let temperature = args.get("temperature").and_then(|t| t.parse::<f32>().ok());
    let max_tokens = args.get("max_tokens").and_then(|t| t.parse::<usize>().ok());
    
    let config = LlmConfig {
        model: model.clone(),
        system_prompt,
        temperature,
        max_tokens,
    };
    
    let request = LlmRequest {
        prompt: prompt.clone(),
        config,
    };
    
    // Initialize the LLM engine
    let engine = LlmEngine::new(&model)?;
    
    // Generate text
    let response = engine.generate(&request)?;
    
    Ok(response.text)
}

/// Process a command output with LLM
pub fn process_output(output: &str, args: &HashMap<String, String>) -> Result<String> {
    let model = args.get("model").unwrap_or(&"mistralai/Mistral-7B-Instruct-v0.2".to_string()).clone();
    let instruction = args.get("instruction").ok_or_else(|| E::msg("Instruction is required"))?;
    
    let prompt = format!("Command output:\n```\n{}\n```\n\nInstruction: {}", output, instruction);
    
    let system_prompt = args.get("system").unwrap_or(&"You are a helpful assistant that processes command outputs according to the given instructions.".to_string()).clone();
    let temperature = args.get("temperature").and_then(|t| t.parse::<f32>().ok()).unwrap_or(0.7);
    let max_tokens = args.get("max_tokens").and_then(|t| t.parse::<usize>().ok()).unwrap_or(512);
    
    let config = LlmConfig {
        model,
        system_prompt: Some(system_prompt),
        temperature: Some(temperature),
        max_tokens: Some(max_tokens),
    };
    
    let request = LlmRequest {
        prompt,
        config,
    };
    
    // Initialize the LLM engine
    let engine = LlmEngine::new(&config.model)?;
    
    // Generate text
    let response = engine.generate(&request)?;
    
    Ok(response.text)
}
