use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, Context as AnyhowContext};
use thiserror::Error;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct PiperParser;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Pest parsing error: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
    
    #[error("Invalid task type: {0}")]
    InvalidTaskType(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid value for field {field}: {message}")]
    InvalidValue { field: String, message: String },
}

// Updated Pipeline structure for the new syntax
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub metadata: HashMap<String, Value>,
    pub data_literals: HashMap<String, Value>,
    pub tasks: HashMap<String, Task>,
    pub flow: Option<Flow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub default_value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Cmd,
    Script,
    Llm,
    Http,
    Notify,
    SetVar,
    Lua,
    MetaTask,
    GenerateTasks,
    GenerateFlow,
}

impl TaskType {
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        match s {
            "cmd" => Ok(TaskType::Cmd),
            "script" => Ok(TaskType::Script),
            "llm" => Ok(TaskType::Llm),
            "http" => Ok(TaskType::Http),
            "notify" => Ok(TaskType::Notify),
            "set_var" => Ok(TaskType::SetVar),
            "lua" => Ok(TaskType::Lua),
            "meta_task" => Ok(TaskType::MetaTask),
            "generate_tasks" => Ok(TaskType::GenerateTasks),
            "generate_flow" => Ok(TaskType::GenerateFlow),
            _ => Err(ParseError::InvalidTaskType(s.to_string())),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            TaskType::Cmd => "cmd".to_string(),
            TaskType::Script => "script".to_string(),
            TaskType::Llm => "llm".to_string(),
            TaskType::Http => "http".to_string(),
            TaskType::Notify => "notify".to_string(),
            TaskType::SetVar => "set_var".to_string(),
            TaskType::Lua => "lua".to_string(),
            TaskType::MetaTask => "meta_task".to_string(),
            TaskType::GenerateTasks => "generate_tasks".to_string(),
            TaskType::GenerateFlow => "generate_flow".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_type: TaskType,
    pub arguments: Vec<Argument>,
    pub named_arguments: HashMap<String, Value>,
    pub meta_task_config: Option<MetaTaskConfig>,
    pub generate_tasks_config: Option<GenerateTasksConfig>,
    pub generate_flow_config: Option<GenerateFlowConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTaskConfig {
    pub task: String,
    pub data_shape: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTasksConfig {
    pub meta_tasks: Vec<String>,
    pub custom_tasks: Vec<String>,
    pub model: String,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateFlowConfig {
    pub tasks: String,
    pub constraints: String,
    pub description: String,
    pub model: String,
    pub visualization: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub name: Option<String>,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Flow {
    Sequential {
        items: Vec<FlowItem>,
    },
    Parallel {
        items: Vec<FlowItem>,
    },
    Conditional {
        condition: Condition,
        if_true: Box<FlowItem>,
        if_false: Option<Box<FlowItem>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FlowItem {
    Task(String),
    Flow(Flow),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition {
    Comparison {
        left: Value,
        operator: ComparisonOperator,
        right: Value,
    },
    Boolean(bool),
    VarInterpolation(String),
    LogicalOperation {
        left: Box<Condition>,
        operator: LogicalOperator,
        right: Box<Condition>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    MultilineString(String),
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    VarInterpolation(String),
    PropertyAccess {
        base: String,
        path: Vec<String>,
    },
    FallbackExpr {
        primary: Box<Value>,
        fallback: Box<Value>,
    },
    FunctionCall {
        function: String,
        arguments: Vec<Argument>,
    },
    ConditionalValue {
        condition: Box<Condition>,
        if_true: Box<Value>,
        if_false: Box<Value>,
    },
}

impl Pipeline {
    // Generate a pipeline from the meta-pipeline
    pub fn generate_pipeline(&self, regenerate: bool) -> Result<String, ParseError> {
        // Check if a generated pipeline already exists
        let generated_path = self.get_generated_pipeline_path();
        
        if !regenerate && generated_path.exists() {
            // Use the existing generated pipeline
            let pipeline_content = fs::read_to_string(&generated_path)
                .map_err(|e| ParseError::InvalidValue {
                    field: "file_read".to_string(),
                    message: format!("Failed to read generated pipeline: {}", e),
                })?;
            return Ok(pipeline_content);
        }
        
        // Generate a new pipeline
        let pipeline_content = self.generate_new_pipeline()?;
        
        // Write the generated pipeline to disk
        fs::create_dir_all(generated_path.parent().unwrap())
            .map_err(|e| ParseError::InvalidValue {
                field: "directory_create".to_string(),
                message: format!("Failed to create directory: {}", e),
            })?;
        
        fs::write(&generated_path, &pipeline_content)
            .map_err(|e| ParseError::InvalidValue {
                field: "file_write".to_string(),
                message: format!("Failed to write generated pipeline: {}", e),
            })?;
        
        Ok(pipeline_content)
    }
    
    // Get the path where the generated pipeline should be stored
    fn get_generated_pipeline_path(&self) -> PathBuf {
        let generated_dir = Path::new("generated");
        generated_dir.join(format!("{}.piper", self.name))
    }
    
    // Generate a new pipeline based on the meta-pipeline
    fn generate_new_pipeline(&self) -> Result<String, ParseError> {
        // In a real implementation, this would call the LLM to generate the pipeline
        // For now, we'll just create a simple template
        
        let mut pipeline = String::new();
        
        // Add header
        pipeline.push_str(&format!("// Generated from meta-pipeline: {}\n", self.name));
        pipeline.push_str("// This pipeline was automatically generated from meta-tasks and constraints\n\n");
        
        // Add pipeline definition
        pipeline.push_str(&format!("pipeline {}(", self.name));
        
        // Add parameters
        let params: Vec<String> = self.parameters.iter()
            .map(|p| {
                if let Some(default_value) = &p.default_value {
                    format!("{}=\"{}\"", p.name, value_to_string(default_value))
                } else {
                    p.name.clone()
                }
            })
            .collect();
        pipeline.push_str(&params.join(", "));
        pipeline.push_str(") {\n");
        
        // Add metadata
        pipeline.push_str("  meta {\n");
        for (key, value) in &self.metadata {
            pipeline.push_str(&format!("    {}: \"{}\"\n", key, value_to_string(value)));
        }
        pipeline.push_str("  }\n\n");
        
        // Add data literals
        for (name, value) in &self.data_literals {
            pipeline.push_str(&format!("  {} = {}\n", name, value_to_string(value)));
        }
        pipeline.push_str("\n");
        
        // Add tasks
        for (name, task) in &self.tasks {
            // Check if this is a meta-task
            if let Some(meta_config) = &task.meta_task_config {
                // Generate a task based on the meta-task description
                pipeline.push_str(&format!("  // Generated from meta-task: {}\n", name));
                pipeline.push_str(&format!("  {} = cmd(\n", name));
                pipeline.push_str(&format!("    command=\"echo 'Implementing: {}'\",\n", meta_config.task));
                pipeline.push_str(&format!("    description=\"{}\"\n", meta_config.task));
                pipeline.push_str("  )\n\n");
            } else {
                // Regular task
                pipeline.push_str(&format!("  {} = {}(\n", name, task.task_type));
                for (key, value) in &task.named_arguments {
                    pipeline.push_str(&format!("    {}={},\n", key, value_to_string(value)));
                }
                pipeline.push_str("  )\n\n");
            }
        }
        
        // Add flow
        if let Some(flow) = &self.flow {
            pipeline.push_str("  // Flow\n");
            pipeline.push_str("  flow:\n");
            pipeline.push_str(&format!("    {}\n", flow_to_string(flow)));
        } else {
            // Generate a simple flow
            pipeline.push_str("  // Generated flow\n");
            pipeline.push_str("  flow:\n");
            
            // Create a simple sequential flow of all tasks
            let task_names: Vec<String> = self.tasks.keys().cloned().collect();
            pipeline.push_str(&format!("    {}\n", task_names.join(" > ")));
        }
        
        // Close the pipeline
        pipeline.push_str("}\n");
        
        Ok(pipeline)
    }
    
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let file = PiperParser::parse(Rule::file, input)?
            .next()
            .unwrap();
        
        let mut pipeline_name = String::new();
        let mut parameters = Vec::new();
        let mut metadata = HashMap::new();
        let mut data_literals = HashMap::new();
        let mut tasks = HashMap::new();
        let mut flow = None;
        
        for record in file.into_inner() {
            match record.as_rule() {
                Rule::pipeline => {
                    let mut inner_rules = record.into_inner();
                    let name_rule = inner_rules.next().unwrap();
                    pipeline_name = name_rule.as_str().to_string();
                    
                    // Parse optional parameters
                    let next_rule = inner_rules.next().unwrap();
                    if next_rule.as_rule() == Rule::parameters {
                        parameters = parse_parameters(next_rule)?;
                        
                        // Continue with the rest of the pipeline
                        for rule in inner_rules {
                            match rule.as_rule() {
                                Rule::metadata => {
                                    metadata = parse_metadata(rule)?;
                                },
                                Rule::data_literal => {
                                    let (name, value) = parse_data_literal(rule)?;
                                    data_literals.insert(name, value);
                                },
                                Rule::task_definition => {
                                    let (name, task) = parse_task_definition(rule)?;
                                    tasks.insert(name, task);
                                },
                                Rule::flow_definition => {
                                    flow = Some(parse_flow_definition(rule)?);
                                },
                                _ => {}
                            }
                        }
                    } else {
                        // No parameters, continue with the pipeline content
                        match next_rule.as_rule() {
                            Rule::metadata => {
                                metadata = parse_metadata(next_rule)?;
                            },
                            Rule::data_literal => {
                                let (name, value) = parse_data_literal(next_rule)?;
                                data_literals.insert(name, value);
                            },
                            Rule::task_definition => {
                                let (name, task) = parse_task_definition(next_rule)?;
                                tasks.insert(name, task);
                            },
                            Rule::flow_definition => {
                                flow = Some(parse_flow_definition(next_rule)?);
                            },
                            _ => {}
                        }
                        
                        // Process remaining rules
                        for rule in inner_rules {
                            match rule.as_rule() {
                                Rule::metadata => {
                                    metadata = parse_metadata(rule)?;
                                },
                                Rule::data_literal => {
                                    let (name, value) = parse_data_literal(rule)?;
                                    data_literals.insert(name, value);
                                },
                                Rule::task_definition => {
                                    let (name, task) = parse_task_definition(rule)?;
                                    tasks.insert(name, task);
                                },
                                Rule::flow_definition => {
                                    flow = Some(parse_flow_definition(rule)?);
                                },
                                _ => {}
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        
        Ok(Pipeline {
            name: pipeline_name,
            parameters,
            metadata,
            data_literals,
            tasks,
            flow,
        })
    }
}

fn parse_parameters(params_rule: pest::iterators::Pair<Rule>) -> Result<Vec<Parameter>, ParseError> {
    let mut parameters = Vec::new();
    
    for param_rule in params_rule.into_inner() {
        if param_rule.as_rule() == Rule::parameter {
            let mut param_inner = param_rule.into_inner();
            let name = param_inner.next().unwrap().as_str().to_string();
            
            let default_value = if let Some(value_rule) = param_inner.next() {
                Some(parse_value(value_rule)?)
            } else {
                None
            };
            
            parameters.push(Parameter {
                name,
                default_value,
            });
        }
    }
    
    Ok(parameters)
}

fn parse_metadata(metadata_rule: pest::iterators::Pair<Rule>) -> Result<HashMap<String, Value>, ParseError> {
    let mut metadata = HashMap::new();
    
    for pair in metadata_rule.into_inner() {
        if pair.as_rule() == Rule::pair {
            let mut pair_inner = pair.into_inner();
            let key = pair_inner.next().unwrap().as_str().to_string();
            let value = parse_value(pair_inner.next().unwrap())?;
            metadata.insert(key, value);
        }
    }
    
    Ok(metadata)
}

fn parse_data_literal(data_rule: pest::iterators::Pair<Rule>) -> Result<(String, Value), ParseError> {
    let mut data_inner = data_rule.into_inner();
    let name = data_inner.next().unwrap().as_str().to_string();
    let value = parse_value(data_inner.next().unwrap())?;
    
    Ok((name, value))
}

fn parse_task_definition(task_rule: pest::iterators::Pair<Rule>) -> Result<(String, Task), ParseError> {
    let mut task_inner = task_rule.into_inner();
    let name = task_inner.next().unwrap().as_str().to_string();
    
    let task_content = task_inner.next().unwrap();
    let task = match task_content.as_rule() {
        Rule::function_call => parse_function_call(task_content)?,
        Rule::inline_command => parse_inline_command(task_content)?,
        _ => return Err(ParseError::InvalidValue {
            field: "task_definition".to_string(),
            message: format!("Unexpected rule: {:?}", task_content.as_rule()),
        }),
    };
    
    Ok((name, task))
}

fn parse_function_call(call_rule: pest::iterators::Pair<Rule>) -> Result<Task, ParseError> {
    let mut call_inner = call_rule.into_inner();
    let task_type = call_inner.next().unwrap().as_str().to_string();
    
    let mut arguments = Vec::new();
    let mut named_arguments = HashMap::new();
    
    // Parse arguments
    for arg_rule in call_inner {
        if arg_rule.as_rule() == Rule::argument {
            let mut arg_inner = arg_rule.into_inner();
            let first = arg_inner.next().unwrap();
            
            if first.as_rule() == Rule::identifier {
                // Named argument
                let name = first.as_str().to_string();
                let value = parse_value(arg_inner.next().unwrap())?;
                named_arguments.insert(name.clone(), value.clone());
                arguments.push(Argument {
                    name: Some(name),
                    value,
                });
            } else {
                // Positional argument
                let value = parse_value(first)?;
                arguments.push(Argument {
                    name: None,
                    value,
                });
            }
        }
    }
    
    // Check for special task types
    let meta_task_config = if task_type == "meta_task" {
        // Parse meta_task arguments
        let mut task_desc = String::new();
        let mut data_shape = String::new();
        
        for arg in &arguments {
            if let Some(name) = &arg.name {
                if name == "task" {
                    if let Value::String(s) = &arg.value {
                        task_desc = s.clone();
                    } else if let Value::MultilineString(s) = &arg.value {
                        task_desc = s.clone();
                    }
                } else if name == "data_shape" {
                    if let Value::String(s) = &arg.value {
                        data_shape = s.clone();
                    } else if let Value::MultilineString(s) = &arg.value {
                        data_shape = s.clone();
                    }
                }
            }
        }
        
        Some(MetaTaskConfig {
            task: task_desc,
            data_shape,
        })
    } else {
        None
    };
    
    let generate_tasks_config = if task_type == "generate_tasks" {
        // Parse generate_tasks arguments
        let mut meta_tasks = Vec::new();
        let mut custom_tasks = Vec::new();
        let mut model = String::new();
        let mut style = None;
        
        for arg in &arguments {
            if let Some(name) = &arg.name {
                if name == "meta_tasks" {
                    if let Value::Array(arr) = &arg.value {
                        for item in arr {
                            if let Value::String(s) = item {
                                meta_tasks.push(s.clone());
                            } else if let Value::VarInterpolation(s) = item {
                                meta_tasks.push(s.clone());
                            }
                        }
                    }
                } else if name == "custom_tasks" {
                    if let Value::Array(arr) = &arg.value {
                        for item in arr {
                            if let Value::String(s) = item {
                                custom_tasks.push(s.clone());
                            } else if let Value::VarInterpolation(s) = item {
                                custom_tasks.push(s.clone());
                            }
                        }
                    }
                } else if name == "model" {
                    if let Value::String(s) = &arg.value {
                        model = s.clone();
                    } else if let Value::VarInterpolation(s) = &arg.value {
                        model = s.clone();
                    }
                } else if name == "style" {
                    if let Value::String(s) = &arg.value {
                        style = Some(s.clone());
                    }
                }
            }
        }
        
        Some(GenerateTasksConfig {
            meta_tasks,
            custom_tasks,
            model,
            style,
        })
    } else {
        None
    };
    
    let generate_flow_config = if task_type == "generate_flow" {
        // Parse generate_flow arguments
        let mut tasks = String::new();
        let mut constraints = String::new();
        let mut description = String::new();
        let mut model = String::new();
        let mut visualization = None;
        
        for arg in &arguments {
            if let Some(name) = &arg.name {
                if name == "tasks" {
                    if let Value::VarInterpolation(s) = &arg.value {
                        tasks = s.clone();
                    }
                } else if name == "constraints" {
                    if let Value::VarInterpolation(s) = &arg.value {
                        constraints = s.clone();
                    }
                } else if name == "description" {
                    if let Value::VarInterpolation(s) = &arg.value {
                        description = s.clone();
                    }
                } else if name == "model" {
                    if let Value::String(s) = &arg.value {
                        model = s.clone();
                    } else if let Value::VarInterpolation(s) = &arg.value {
                        model = s.clone();
                    }
                } else if name == "visualization" {
                    if let Value::Boolean(b) = &arg.value {
                        visualization = Some(*b);
                    }
                }
            }
        }
        
        Some(GenerateFlowConfig {
            tasks,
            constraints,
            description,
            model,
            visualization,
        })
    } else {
        None
    };
    
    Ok(Task {
        task_type,
        arguments,
        named_arguments,
        meta_task_config,
        generate_tasks_config,
        generate_flow_config,
    })
}

fn parse_inline_command(cmd_rule: pest::iterators::Pair<Rule>) -> Result<Task, ParseError> {
    let mut cmd_inner = cmd_rule.into_inner();
    let command = cmd_inner.next().unwrap().as_str().to_string();
    
    let mut arguments = Vec::new();
    let mut named_arguments = HashMap::new();
    
    // Add command as first argument
    let command_value = Value::String(command);
    arguments.push(Argument {
        name: Some("command".to_string()),
        value: command_value.clone(),
    });
    named_arguments.insert("command".to_string(), command_value);
    
    // Check for output redirection
    if let Some(output_rule) = cmd_inner.next() {
        let output = output_rule.as_str().to_string();
        let output_value = Value::String(output);
        arguments.push(Argument {
            name: Some("output".to_string()),
            value: output_value.clone(),
        });
        named_arguments.insert("output".to_string(), output_value);
    }
    
    Ok(Task {
        task_type: "cmd".to_string(),
        arguments,
        named_arguments,
        meta_task_config: None,
        generate_tasks_config: None,
        generate_flow_config: None,
    })
}

fn parse_flow_definition(flow_rule: pest::iterators::Pair<Rule>) -> Result<Flow, ParseError> {
    let flow_expr = flow_rule.into_inner().next().unwrap();
    parse_flow_expr(flow_expr)
}

fn parse_flow_expr(expr_rule: pest::iterators::Pair<Rule>) -> Result<Flow, ParseError> {
    let mut items = Vec::new();
    let mut expr_inner = expr_rule.into_inner();
    
    // Parse the first item
    let first_item = expr_inner.next().unwrap();
    let first_flow_item = parse_flow_item(first_item)?;
    items.push(first_flow_item);
    
    // Check for additional items with operators
    while let Some(op_rule) = expr_inner.next() {
        if op_rule.as_rule() == Rule::flow_operator {
            // Get the next item after the operator
            if let Some(next_item) = expr_inner.next() {
                let next_flow_item = parse_flow_item(next_item)?;
                items.push(next_flow_item);
            }
        }
    }
    
    // If there's only one item, return it directly
    if items.len() == 1 {
        match &items[0] {
            FlowItem::Flow(flow) => return Ok(flow.clone()),
            FlowItem::Task(_) => {
                return Ok(Flow::Sequential { items });
            }
        }
    }
    
    // Otherwise, return a sequential flow with all items
    Ok(Flow::Sequential { items })
}

fn parse_flow_item(item_rule: pest::iterators::Pair<Rule>) -> Result<FlowItem, ParseError> {
    match item_rule.as_rule() {
        Rule::identifier => {
            let task_name = item_rule.as_str().to_string();
            Ok(FlowItem::Task(task_name))
        },
        Rule::parallel_flow => {
            let mut parallel_items = Vec::new();
            
            for item in item_rule.into_inner() {
                let flow_item = parse_flow_item(item)?;
                parallel_items.push(flow_item);
            }
            
            Ok(FlowItem::Flow(Flow::Parallel { items: parallel_items }))
        },
        Rule::conditional_flow => {
            let mut cond_inner = item_rule.into_inner();
            let condition = parse_condition(cond_inner.next().unwrap())?;
            let if_true = Box::new(parse_flow_item(cond_inner.next().unwrap())?);
            
            let if_false = if let Some(else_item) = cond_inner.next() {
                Some(Box::new(parse_flow_item(else_item)?))
            } else {
                None
            };
            
            Ok(FlowItem::Flow(Flow::Conditional {
                condition,
                if_true,
                if_false,
            }))
        },
        Rule::flow_expr => {
            let flow = parse_flow_expr(item_rule)?;
            Ok(FlowItem::Flow(flow))
        },
        _ => Err(ParseError::InvalidValue {
            field: "flow_item".to_string(),
            message: format!("Unexpected rule: {:?}", item_rule.as_rule()),
        }),
    }
}

fn parse_condition(cond_rule: pest::iterators::Pair<Rule>) -> Result<Condition, ParseError> {
    match cond_rule.as_rule() {
        Rule::condition => {
            let mut terms = Vec::new();
            let mut operators = Vec::new();
            
            for (i, part) in cond_rule.into_inner().enumerate() {
                if i % 2 == 0 {
                    // Term
                    terms.push(parse_condition_term(part)?);
                } else {
                    // Operator
                    let op = match part.as_str() {
                        "&&" => LogicalOperator::And,
                        "||" => LogicalOperator::Or,
                        _ => return Err(ParseError::InvalidValue {
                            field: "logical_operator".to_string(),
                            message: format!("Unknown logical operator: {}", part.as_str()),
                        }),
                    };
                    operators.push(op);
                }
            }
            
            // If there's only one term, return it directly
            if terms.len() == 1 {
                return Ok(terms[0].clone());
            }
            
            // Otherwise, build a tree of logical operations
            let mut result = terms[0].clone();
            
            for i in 0..operators.len() {
                result = Condition::LogicalOperation {
                    left: Box::new(result),
                    operator: operators[i].clone(),
                    right: Box::new(terms[i + 1].clone()),
                };
            }
            
            Ok(result)
        },
        _ => Err(ParseError::InvalidValue {
            field: "condition".to_string(),
            message: format!("Unexpected rule: {:?}", cond_rule.as_rule()),
        }),
    }
}

fn parse_condition_term(term_rule: pest::iterators::Pair<Rule>) -> Result<Condition, ParseError> {
    match term_rule.as_rule() {
        Rule::condition_term => {
            let inner = term_rule.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::comparison => parse_comparison(inner),
                Rule::boolean => {
                    let bool_val = inner.as_str() == "true";
                    Ok(Condition::Boolean(bool_val))
                },
                Rule::var_interpolation => {
                    let var_name = inner.as_str().to_string();
                    Ok(Condition::VarInterpolation(var_name))
                },
                Rule::condition => parse_condition(inner),
                _ => Err(ParseError::InvalidValue {
                    field: "condition_term".to_string(),
                    message: format!("Unexpected rule: {:?}", inner.as_rule()),
                }),
            }
        },
        _ => Err(ParseError::InvalidValue {
            field: "condition_term".to_string(),
            message: format!("Unexpected rule: {:?}", term_rule.as_rule()),
        }),
    }
}

fn parse_comparison(comp_rule: pest::iterators::Pair<Rule>) -> Result<Condition, ParseError> {
    let mut comp_inner = comp_rule.into_inner();
    let left_value = parse_value(comp_inner.next().unwrap())?;
    
    let op_rule = comp_inner.next().unwrap();
    let operator = match op_rule.as_str() {
        "==" => ComparisonOperator::Equal,
        "!=" => ComparisonOperator::NotEqual,
        ">" => ComparisonOperator::GreaterThan,
        "<" => ComparisonOperator::LessThan,
        ">=" => ComparisonOperator::GreaterThanOrEqual,
        "<=" => ComparisonOperator::LessThanOrEqual,
        _ => return Err(ParseError::InvalidValue {
            field: "comparison_operator".to_string(),
            message: format!("Unknown comparison operator: {}", op_rule.as_str()),
        }),
    };
    
    let right_value = parse_value(comp_inner.next().unwrap())?;
    
    Ok(Condition::Comparison {
        left: left_value,
        operator,
        right: right_value,
    })
}

// Helper function to convert a Value to a string representation
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s),
        Value::MultilineString(s) => format!("\"\"\"{}\"\"\"", s),
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Object(map) => {
            let pairs: Vec<String> = map.iter()
                .map(|(k, v)| format!("{}: {}", k, value_to_string(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        },
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter()
                .map(|v| value_to_string(v))
                .collect();
            format!("[{}]", items.join(", "))
        },
        Value::VarInterpolation(v) => format!("#{{{}}}", v),
        Value::PropertyAccess { base, path } => {
            let path_str = path.join(".");
            format!("#{{{}.{}}}", base, path_str)
        },
        Value::FallbackExpr { primary, fallback } => {
            format!("#{{{} || {}}}", value_to_string(primary), value_to_string(fallback))
        },
        Value::FunctionCall { function, arguments } => {
            let args: Vec<String> = arguments.iter()
                .map(|arg| {
                    if let Some(name) = &arg.name {
                        format!("{}={}", name, value_to_string(&arg.value))
                    } else {
                        value_to_string(&arg.value)
                    }
                })
                .collect();
            format!("{}({})", function, args.join(", "))
        },
        Value::ConditionalValue { condition, if_true, if_false } => {
            format!("{} ? {} : {}", 
                condition_to_string(condition), 
                value_to_string(if_true), 
                value_to_string(if_false))
        },
    }
}

// Helper function to convert a Condition to a string representation
fn condition_to_string(condition: &Condition) -> String {
    match condition {
        Condition::Comparison { left, operator, right } => {
            let op_str = match operator {
                ComparisonOperator::Equal => "==",
                ComparisonOperator::NotEqual => "!=",
                ComparisonOperator::GreaterThan => ">",
                ComparisonOperator::LessThan => "<",
                ComparisonOperator::GreaterThanOrEqual => ">=",
                ComparisonOperator::LessThanOrEqual => "<=",
            };
            format!("{} {} {}", value_to_string(left), op_str, value_to_string(right))
        },
        Condition::Boolean(b) => b.to_string(),
        Condition::VarInterpolation(v) => format!("#{{{}}}", v),
        Condition::LogicalOperation { left, operator, right } => {
            let op_str = match operator {
                LogicalOperator::And => "&&",
                LogicalOperator::Or => "||",
            };
            format!("({} {} {})", condition_to_string(left), op_str, condition_to_string(right))
        },
    }
}

// Helper function to convert a Flow to a string representation
fn flow_to_string(flow: &Flow) -> String {
    match flow {
        Flow::Sequential { items } => {
            let item_strs: Vec<String> = items.iter()
                .map(|item| flow_item_to_string(item))
                .collect();
            item_strs.join(" > ")
        },
        Flow::Parallel { items } => {
            let item_strs: Vec<String> = items.iter()
                .map(|item| flow_item_to_string(item))
                .collect();
            format!("[{}]", item_strs.join(", "))
        },
        Flow::Conditional { condition, if_true, if_false } => {
            if let Some(else_item) = if_false {
                format!("({} ? {} : {})", 
                    condition_to_string(condition), 
                    flow_item_to_string(if_true), 
                    flow_item_to_string(else_item))
            } else {
                format!("({} ? {})", 
                    condition_to_string(condition), 
                    flow_item_to_string(if_true))
            }
        },
    }
}

// Helper function to convert a FlowItem to a string representation
fn flow_item_to_string(item: &FlowItem) -> String {
    match item {
        FlowItem::Task(name) => name.clone(),
        FlowItem::Flow(flow) => flow_to_string(flow),
    }
}

fn parse_value(value_rule: pest::iterators::Pair<Rule>) -> Result<Value, ParseError> {
    match value_rule.as_rule() {
        Rule::value => {
            let inner = value_rule.into_inner().next().unwrap();
            parse_value(inner)
        },
        Rule::basic_value => {
            let inner = value_rule.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::string_literal => {
                    let content = inner.into_inner().next().unwrap().as_str().to_string();
                    Ok(Value::String(content))
                },
                Rule::multiline_string => {
                    let content = inner.into_inner().next().unwrap().as_str().to_string();
                    Ok(Value::MultilineString(content))
                },
                Rule::number => {
                    let num_str = inner.as_str();
                    let num = num_str.parse::<f64>().map_err(|_| {
                        ParseError::InvalidValue {
                            field: "number".to_string(),
                            message: format!("Could not parse '{}' as a number", num_str),
                        }
                    })?;
                    Ok(Value::Number(num))
                },
                Rule::boolean => {
                    let bool_val = inner.as_str() == "true";
                    Ok(Value::Boolean(bool_val))
                },
                Rule::var_interpolation => {
                    let var_content = inner.into_inner().next().unwrap();
                    match var_content.as_rule() {
                        Rule::identifier => {
                            Ok(Value::VarInterpolation(var_content.as_str().to_string()))
                        },
                        Rule::property_access => {
                            let mut path_parts = Vec::new();
                            let mut base = String::new();
                            
                            for (i, part) in var_content.into_inner().enumerate() {
                                if i == 0 {
                                    base = part.as_str().to_string();
                                } else {
                                    path_parts.push(part.as_str().to_string());
                                }
                            }
                            
                            Ok(Value::PropertyAccess {
                                base,
                                path: path_parts,
                            })
                        },
                        Rule::fallback_expr => {
                            let mut fallback_inner = var_content.into_inner();
                            let primary = parse_value(fallback_inner.next().unwrap())?;
                            let fallback = parse_value(fallback_inner.next().unwrap())?;
                            
                            Ok(Value::FallbackExpr {
                                primary: Box::new(primary),
                                fallback: Box::new(fallback),
                            })
                        },
                        _ => Err(ParseError::InvalidValue {
                            field: "var_interpolation".to_string(),
                            message: format!("Unexpected rule: {:?}", var_content.as_rule()),
                        }),
                    }
                },
                Rule::function_call => {
                    let mut call_inner = inner.into_inner();
                    let function = call_inner.next().unwrap().as_str().to_string();
                    
                    let mut arguments = Vec::new();
                    
                    for arg_rule in call_inner {
                        if arg_rule.as_rule() == Rule::argument {
                            let mut arg_inner = arg_rule.into_inner();
                            let first = arg_inner.next().unwrap();
                            
                            if first.as_rule() == Rule::identifier {
                                // Named argument
                                let name = first.as_str().to_string();
                                let value = parse_value(arg_inner.next().unwrap())?;
                                arguments.push(Argument {
                                    name: Some(name),
                                    value,
                                });
                            } else {
                                // Positional argument
                                let value = parse_value(first)?;
                                arguments.push(Argument {
                                    name: None,
                                    value,
                                });
                            }
                        }
                    }
                    
                    Ok(Value::FunctionCall {
                        function,
                        arguments,
                    })
                },
                Rule::identifier => {
                    Ok(Value::VarInterpolation(inner.as_str().to_string()))
                },
                _ => Err(ParseError::InvalidValue {
                    field: "basic_value".to_string(),
                    message: format!("Unexpected rule: {:?}", inner.as_rule()),
                }),
            }
        },
        Rule::object => {
            let mut map = HashMap::new();
            
            for pair in value_rule.into_inner() {
                if pair.as_rule() == Rule::pair {
                    let mut pair_inner = pair.into_inner();
                    let key_rule = pair_inner.next().unwrap();
                    let key = match key_rule.as_rule() {
                        Rule::identifier => key_rule.as_str().to_string(),
                        Rule::string_literal => {
                            key_rule.into_inner().next().unwrap().as_str().to_string()
                        },
                        _ => return Err(ParseError::InvalidValue {
                            field: "object_key".to_string(),
                            message: format!("Unexpected rule: {:?}", key_rule.as_rule()),
                        }),
                    };
                    
                    let value = parse_value(pair_inner.next().unwrap())?;
                    map.insert(key, value);
                }
            }
            
            Ok(Value::Object(map))
        },
        Rule::array => {
            let mut values = Vec::new();
            
            for item in value_rule.into_inner() {
                values.push(parse_value(item)?);
            }
            
            Ok(Value::Array(values))
        },
        Rule::conditional_value => {
            let mut cond_inner = value_rule.into_inner();
            let condition = parse_condition(cond_inner.next().unwrap())?;
            let if_true = parse_value(cond_inner.next().unwrap())?;
            let if_false = parse_value(cond_inner.next().unwrap())?;
            
            Ok(Value::ConditionalValue {
                condition: Box::new(condition),
                if_true: Box::new(if_true),
                if_false: Box::new(if_false),
            })
        },
        _ => Err(ParseError::InvalidValue {
            field: "value".to_string(),
            message: format!("Unexpected rule: {:?}", value_rule.as_rule()),
        }),
    }
}
