use crate::interpreter::value::Value;
use crate::error::{StriaResult, StriaError};

pub fn get_builtin_functions() -> Vec<(&'static str, fn(&[Value]) -> StriaResult<Value>)> {
    vec![
        ("print", builtin_print),
        ("println", builtin_println),
        ("len", builtin_len),
        ("str", builtin_str),
        ("int", builtin_int),
        ("float", builtin_float),
    ]
}

fn builtin_print(args: &[Value]) -> StriaResult<Value> {
    if args.is_empty() {
        return Err(StriaError::runtime("print requires at least one argument".to_string()));
    }
    
    let output = args.iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    
    print!("{}", output);
    Ok(Value::Null)
}

fn builtin_println(args: &[Value]) -> StriaResult<Value> {
    if args.is_empty() {
        println!();
    } else {
        let output = args.iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        
        println!("{}", output);
    }
    
    Ok(Value::Null)
}

fn builtin_len(args: &[Value]) -> StriaResult<Value> {
    if args.len() != 1 {
        return Err(StriaError::runtime("len requires exactly one argument".to_string()));
    }
    
    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64, crate::parser::ast::IntegerType::I64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64, crate::parser::ast::IntegerType::I64)),
        _ => Err(StriaError::runtime("len can only be applied to strings and arrays".to_string())),
    }
}

fn builtin_str(args: &[Value]) -> StriaResult<Value> {
    if args.len() != 1 {
        return Err(StriaError::runtime("str requires exactly one argument".to_string()));
    }
    
    Ok(Value::String(args[0].to_string()))
}

fn builtin_int(args: &[Value]) -> StriaResult<Value> {
    if args.len() != 1 {
        return Err(StriaError::runtime("int requires exactly one argument".to_string()));
    }
    
    match &args[0] {
        Value::Integer(i, _) => Ok(Value::Integer(*i, crate::parser::ast::IntegerType::I64)),
        Value::Float(f, _) => Ok(Value::Integer(*f as i64, crate::parser::ast::IntegerType::I64)),
        Value::String(s) => {
            match s.parse::<i64>() {
                Ok(i) => Ok(Value::Integer(i, crate::parser::ast::IntegerType::I64)),
                Err(_) => Err(StriaError::runtime(format!("Cannot convert '{}' to integer", s))),
            }
        }
        Value::Boolean(b) => Ok(Value::Integer(if *b { 1 } else { 0 }, crate::parser::ast::IntegerType::I64)),
        _ => Err(StriaError::runtime("Cannot convert value to integer".to_string())),
    }
}

fn builtin_float(args: &[Value]) -> StriaResult<Value> {
    if args.len() != 1 {
        return Err(StriaError::runtime("float requires exactly one argument".to_string()));
    }
    
    match &args[0] {
        Value::Integer(i, _) => Ok(Value::Float(*i as f64, crate::parser::ast::FloatType::F64)),
        Value::Float(f, _) => Ok(Value::Float(*f, crate::parser::ast::FloatType::F64)),
        Value::String(s) => {
            match s.parse::<f64>() {
                Ok(f) => Ok(Value::Float(f, crate::parser::ast::FloatType::F64)),
                Err(_) => Err(StriaError::runtime(format!("Cannot convert '{}' to float", s))),
            }
        }
        _ => Err(StriaError::runtime("Cannot convert value to float".to_string())),
    }
}
// TODO: Implement complete standard library
