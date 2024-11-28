use chrono::Utc;
use core::str;
use mlua::{Lua, Result as LuaResult, Table, Value as LuaValue};
use std::collections::HashMap;
use std::env;
use std::process::Command;

use crate::task::{Task, TaskError, TaskMetadata};

pub struct CommandTask {
    args: HashMap<String, String>,
    metadata: TaskMetadata,
}

impl CommandTask {
    pub fn new(name: String, description: Option<String>, args: HashMap<String, String>) -> Self {
        Self {
            args,
            metadata: TaskMetadata {
                name,
                description,
                timestamp: Utc::now(),
                task_kind: "cmd".to_string(),
            },
        }
    }
}

impl Task for CommandTask {
    fn validate(&self) -> Result<(), crate::task::TaskError> {
        todo!()
    }
    fn execute(&self, lua: &Lua, ctx: &Table, result_table: Table) -> Result<(), TaskError> {
        let cmd = self
            .args
            .get("cmd")
            .ok_or_else(|| TaskError::ExecutionError("Missing cmd argument".into()))?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| TaskError::ExecutionError(e.to_string()))?;

        let stdout = str::from_utf8(&output.stdout)
            .map_err(|e| TaskError::ExecutionError(e.to_string()))?
            .to_owned();

        let stderr = str::from_utf8(&output.stderr)
            .map_err(|e| TaskError::ExecutionError(e.to_string()))?
            .to_owned();

        result_table
            .set("stdout", stdout)
            .map_err(|e| TaskError::ContextError(e.to_string()))?;
        result_table
            .set("stderr", stderr)
            .map_err(|e| TaskError::ContextError(e.to_string()))?;
        result_table
            .set("timestamp", self.metadata.timestamp.to_rfc3339())
            .map_err(|e| TaskError::ContextError(e.to_string()))?;
        result_table
            .set("task_type", &self.metadata.task_type)
            .map_err(|e| TaskError::ContextError(e.to_string()))?;

        Ok(())
    }

    fn metadata(&self) -> &TaskMetadata {
        todo!()
    }
}

/// Runs a given command using the default system shell.
pub fn run(args: &HashMap<String, String>) -> (String, String) {
    let cmd = args.get("cmd").unwrap().to_owned();

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute process");
    let stdout = str::from_utf8(&output.stdout).unwrap().to_owned();
    let stderr = str::from_utf8(&output.stderr).unwrap().to_owned();

    (stdout, stderr)
}
