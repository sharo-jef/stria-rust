// Runtime values in Stria
use crate::parser::ast::*;
use crate::error::{StriaResult, StriaError};
use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64, IntegerType),
    Float(f64, FloatType),
    String(String),
    Boolean(bool),
    Null,
    Array(Vec<Value>),
    Struct(String, IndexMap<String, Value>),
    Function(String),
}

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Value::Integer(_, int_type) => Type::Integer(int_type.clone()),
            Value::Float(_, float_type) => Type::Float(float_type.clone()),
            Value::String(_) => Type::String,
            Value::Boolean(_) => Type::Boolean,
            Value::Null => Type::Null,
            Value::Array(elements) => {
                if let Some(first) = elements.first() {
                    Type::Array(Box::new(first.get_type()))
                } else {
                    Type::Array(Box::new(Type::Null))
                }
            }
            Value::Struct(name, _) => Type::Struct(name.clone()),
            Value::Function(_) => Type::Function,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(0, _) => false,
            Value::Float(f, _) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            _ => true,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Integer(i, _) => i.to_string(),
            Value::Float(f, _) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Struct(name, fields) => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{} {{ {} }}", name, field_strs.join(", "))
            }
            Value::Function(name) => format!("<function {}>", name),
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => a == b,
            (Value::Float(a, _), Value::Float(b, _)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equals(y))
            }
            (Value::Struct(name_a, fields_a), Value::Struct(name_b, fields_b)) => {
                name_a == name_b && 
                fields_a.len() == fields_b.len() &&
                fields_a.iter().all(|(k, v)| {
                    fields_b.get(k).map_or(false, |other_v| v.equals(other_v))
                })
            }
            // Allow comparison between integers and floats
            (Value::Integer(a, _), Value::Float(b, _)) => *a as f64 == *b,
            (Value::Float(a, _), Value::Integer(b, _)) => *a == *b as f64,
            _ => false,
        }
    }

    pub fn compare(&self, other: &Value) -> StriaResult<std::cmp::Ordering> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => Ok(a.cmp(b)),
            (Value::Float(a, _), Value::Float(b, _)) => Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b)),
            (Value::Integer(a, _), Value::Float(b, _)) => Ok((*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
            (Value::Float(a, _), Value::Integer(b, _)) => Ok(a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal)),
            _ => Err(StriaError::runtime(format!(
                "Cannot compare {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn get_type_name(&self) -> &str {
        match self {
            Value::Integer(_, _) => "integer",
            Value::Float(_, _) => "float",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Null => "null",
            Value::Array(_) => "array",
            Value::Struct(_, _) => "struct",
            Value::Function(_) => "function",
        }
    }

    pub fn add(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                Ok(Value::Integer(a + b, IntegerType::I64))
            }
            (Value::Float(a, _), Value::Float(b, _)) => {
                Ok(Value::Float(a + b, FloatType::F64))
            }
            (Value::Integer(a, _), Value::Float(b, _)) => {
                Ok(Value::Float(*a as f64 + b, FloatType::F64))
            }
            (Value::Float(a, _), Value::Integer(b, _)) => {
                Ok(Value::Float(a + *b as f64, FloatType::F64))
            }
            (Value::String(a), Value::String(b)) => {
                Ok(Value::String(format!("{}{}", a, b)))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot add {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn subtract(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                Ok(Value::Integer(a - b, IntegerType::I64))
            }
            (Value::Float(a, _), Value::Float(b, _)) => {
                Ok(Value::Float(a - b, FloatType::F64))
            }
            (Value::Integer(a, _), Value::Float(b, _)) => {
                Ok(Value::Float(*a as f64 - b, FloatType::F64))
            }
            (Value::Float(a, _), Value::Integer(b, _)) => {
                Ok(Value::Float(a - *b as f64, FloatType::F64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot subtract {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn multiply(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                Ok(Value::Integer(a * b, IntegerType::I64))
            }
            (Value::Float(a, _), Value::Float(b, _)) => {
                Ok(Value::Float(a * b, FloatType::F64))
            }
            (Value::Integer(a, _), Value::Float(b, _)) => {
                Ok(Value::Float(*a as f64 * b, FloatType::F64))
            }
            (Value::Float(a, _), Value::Integer(b, _)) => {
                Ok(Value::Float(a * *b as f64, FloatType::F64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot multiply {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn divide(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                if *b == 0 {
                    return Err(StriaError::runtime("Division by zero".to_string()));
                }
                Ok(Value::Integer(a / b, IntegerType::I64))
            }
            (Value::Float(a, _), Value::Float(b, _)) => {
                if *b == 0.0 {
                    return Err(StriaError::runtime("Division by zero".to_string()));
                }
                Ok(Value::Float(a / b, FloatType::F64))
            }
            (Value::Integer(a, _), Value::Float(b, _)) => {
                if *b == 0.0 {
                    return Err(StriaError::runtime("Division by zero".to_string()));
                }
                Ok(Value::Float(*a as f64 / b, FloatType::F64))
            }
            (Value::Float(a, _), Value::Integer(b, _)) => {
                if *b == 0 {
                    return Err(StriaError::runtime("Division by zero".to_string()));
                }
                Ok(Value::Float(a / *b as f64, FloatType::F64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot divide {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn modulo(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                if *b == 0 {
                    return Err(StriaError::runtime("Division by zero".to_string()));
                }
                Ok(Value::Integer(a % b, IntegerType::I64))
            }
            (Value::Float(a, _), Value::Float(b, _)) => {
                if *b == 0.0 {
                    return Err(StriaError::runtime("Division by zero".to_string()));
                }
                Ok(Value::Float(a % b, FloatType::F64))
            }
            (Value::Integer(a, _), Value::Float(b, _)) => {
                if *b == 0.0 {
                    return Err(StriaError::runtime("Division by zero".to_string()));
                }
                Ok(Value::Float((*a as f64) % b, FloatType::F64))
            }
            (Value::Float(a, _), Value::Integer(b, _)) => {
                if *b == 0 {
                    return Err(StriaError::runtime("Division by zero".to_string()));
                }
                Ok(Value::Float(a % (*b as f64), FloatType::F64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot modulo {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn negate(&self) -> StriaResult<Value> {
        match self {
            Value::Integer(i, int_type) => Ok(Value::Integer(-i, int_type.clone())),
            Value::Float(f, float_type) => Ok(Value::Float(-f, float_type.clone())),
            _ => Err(StriaError::runtime(format!(
                "Cannot negate {}",
                self.get_type_name()
            ))),
        }
    }

    pub fn logical_not(&self) -> Value {
        Value::Boolean(!self.is_truthy())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
