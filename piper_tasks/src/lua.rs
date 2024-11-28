pub struct LuaTask {
    args: HashMap<String, String>,
    metadata: TaskMetadata,
}

impl LuaTask {
    pub fn new(name: String, description: String, args: HashMap<String, String>) -> Self {
        Self {
            args,
            metadata: TaskMetadata {
                name,
                description,
                timestamp: Utc::now(),
                task_type: "lua".to_string(),
            },
        }
    }
}

impl Task for LuaTask {
    fn validate(&self) -> Result<(), TaskError> {
        match self.args.get("script") {
            Some(_) => Ok(()),
            None => Err(TaskError::ValidationError(
                "Missing 'script' argument".into(),
            )),
        }
    }

    fn execute(&self, lua: &Lua, ctx: &Table, result_table: Table) -> Result<(), TaskError> {
        let script = self
            .args
            .get("script")
            .ok_or_else(|| TaskError::ExecutionError("Missing script argument".into()))?;

        match lua.load(script).exec() {
            Ok(_) => {
                result_table
                    .set("status", "success")
                    .map_err(|e| TaskError::ContextError(e.to_string()))?;
                result_table
                    .set("script", script)
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
                    .set("script", script)
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
