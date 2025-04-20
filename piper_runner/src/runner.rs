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