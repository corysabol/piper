use anyhow::{Context, Result};
use hyper::Uri;
use piper_dsl::{Pipeline, Task, TaskType, Value};
use piper_tasks::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tokio::runtime::Runtime;

use mlua::{Lua, LuaTable, Function, Value as LuaValue};

pub fn run_from_file(path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let pipeline_string = fs::read_to_string(path)
        .context("Failed to read pipeline file")?;
    run(pipeline_string)
}

pub fn run(pipeline_string: String) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::new().unwrap();
    
    // Parse the pipeline using the DSL parser
    let pipeline = Pipeline::parse(&pipeline_string)
        .context("Failed to parse pipeline")?;
    
    println!(
        "Pipeline: {}\nAuthor: {:?}\nDesc: {:?}\n",
        pipeline.name, pipeline.author, pipeline.description
    );
    
    // Create a Lua state for script execution
    let lua = Lua::new();
    
    // Create a context table for variable storage
    let ctx = lua.create_table()?;
    ctx.set("bar", 1337)?;
    
    // Execute each task in the pipeline
    for task in pipeline.tasks {
        let task_type = &task.task_type;
        let task_name = &task.name;
        
        println!("[+] Running Task: {:?}", task_type);
        
        if let Some(name) = task_name {
            println!("    Name: {}", name);
        }
        
        match task_type {
            TaskType::Cmd => {
                let cmd = get_string_property(&task.properties, "cmd")
                    .context("Missing 'cmd' property for cmd task")?;
                
                // Convert task properties to HashMap<String, String>
                let args = convert_properties_to_hashmap(&task.properties);
                
                // Interpolate variables in the command
                let cmd = interpolate_variables(&cmd, &ctx)?;
                let mut args_map = HashMap::new();
                args_map.insert("cmd".to_string(), cmd);
                
                let (stdout, stderr) = cmd::run(&args_map);
                if !stdout.is_empty() {
                    println!("stdout: {}", stdout);
                }
                
                if !stderr.is_empty() {
                    println!("stderr: {}", stderr);
                }
            }
            TaskType::Script => {
                let script = get_string_property(&task.properties, "script")
                    .context("Missing 'script' property for script task")?;
                
                // Interpolate variables in the script
                let script = interpolate_variables(&script, &ctx)?;
                
                // Execute the Lua script
                lua.globals().set("ctx", ctx.clone())?;
                lua.load(&script).exec()?;
            }
            TaskType::Llm => {
                let prompt = get_string_property(&task.properties, "prompt")
                    .context("Missing 'prompt' property for llm task")?;
                
                // Convert task properties to HashMap<String, String>
                let args = convert_properties_to_hashmap(&task.properties);
                
                // Run LLM inference
                let result = llm::run(&args)?;
                println!("LLM output: {}", result);
                
                // Store the result in the context if output variable is specified
                if let Some(output_var) = task.properties.get("output") {
                    if let Value::String(var_name) = output_var {
                        ctx.set(var_name.clone(), result)?;
                    }
                }
            }
            TaskType::Http => {
                let url = get_string_property(&task.properties, "url")
                    .context("Missing 'url' property for http task")?;
                
                // Convert task properties to HashMap<String, String>
                let args = convert_properties_to_hashmap(&task.properties);
                
                // Make HTTP request
                runtime.block_on(http::http_get(&args));
            }
            TaskType::Notify => {
                let uri = get_string_property(&task.properties, "uri")
                    .context("Missing 'uri' property for notify task")?;
                
                // Convert task properties to HashMap<String, String>
                let args = convert_properties_to_hashmap(&task.properties);
                
                // Send notification
                let message = serde_json::Value::String(serde_json::to_string(&args).unwrap());
                notify::notify(&uri, message);
            }
            TaskType::SetVar => {
                let var_name = get_string_property(&task.properties, "var")
                    .context("Missing 'var' property for set_var task")?;
                
                let value = task.properties.get("val")
                    .context("Missing 'val' property for set_var task")?;
                
                // Set variable in context
                match value {
                    Value::String(s) => ctx.set(var_name, s.clone())?,
                    Value::Number(n) => ctx.set(var_name, *n)?,
                    Value::Boolean(b) => ctx.set(var_name, *b)?,
                    _ => ctx.set(var_name, value.to_string())?,
                }
            }
            TaskType::Lua => {
                let code = get_string_property(&task.properties, "code")
                    .context("Missing 'code' property for lua task")?;
                
                // Execute Lua code
                lua.globals().set("ctx", ctx.clone())?;
                lua.load(&code).exec()?;
            }
        }
    }
    
    Ok(())
}

// Helper function to get a string property from task properties
fn get_string_property(properties: &HashMap<String, Value>, key: &str) -> Option<String> {
    properties.get(key).and_then(|v| {
        match v {
            Value::String(s) => Some(s.clone()),
            _ => Some(v.to_string()),
        }
    })
}

// Helper function to convert task properties to HashMap<String, String>
fn convert_properties_to_hashmap(properties: &HashMap<String, Value>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    
    for (key, value) in properties {
        let string_value = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            _ => value.to_string(),
        };
        
        result.insert(key.clone(), string_value);
    }
    
    result
}

// Helper function to interpolate variables in a string
fn interpolate_variables(input: &str, ctx: &LuaTable) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = input.to_string();
    
    // Find all variable interpolations (#{var})
    let re = regex::Regex::new(r"#\{([a-zA-Z0-9_]+)\}")?;
    
    for cap in re.captures_iter(input) {
        let var_name = &cap[1];
        let var_value = ctx.get::<_, LuaValue>(var_name)?;
        
        let var_str = match var_value {
            LuaValue::String(s) => s.to_str()?.to_string(),
            LuaValue::Integer(i) => i.to_string(),
            LuaValue::Number(n) => n.to_string(),
            LuaValue::Boolean(b) => b.to_string(),
            _ => "[complex value]".to_string(),
        };
        
        result = result.replace(&format!("#{{{}}}", var_name), &var_str);
    }
    
    Ok(result)
}
