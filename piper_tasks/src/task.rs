use chrono::{DateTime, Utc};
use mlua::{Lua, Table, Value as LuaValue};

pub struct TaskMetadata {
    pub name: String,
    pub description: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub task_kind: String,
}

#[derive(Debug)]
pub enum TaskError {
    ExecutionError(String),
    ValidationError(String),
    ContextError(String),
    DuplicateTaskName(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskFlow {
    Sequential,  // Default - run in order
    Conditional, // If/else branching
    Dynamic,     // LLM-selected tasks
    Parallel,    // Run tasks concurrently
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDefinition {
    name: String,
    description: String,
    task: String,
    flow: Option<TaskFlow>,
    args: Option<HashMap<String, String>>,
    // For conditional branching
    if_condition: Option<String>,    // Lua expression to evaluate
    then_tasks: Option<Vec<String>>, // Task names to run if true
    else_tasks: Option<Vec<String>>, // Task names to run if false
    // For LLM task selection
    available_tasks: Option<Vec<String>>, // Tasks LLM can choose from
}

pub trait Task {
    fn validate(&self) -> Result<(), TaskError>;
    fn execute(&self, lua: &Lua, ctx: &Table, result_table: Table) -> Result<(), TaskError>;
    fn metadata(&self) -> &TaskMetadata;
}
