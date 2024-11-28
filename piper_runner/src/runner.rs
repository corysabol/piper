use cmd::CommandTask;
use lua::LuaTask;
use mlua::{Lua, Table, Value as LuaValue};
use piper_tasks::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::str::FromStr;
use task::{Task, TaskDefinition};
use tokio::runtime::Runtime;

// Pipeline
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Pipeline {
    name: String,
    author: String,
    description: String,
    tasks: Vec<Task>,
}

// task struct which represents a task to be executed
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    name: Option<String>,
    comment: Option<String>,
    task: String,
    args: Option<HashMap<String, String>>,
}

pub fn run_from_file(path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let pipeline_string = fs::read_to_string(path).unwrap();
    run(pipeline_string)
}

pub fn run(pipeline_string: String) -> Result<(), Box<dyn std::error::Error>> {
    let lua = Lua::new();
    let globals = lua.globals();
    let ctx: Table = lua.create_table().unwrap();
    globals.set("ctx", ctx.clone());

    let pipeline: Pipeline = serde_yaml::from_str(&pipeline_string).unwrap();

    // Task dep graph
    let mut task_map: HashMap<String, TaskDefinition> = HashMap::new();
    for task in &pipeline.tasks {
        task_map.insert(task.name.clone(), task.clone());
    }

    println!(
        "Pipeline: {:#?}\nAuthor: {:#?}\nDesc:{:#?}\n",
        pipeline.name, pipeline.author, pipeline.description
    );

    let mut i = 0;
    while i < pipeline.tasks.len() {
        let task = &pipeline.tasks[i];

        match task.task.as_str() {
            "if" => {
                // Evaluate condition in Lua
                let condition = task.if_condition.as_ref().unwrap();
                let result: bool = lua.load(&condition).eval()?;

                // Execute appropriate branch tasks
                let branch_tasks = if result {
                    &task.then_tasks
                } else {
                    &task.else_tasks
                };

                for task_name in branch_tasks.as_ref().unwrap() {
                    execute_task(&lua, &ctx, &task_map[task_name])?;
                }
            }
            "llm" => {
                // Execute LLM task first
                execute_task(&lua, &ctx, task)?;

                // Get LLM's decision from context
                let decision: String = ctx.get(format!("task_{}_decision", task.name))?;

                // Execute chosen task
                if task.available_tasks.as_ref().unwrap().contains(&decision) {
                    execute_task(&lua, &ctx, &task_map[&decision])?;
                }
            }
            _ => {
                if task.flow == Some(TaskFlow::Sequential) || task.flow.is_none() {
                    execute_task(&lua, &ctx, task)?;
                }
            }
        }
        i += 1;
    }
    Ok(())
}
