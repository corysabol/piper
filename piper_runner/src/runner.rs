use anyhow::{Context, Result};
use piper_dsl::{
    Argument, Flow, FlowItem, Parameter, ParseError, Pipeline, PiperParser, Task, TaskType, Value,
};
use piper_tasks::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::runtime::Runtime;

use mlua::{Lua, LuaTable, Value as LuaValue};
use regex::Regex;

pub fn run_from_file(path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    run_from_file_with_options(path, false)
}

pub fn run_from_file_with_options(
    path: std::path::PathBuf,
    regenerate: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let pipeline_string = fs::read_to_string(&path).context("Failed to read pipeline file")?;

    // Try to parse as a meta-pipeline first
    match Pipeline::parse(&pipeline_string) {
        Ok(meta_pipeline) => {
            println!("Detected meta-pipeline: {}", meta_pipeline.name);

            // Generate the actual pipeline
            let generated_pipeline = meta_pipeline
                .generate_pipeline(regenerate)
                .context("Failed to generate pipeline from meta-pipeline")?;

            // Get the path to the generated pipeline
            let generated_dir = Path::new("generated");
            let generated_path = generated_dir.join(format!("{}.piper", meta_pipeline.name));

            println!("Using generated pipeline at: {:?}", generated_path);

            // Run the generated pipeline
            run(generated_pipeline)
        }
        Err(_) => {
            // Not a meta-pipeline, run as a regular pipeline
            run(pipeline_string)
        }
    }
}

pub fn run(pipeline_string: String) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::new().unwrap();

    // Parse the pipeline using the DSL parser
    let pipeline = Pipeline::parse(&pipeline_string).context("Failed to parse pipeline")?;

    // Create a Lua state for script execution
    let lua = Lua::new();

    // Create a context table for variable storage
    let ctx = lua.create_table()?;
    ctx.set("bar", 1337)?;

    // Create storage for meta-tasks and generated content
    let mut meta_tasks = HashMap::new();

    // Execute each task in the pipeline

    for (task_name, task) in pipeline.tasks {

        println!("[+] Running Task: {:?}", task_name);

        if let Some(name) = task_name {
            println!("    Name: {}", name);
        }

        match task.task_type {
        }
    }

    Ok(())
}

// Helper function to get a string property from task properties
fn get_string_property(properties: &HashMap<String, Value>, key: &str) -> Option<String> {
    properties.get(key).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        _ => Some(v.to_string()),
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
fn interpolate_variables(
    input: &str,
    ctx: &LuaTable,
) -> Result<String, Box<dyn std::error::Error>> {
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

// Function to generate tasks from meta-task descriptions
fn generate_tasks_from_meta_tasks(
    meta_tasks: &HashMap<String, (String, String)>,
    meta_task_names: &[String],
    custom_task_names: &[String],
    model: &str,
    style: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    todo!()
}

// Function to generate flow from constraints
fn generate_flow_from_constraints(
    tasks: &LuaValue,
    constraints: &LuaValue,
    description: &LuaValue,
    model: &str,
    visualization: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    todo!()
}

// Helper function to convert Lua values to strings
fn lua_value_to_string(value: &LuaValue) -> Result<String, Box<dyn std::error::Error>> {
    match value {
        LuaValue::String(s) => Ok(s.to_str()?.to_string()),
        LuaValue::Table(t) => {
            let mut result = String::new();

            // Try to convert table to array of strings
            let len = t.len()?;
            if len > 0 {
                for i in 1..=len {
                    if let Ok(item) = t.get::<_, LuaValue>(i) {
                        if let Ok(item_str) = lua_value_to_string(&item) {
                            result.push_str(&format!("- {}\n", item_str));
                        }
                    }
                }
            } else {
                // Try to convert table to key-value pairs
                for pair in t.pairs::<LuaValue, LuaValue>() {
                    if let Ok((key, value)) = pair {
                        if let (Ok(key_str), Ok(value_str)) =
                            (lua_value_to_string(&key), lua_value_to_string(&value))
                        {
                            result.push_str(&format!("{}: {}\n", key_str, value_str));
                        }
                    }
                }
            }

            Ok(result)
        }
        _ => Ok(format!("{:?}", value)),
    }
}
