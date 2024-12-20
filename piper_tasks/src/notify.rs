use chrono::Utc;
use mlua::{Lua, Table};
use reqwest;
use serde_json::Value;
use serde_json::Value;
use std::collections::HashMap;

pub struct NotifyTask {
    args: HashMap<String, String>,
    metadata: TaskMetadata,
}

impl NotifyTask {
    pub fn new(name: String, description: String, args: HashMap<String, String>) -> Self {
        Self {
            args,
            metadata: TaskMetadata {
                name,
                description,
                timestamp: Utc::now(),
                task_type: "notify".to_string(),
            },
        }
    }
}

impl Task for NotifyTask {
    fn validate(&self) -> Result<(), TaskError> {
        if !self.args.contains_key("uri") {
            return Err(TaskError::ValidationError("Missing 'uri' argument".into()));
        }
        Ok(())
    }

    fn execute(&self, lua: &Lua, ctx: &Table, result_table: Table) -> Result<(), TaskError> {
        let uri = self
            .args
            .get("uri")
            .ok_or_else(|| TaskError::ExecutionError("Missing uri argument".into()))?;

        // Create JSON from args
        let json = serde_json::to_value(&self.args)
            .map_err(|e| TaskError::ExecutionError(format!("JSON serialization error: {}", e)))?;

        // Create a new runtime for async operations
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| TaskError::ExecutionError(format!("Failed to create runtime: {}", e)))?;

        // Execute the notification
        let result = rt.block_on(async {
            let client = reqwest::Client::new();
            let json_string = serde_json::to_string(&json).map_err(|e| {
                TaskError::ExecutionError(format!("JSON string conversion error: {}", e))
            })?;

            let resp = client
                .post(uri)
                .body(json_string)
                .send()
                .await
                .map_err(|e| TaskError::ExecutionError(format!("HTTP request failed: {}", e)))?;

            Ok::<reqwest::Response, TaskError>(resp)
        });

        // Store results
        match result {
            Ok(response) => {
                result_table
                    .set("status", "success")
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("uri", uri)
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("timestamp", self.metadata.timestamp.to_rfc3339())
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("task_type", &self.metadata.task_type)
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("status_code", response.status().as_u16())
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
            }
            Err(e) => {
                result_table
                    .set("status", "error")
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("error", e.to_string())
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("uri", uri)
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("timestamp", self.metadata.timestamp.to_rfc3339())
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("task_type", &self.metadata.task_type)
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn metadata(&self) -> &TaskMetadata {
        &self.metadata
    }
}
