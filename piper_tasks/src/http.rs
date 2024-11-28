use chrono::Utc;
use mlua::{Lua, Table};
use reqwest::{self, Method};
use serde_json::Value;
use std::collections::HashMap;

pub struct HttpTask {
    args: HashMap<String, String>,
    metadata: TaskMetadata,
}

impl HttpTask {
    pub fn new(name: String, description: String, args: HashMap<String, String>) -> Self {
        Self {
            args,
            metadata: TaskMetadata {
                name,
                description,
                timestamp: Utc::now(),
                task_type: "http".to_string(),
            },
        }
    }
}

impl Task for HttpTask {
    fn validate(&self) -> Result<(), TaskError> {
        if !self.args.contains_key("url") {
            return Err(TaskError::ValidationError("Missing 'url' argument".into()));
        }
        if !self.args.contains_key("method") {
            return Err(TaskError::ValidationError(
                "Missing 'method' argument".into(),
            ));
        }

        // Validate HTTP method
        let method = self.args.get("method").unwrap().to_uppercase();
        match method.as_str() {
            "GET" | "POST" => Ok(()),
            _ => Err(TaskError::ValidationError(
                "Invalid HTTP method. Supported: GET, POST".into(),
            )),
        }
    }

    fn execute(&self, lua: &Lua, ctx: &Table, result_table: Table) -> Result<(), TaskError> {
        let url = self
            .args
            .get("url")
            .ok_or_else(|| TaskError::ExecutionError("Missing url argument".into()))?;
        let method = self
            .args
            .get("method")
            .ok_or_else(|| TaskError::ExecutionError("Missing method argument".into()))?
            .to_uppercase();

        // Create runtime for async operations
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| TaskError::ExecutionError(format!("Failed to create runtime: {}", e)))?;

        // Execute the HTTP request
        let result = rt.block_on(async {
            let client = reqwest::Client::new();
            let mut request = client.request(
                Method::from_bytes(method.as_bytes())
                    .map_err(|e| TaskError::ExecutionError(format!("Invalid method: {}", e)))?,
                url,
            );

            // Add headers if specified
            if let Some(headers) = self.args.get("headers") {
                if let Ok(headers_map) = serde_json::from_str::<HashMap<String, String>>(headers) {
                    for (key, value) in headers_map {
                        request = request.header(key, value);
                    }
                }
            }

            // Add body for POST requests
            if method == "POST" {
                if let Some(body) = self.args.get("body") {
                    request = request.body(body.clone());
                }
            }

            let response = request
                .send()
                .await
                .map_err(|e| TaskError::ExecutionError(format!("Request failed: {}", e)))?;

            let status = response.status();
            let headers = response.headers().clone();
            let body = response.text().await.map_err(|e| {
                TaskError::ExecutionError(format!("Failed to read response: {}", e))
            })?;

            Ok::<(reqwest::StatusCode, reqwest::header::HeaderMap, String), TaskError>((
                status, headers, body,
            ))
        });

        // Store results
        match result {
            Ok((status, headers, body)) => {
                result_table
                    .set("status", "success")
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("url", url)
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("method", method)
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("status_code", status.as_u16())
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("body", body)
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;

                // Convert headers to a table
                let headers_table = lua
                    .create_table()
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                for (key, value) in headers.iter() {
                    if let Ok(value_str) = value.to_str() {
                        headers_table
                            .set(key.as_str(), value_str)
                            .map_err(|e| TaskError::ContextError(e.to_string()))?;
                    }
                }
                result_table
                    .set("headers", headers_table)
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;

                result_table
                    .set("timestamp", self.metadata.timestamp.to_rfc3339())
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("task_type", &self.metadata.task_type)
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
                    .set("url", url)
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("method", method)
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
