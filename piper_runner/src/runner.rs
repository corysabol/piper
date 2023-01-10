use piper_tasks::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::runtime::Runtime;

use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyBytes, PyDict, PyModule},
};

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
    let runtime = Runtime::new().unwrap();
    //dbg!(dns_lookup(vec!["www.secureideas.com".to_owned()]));

    let deserialized_pipeline: Pipeline = serde_yaml::from_str(&pipeline_string)?;

    println!(
        "Pipeline: {:#?}\nAuthor: {:#?}\nDesc:{:#?}\n",
        deserialized_pipeline.name, deserialized_pipeline.author, deserialized_pipeline.description
    );

    Python::with_gil(|py| {
        let locals = PyDict::new(py);
        let ctx = PyDict::new(py);
        ctx.set_item("bar", 1337);
        locals.set_item("ctx", ctx);

        deserialized_pipeline.tasks.iter().for_each(|task| {
            let task_kind = &task.task;
            let mut task_name = task.name.as_ref();
            let mut task_comment = task.comment.as_ref();

            println!("[+] Running Task: {}", task_kind);

            if task_name.is_some() {
                println!("    Name: {}", task_name.unwrap());
            }
            if task_comment.is_some() {
                println!("    Comment: {}", task_comment.unwrap());
            }

            let args = task.args.as_ref().unwrap();

            match task_kind.as_str() {
                "script" => {
                    let mut code = args.get("script").unwrap();

                    // string interpolation only works on cmds and scripts
                    let res = var_ops::interpolate_string(code, ctx);
                    code = &res;

                    py.run(code, None, Some(locals)).unwrap();
                }
                "cmd" => {
                    var_ops::interpolate_string(&args["cmd"], ctx);

                    let (stdout, stderr) = cmd::run(&args);
                    if !stdout.is_empty() {
                        println!("stdout: {}", stdout);
                    }

                    if !stderr.is_empty() {
                        println!("stderr: {}", stderr);
                    }
                }
                "fetch_url" => {
                    runtime.block_on(http::fetch_url(&args));
                }
                "set_var" => {
                    var_ops::set_var(&args, ctx);
                }
                _ => (),
            }
        });
    });

    Ok(())
}
