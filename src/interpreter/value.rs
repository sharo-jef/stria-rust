// Runtime values in Stria
use crate::error::{StriaError, StriaResult};
use crate::parser::ast::*;
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64, IntegerType),
    Float(f64, FloatType),
    String(String),
    Boolean(bool),
    Null,
    List(Vec<Value>),
    I32Range {
        from: i32,
        to: i32,
        step: i32,
    },
    F64Range {
        from: f64,
        to: f64,
        step: f64,
    },
    Struct(String, IndexMap<String, Value>),
    Function(String),
    Optional(Option<Box<Value>>),
    #[allow(dead_code)]
    Union(Box<Value>, Vec<Type>), // Value with possible types
}

impl Value {
    #[allow(dead_code)]
    pub fn get_type(&self) -> Type {
        match self {
            Value::Integer(_, int_type) => Type::Integer(int_type.clone()),
            Value::Float(_, float_type) => Type::Float(float_type.clone()),
            Value::String(_) => Type::String,
            Value::Boolean(_) => Type::Boolean,
            Value::Null => Type::Null,
            Value::List(elements) => {
                if let Some(first) = elements.first() {
                    Type::List(Box::new(first.get_type()))
                } else {
                    Type::List(Box::new(Type::Null))
                }
            }
            Value::I32Range { .. } => Type::I32Range,
            Value::F64Range { .. } => Type::F64Range,
            Value::Struct(name, _) => Type::Struct(name.clone()),
            Value::Function(_) => Type::Function,
            Value::Optional(opt) => {
                if let Some(val) = opt {
                    Type::Optional(Box::new(val.get_type()))
                } else {
                    Type::Optional(Box::new(Type::Null))
                }
            }
            Value::Union(val, _) => val.get_type(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(0, _) => false,
            Value::Float(f, _) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(arr) => !arr.is_empty(),
            Value::I32Range { from, to, step } => {
                if *step > 0 {
                    from < to
                } else {
                    from > to
                }
            }
            Value::F64Range { from, to, step } => {
                if *step > 0.0 {
                    from < to
                } else {
                    from > to
                }
            }
            Value::Optional(opt) => opt.is_some(),
            Value::Union(val, _) => val.is_truthy(),
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
            Value::List(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::I32Range { from, to, step } => {
                if *step == 1 {
                    format!("{}..{}", from, to)
                } else {
                    format!("{}..{} step {}", from, to, step)
                }
            }
            Value::F64Range { from, to, step } => {
                if *step == 1.0 {
                    format!("{}..{}", from, to)
                } else {
                    format!("{}..{} step {}", from, to, step)
                }
            }
            Value::Struct(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{} {{ {} }}", name, field_strs.join(", "))
            }
            Value::Function(name) => format!("<function {}>", name),
            Value::Optional(opt) => {
                if let Some(val) = opt {
                    val.to_string()
                } else {
                    "null".to_string()
                }
            }
            Value::Union(val, _) => val.to_string(),
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => a == b,
            (Value::Float(a, _), Value::Float(b, _)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equals(y))
            }
            (
                Value::I32Range {
                    from: a_from,
                    to: a_to,
                    step: a_step,
                },
                Value::I32Range {
                    from: b_from,
                    to: b_to,
                    step: b_step,
                },
            ) => a_from == b_from && a_to == b_to && a_step == b_step,
            (
                Value::F64Range {
                    from: a_from,
                    to: a_to,
                    step: a_step,
                },
                Value::F64Range {
                    from: b_from,
                    to: b_to,
                    step: b_step,
                },
            ) => a_from == b_from && a_to == b_to && a_step == b_step,
            (Value::Struct(name_a, fields_a), Value::Struct(name_b, fields_b)) => {
                name_a == name_b
                    && fields_a.len() == fields_b.len()
                    && fields_a
                        .iter()
                        .all(|(k, v)| fields_b.get(k).map_or(false, |other_v| v.equals(other_v)))
            }
            (Value::Optional(a), Value::Optional(b)) => match (a, b) {
                (Some(a_val), Some(b_val)) => a_val.equals(b_val),
                (None, None) => true,
                _ => false,
            },
            (Value::Union(a, _), Value::Union(b, _)) => a.equals(b),
            // Allow comparison between integers and floats
            (Value::Integer(a, _), Value::Float(b, _)) => *a as f64 == *b,
            (Value::Float(a, _), Value::Integer(b, _)) => *a == *b as f64,
            // Optional comparisons
            (Value::Optional(Some(a)), b) => a.equals(b),
            (a, Value::Optional(Some(b))) => a.equals(b),
            (Value::Optional(None), Value::Null) => true,
            (Value::Null, Value::Optional(None)) => true,
            _ => false,
        }
    }

    pub fn compare(&self, other: &Value) -> StriaResult<std::cmp::Ordering> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => Ok(a.cmp(b)),
            (Value::Float(a, _), Value::Float(b, _)) => {
                Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            }
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b)),
            (Value::Integer(a, _), Value::Float(b, _)) => Ok((*a as f64)
                .partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Equal)),
            (Value::Float(a, _), Value::Integer(b, _)) => Ok(a
                .partial_cmp(&(*b as f64))
                .unwrap_or(std::cmp::Ordering::Equal)),
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
            Value::List(_) => "list",
            Value::I32Range { .. } => "i32_range",
            Value::F64Range { .. } => "f64_range",
            Value::Struct(_, _) => "struct",
            Value::Function(_) => "function",
            Value::Optional(_) => "optional",
            Value::Union(_, _) => "union",
        }
    }

    pub fn add(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                Ok(Value::Integer(a + b, IntegerType::I64))
            }
            (Value::Float(a, _), Value::Float(b, _)) => Ok(Value::Float(a + b, FloatType::F64)),
            (Value::Integer(a, _), Value::Float(b, _)) => {
                Ok(Value::Float(*a as f64 + b, FloatType::F64))
            }
            (Value::Float(a, _), Value::Integer(b, _)) => {
                Ok(Value::Float(a + *b as f64, FloatType::F64))
            }
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
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
            (Value::Float(a, _), Value::Float(b, _)) => Ok(Value::Float(a - b, FloatType::F64)),
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
            (Value::Float(a, _), Value::Float(b, _)) => Ok(Value::Float(a * b, FloatType::F64)),
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

    // Range operations
    pub fn step(&self, step_val: &Value) -> StriaResult<Value> {
        match (self, step_val) {
            (Value::I32Range { from, to, step: _ }, Value::Integer(new_step, _)) => {
                let new_step = *new_step as i32;
                if new_step == 0 {
                    return Err(StriaError::runtime("step must not be zero".to_string()));
                }
                let final_step = if (new_step < 0 && from < to) || (new_step > 0 && from > to) {
                    -new_step
                } else {
                    new_step
                };
                Ok(Value::I32Range {
                    from: *from,
                    to: *to,
                    step: final_step,
                })
            }
            (Value::F64Range { from, to, step: _ }, Value::Float(new_step, _)) => {
                if *new_step == 0.0 {
                    return Err(StriaError::runtime("step must not be zero".to_string()));
                }
                let final_step = if (*new_step < 0.0 && from < to) || (*new_step > 0.0 && from > to)
                {
                    -*new_step
                } else {
                    *new_step
                };
                Ok(Value::F64Range {
                    from: *from,
                    to: *to,
                    step: final_step,
                })
            }
            (Value::F64Range { from, to, step: _ }, Value::Integer(new_step, _)) => {
                let new_step = *new_step as f64;
                if new_step == 0.0 {
                    return Err(StriaError::runtime("step must not be zero".to_string()));
                }
                let final_step = if (new_step < 0.0 && from < to) || (new_step > 0.0 && from > to) {
                    -new_step
                } else {
                    new_step
                };
                Ok(Value::F64Range {
                    from: *from,
                    to: *to,
                    step: final_step,
                })
            }
            _ => Err(StriaError::runtime(
                "step can only be applied to ranges".to_string(),
            )),
        }
    }

    pub fn contains(&self, value: &Value) -> StriaResult<bool> {
        match (self, value) {
            (Value::I32Range { from, to, step }, Value::Integer(val, _)) => {
                let val = *val as i32;
                Ok(if *step > 0 {
                    val >= *from && val <= *to && ((val - from) % step == 0)
                } else {
                    val <= *from && val >= *to && ((from - val) % (-step) == 0)
                })
            }
            (Value::F64Range { from, to, step }, Value::Float(val, _)) => Ok(if *step > 0.0 {
                *val >= *from && *val <= *to && ((*val - from) % step == 0.0)
            } else {
                *val <= *from && *val >= *to && ((*from - val) % (-step) == 0.0)
            }),
            (Value::F64Range { from, to, step }, Value::Integer(val, _)) => {
                let val = *val as f64;
                Ok(if *step > 0.0 {
                    val >= *from && val <= *to && ((val - from) % step == 0.0)
                } else {
                    val <= *from && val >= *to && ((*from - val) % (-step) == 0.0)
                })
            }
            (Value::List(elements), value) => Ok(elements.iter().any(|elem| elem.equals(value))),
            (Value::String(s), Value::String(substr)) => Ok(s.contains(substr)),
            _ => Err(StriaError::runtime(
                "in operator not supported for these types".to_string(),
            )),
        }
    }

    pub fn until(&self, to: &Value) -> StriaResult<Value> {
        match (self, to) {
            (Value::Integer(from, _), Value::Integer(to_val, _)) => {
                let from = *from as i32;
                let to_val = *to_val as i32;
                if to_val == from {
                    return Err(StriaError::runtime(
                        "to must not be equal to from".to_string(),
                    ));
                }
                if to_val < from {
                    return Err(StriaError::runtime(
                        "to must be greater than from".to_string(),
                    ));
                }
                Ok(Value::I32Range {
                    from,
                    to: to_val - 1,
                    step: 1,
                })
            }
            (Value::Float(from, _), Value::Float(to_val, _)) => {
                if *to_val == *from {
                    return Err(StriaError::runtime(
                        "to must not be equal to from".to_string(),
                    ));
                }
                if *to_val < *from {
                    return Err(StriaError::runtime(
                        "to must be greater than from".to_string(),
                    ));
                }
                Ok(Value::F64Range {
                    from: *from,
                    to: *to_val,
                    step: 1.0,
                })
            }
            _ => Err(StriaError::runtime(
                "until can only be applied to integers and floats".to_string(),
            )),
        }
    }

    pub fn down_to(&self, to: &Value) -> StriaResult<Value> {
        match (self, to) {
            (Value::Integer(from, _), Value::Integer(to_val, _)) => {
                let from = *from as i32;
                let to_val = *to_val as i32;
                if to_val == from {
                    return Err(StriaError::runtime(
                        "to must not be equal to from".to_string(),
                    ));
                }
                if to_val > from {
                    return Err(StriaError::runtime("to must be less than from".to_string()));
                }
                Ok(Value::I32Range {
                    from,
                    to: to_val,
                    step: -1,
                })
            }
            (Value::Float(from, _), Value::Float(to_val, _)) => {
                if *to_val == *from {
                    return Err(StriaError::runtime(
                        "to must not be equal to from".to_string(),
                    ));
                }
                if *to_val > *from {
                    return Err(StriaError::runtime("to must be less than from".to_string()));
                }
                Ok(Value::F64Range {
                    from: *from,
                    to: *to_val,
                    step: -1.0,
                })
            }
            _ => Err(StriaError::runtime(
                "downTo can only be applied to integers and floats".to_string(),
            )),
        }
    }

    // List operations
    #[allow(dead_code)]
    pub fn push(&mut self, value: Value) -> StriaResult<()> {
        match self {
            Value::List(elements) => {
                elements.push(value);
                Ok(())
            }
            _ => Err(StriaError::runtime(
                "push can only be applied to lists".to_string(),
            )),
        }
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn pop(&mut self) -> StriaResult<Value> {
        match self {
            Value::List(elements) => elements
                .pop()
                .ok_or_else(|| StriaError::runtime("cannot pop from empty list".to_string())),
            _ => Err(StriaError::runtime(
                "pop can only be applied to lists".to_string(),
            )),
        }
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, index: usize, value: Value) -> StriaResult<()> {
        match self {
            Value::List(elements) => {
                if index <= elements.len() {
                    elements.insert(index, value);
                    Ok(())
                } else {
                    Err(StriaError::runtime(format!(
                        "insert index {} out of bounds",
                        index
                    )))
                }
            }
            _ => Err(StriaError::runtime(
                "insert can only be applied to lists".to_string(),
            )),
        }
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, index: usize) -> StriaResult<Value> {
        match self {
            Value::List(elements) => {
                if index < elements.len() {
                    Ok(elements.remove(index))
                } else {
                    Err(StriaError::runtime(format!(
                        "remove index {} out of bounds",
                        index
                    )))
                }
            }
            _ => Err(StriaError::runtime(
                "remove can only be applied to lists".to_string(),
            )),
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> StriaResult<Value> {
        match self {
            Value::List(elements) => Ok(Value::Integer(elements.len() as i64, IntegerType::I32)),
            Value::String(s) => Ok(Value::Integer(s.len() as i64, IntegerType::I32)),
            _ => Err(StriaError::runtime(
                "len can only be applied to lists and strings".to_string(),
            )),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> StriaResult<bool> {
        match self {
            Value::List(elements) => Ok(elements.is_empty()),
            Value::String(s) => Ok(s.is_empty()),
            _ => Err(StriaError::runtime(
                "isEmpty can only be applied to lists and strings".to_string(),
            )),
        }
    }

    // Type checking
    pub fn is_type(&self, type_name: &str) -> bool {
        match type_name {
            "string" => matches!(self, Value::String(_)),
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" => {
                matches!(self, Value::Integer(_, _))
            }
            "f32" | "f64" => matches!(self, Value::Float(_, _)),
            "bool" => matches!(self, Value::Boolean(_)),
            "null" => matches!(self, Value::Null),
            "list" => matches!(self, Value::List(_)),
            "i32_range" => matches!(self, Value::I32Range { .. }),
            "f64_range" => matches!(self, Value::F64Range { .. }),
            _ => false,
        }
    }

    // Type casting
    pub fn cast_to(&self, target_type: &str) -> StriaResult<Value> {
        match target_type {
            "string" => Ok(Value::String(self.to_string())),
            "i8" | "i16" | "i32" | "i64" => match self {
                Value::Integer(i, _) => Ok(Value::Integer(*i, IntegerType::I32)),
                Value::Float(f, _) => Ok(Value::Integer(*f as i64, IntegerType::I32)),
                Value::String(s) => match s.parse::<i64>() {
                    Ok(i) => Ok(Value::Integer(i, IntegerType::I32)),
                    Err(_) => Err(StriaError::runtime(format!(
                        "Cannot cast '{}' to integer",
                        s
                    ))),
                },
                Value::Boolean(b) => Ok(Value::Integer(if *b { 1 } else { 0 }, IntegerType::I32)),
                _ => Err(StriaError::runtime(format!(
                    "Cannot cast {} to integer",
                    self.get_type_name()
                ))),
            },
            "f32" | "f64" => match self {
                Value::Integer(i, _) => Ok(Value::Float(*i as f64, FloatType::F64)),
                Value::Float(f, _) => Ok(Value::Float(*f, FloatType::F64)),
                Value::String(s) => match s.parse::<f64>() {
                    Ok(f) => Ok(Value::Float(f, FloatType::F64)),
                    Err(_) => Err(StriaError::runtime(format!("Cannot cast '{}' to float", s))),
                },
                _ => Err(StriaError::runtime(format!(
                    "Cannot cast {} to float",
                    self.get_type_name()
                ))),
            },
            "bool" => match self {
                Value::Boolean(b) => Ok(Value::Boolean(*b)),
                Value::Integer(i, _) => Ok(Value::Boolean(*i != 0)),
                Value::Float(f, _) => Ok(Value::Boolean(*f != 0.0)),
                Value::String(s) => match s.to_lowercase().as_str() {
                    "true" => Ok(Value::Boolean(true)),
                    "false" => Ok(Value::Boolean(false)),
                    _ => Err(StriaError::runtime(format!(
                        "Cannot cast '{}' to boolean",
                        s
                    ))),
                },
                _ => Err(StriaError::runtime(format!(
                    "Cannot cast {} to boolean",
                    self.get_type_name()
                ))),
            },
            _ => Err(StriaError::runtime(format!(
                "Unknown target type: {}",
                target_type
            ))),
        }
    }

    // Power operation
    pub fn power(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(base, _), Value::Integer(exp, _)) => {
                if *exp < 0 {
                    // For negative exponents, return a float
                    let result = (*base as f64).powf(*exp as f64);
                    Ok(Value::Float(result, FloatType::F64))
                } else {
                    let result = (*base as i64).pow(*exp as u32);
                    Ok(Value::Integer(result, IntegerType::I64))
                }
            }
            (Value::Float(base, _), Value::Float(exp, _)) => {
                Ok(Value::Float(base.powf(*exp), FloatType::F64))
            }
            (Value::Integer(base, _), Value::Float(exp, _)) => {
                let result = (*base as f64).powf(*exp);
                Ok(Value::Float(result, FloatType::F64))
            }
            (Value::Float(base, _), Value::Integer(exp, _)) => {
                let result = base.powf(*exp as f64);
                Ok(Value::Float(result, FloatType::F64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot raise {} to the power of {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    // Bitwise operations
    pub fn bitwise_and(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                Ok(Value::Integer(a & b, IntegerType::I64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot perform bitwise AND on {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn bitwise_or(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                Ok(Value::Integer(a | b, IntegerType::I64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot perform bitwise OR on {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn bitwise_xor(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                Ok(Value::Integer(a ^ b, IntegerType::I64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot perform bitwise XOR on {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn bitwise_not(&self) -> StriaResult<Value> {
        match self {
            Value::Integer(i, int_type) => Ok(Value::Integer(!i, int_type.clone())),
            _ => Err(StriaError::runtime(format!(
                "Cannot perform bitwise NOT on {}",
                self.get_type_name()
            ))),
        }
    }

    pub fn left_shift(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                Ok(Value::Integer(a << b, IntegerType::I64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot perform left shift on {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }

    pub fn right_shift(&self, other: &Value) -> StriaResult<Value> {
        match (self, other) {
            (Value::Integer(a, _), Value::Integer(b, _)) => {
                Ok(Value::Integer(a >> b, IntegerType::I64))
            }
            _ => Err(StriaError::runtime(format!(
                "Cannot perform right shift on {} and {}",
                self.get_type_name(),
                other.get_type_name()
            ))),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
