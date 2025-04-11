use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, Context};
use thiserror::Error;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: Option<String>,
    pub task_type: TaskType,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Cmd,
    Script,
    Llm,
    Http,
    Notify,
    SetVar,
    Lua,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    VarInterpolation(String),
}

impl Pipeline {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let file = PiperParser::parse(Rule::file, input)?
            .next()
            .unwrap();
        
        let mut pipeline_name = String::new();
        let mut metadata = HashMap::new();
        let mut tasks = Vec::new();
        
        for record in file.into_inner() {
            match record.as_rule() {
                Rule::pipeline => {
                    let mut inner_rules = record.into_inner();
                    let name = inner_rules.next().unwrap();
                    pipeline_name = name.as_str().to_string();
                    
                    for inner_rule in inner_rules {
                        match inner_rule.as_rule() {
                            Rule::metadata => {
                                for meta_pair in inner_rule.into_inner() {
                                    let mut pair_iter = meta_pair.into_inner();
                                    let key = pair_iter.next().unwrap().as_str();
                                    let value = parse_value(pair_iter.next().unwrap())?;
                                    metadata.insert(key.to_string(), value);
                                }
                            }
                            Rule::tasks => {
                                for task_rule in inner_rule.into_inner() {
                                    if task_rule.as_rule() == Rule::task {
                                        let task = parse_task(task_rule)?;
                                        tasks.push(task);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        
        let author = metadata.get("author").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });
        
        let description = metadata.get("description").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });
        
        let version = metadata.get("version").and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });
        
        Ok(Pipeline {
            name: pipeline_name,
            author,
            description,
            version,
            tasks,
        })
    }
}

fn parse_task(task_rule: pest::iterators::Pair<Rule>) -> Result<Task, ParseError> {
    let mut inner_rules = task_rule.into_inner();
    let task_type_rule = inner_rules.next().unwrap();
    let task_type = match task_type_rule.as_str() {
        "cmd" => TaskType::Cmd,
        "script" => TaskType::Script,
        "llm" => TaskType::Llm,
        "http" => TaskType::Http,
        "notify" => TaskType::Notify,
        "set_var" => TaskType::SetVar,
        "lua" => TaskType::Lua,
        _ => return Err(ParseError::InvalidTaskType(task_type_rule.as_str().to_string())),
    };
    
    let name_rule = inner_rules.next().unwrap();
    let name = if name_rule.as_rule() == Rule::identifier {
        Some(name_rule.as_str().to_string())
    } else {
        None
    };
    
    let properties_rule = if name.is_some() {
        inner_rules.next().unwrap()
    } else {
        name_rule
    };
    
    let mut properties = HashMap::new();
    for property in properties_rule.into_inner() {
        if property.as_rule() == Rule::task_property {
            let mut property_inner = property.into_inner();
            let key = property_inner.next().unwrap().as_str().to_string();
            let value = parse_value(property_inner.next().unwrap())?;
            properties.insert(key, value);
        }
    }
    
    Ok(Task {
        name,
        task_type,
        properties,
    })
}

fn parse_value(value_rule: pest::iterators::Pair<Rule>) -> Result<Value, ParseError> {
    match value_rule.as_rule() {
        Rule::string_literal => {
            let inner = value_rule.into_inner().next().unwrap();
            Ok(Value::String(inner.as_str().to_string()))
        }
        Rule::number => {
            let num_str = value_rule.as_str();
            let num = num_str.parse::<f64>().map_err(|_| {
                ParseError::InvalidValue {
                    field: "number".to_string(),
                    message: format!("Could not parse '{}' as a number", num_str),
                }
            })?;
            Ok(Value::Number(num))
        }
        Rule::boolean => {
            let bool_str = value_rule.as_str();
            let bool_val = bool_str == "true";
            Ok(Value::Boolean(bool_val))
        }
        Rule::var_interpolation => {
            let var_name = value_rule.as_str();
            Ok(Value::VarInterpolation(var_name.to_string()))
        }
        Rule::object => {
            let mut map = HashMap::new();
            for pair in value_rule.into_inner() {
                if pair.as_rule() == Rule::pair {
                    let mut pair_inner = pair.into_inner();
                    let key = pair_inner.next().unwrap().as_str().to_string();
                    let value = parse_value(pair_inner.next().unwrap())?;
                    map.insert(key, value);
                }
            }
            Ok(Value::Object(map))
        }
        Rule::array => {
            let mut values = Vec::new();
            for value in value_rule.into_inner() {
                values.push(parse_value(value)?);
            }
            Ok(Value::Array(values))
        }
        _ => Err(ParseError::InvalidValue {
            field: "value".to_string(),
            message: format!("Unexpected rule: {:?}", value_rule.as_rule()),
        }),
    }
}
