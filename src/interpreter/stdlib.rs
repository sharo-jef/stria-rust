use crate::error::{StriaError, StriaResult};
use crate::interpreter::value::Value;
use crate::parser::ast::*;
use std::env;

pub fn get_builtin_functions() -> Vec<(&'static str, fn(&[Value]) -> StriaResult<Value>)> {
    vec![
        ("print", builtin_print),
        ("println", builtin_println),
        ("len", builtin_len),
        ("str", builtin_str),
        ("int", builtin_int),
        ("float", builtin_float),
        ("getEnv", builtin_get_env),
        ("random", builtin_random),
    ]
}

fn builtin_print(args: &[Value]) -> StriaResult<Value> {
    if args.is_empty() {
        return Err(StriaError::runtime(
            "print requires at least one argument".to_string(),
        ));
    }

    let output = args
        .iter()
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
        let output = args
            .iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        println!("{}", output);
    }

    Ok(Value::Null)
}

fn builtin_len(args: &[Value]) -> StriaResult<Value> {
    if args.len() != 1 {
        return Err(StriaError::runtime(
            "len requires exactly one argument".to_string(),
        ));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64, IntegerType::I32)),
        Value::List(arr) => Ok(Value::Integer(arr.len() as i64, IntegerType::I32)),
        _ => Err(StriaError::runtime(
            "len can only be applied to strings and lists".to_string(),
        )),
    }
}

fn builtin_str(args: &[Value]) -> StriaResult<Value> {
    if args.len() != 1 {
        return Err(StriaError::runtime(
            "str requires exactly one argument".to_string(),
        ));
    }

    Ok(Value::String(args[0].to_string()))
}

fn builtin_int(args: &[Value]) -> StriaResult<Value> {
    if args.len() != 1 {
        return Err(StriaError::runtime(
            "int requires exactly one argument".to_string(),
        ));
    }

    match &args[0] {
        Value::Integer(i, _) => Ok(Value::Integer(*i, crate::parser::ast::IntegerType::I64)),
        Value::Float(f, _) => Ok(Value::Integer(
            *f as i64,
            crate::parser::ast::IntegerType::I64,
        )),
        Value::String(s) => match s.parse::<i64>() {
            Ok(i) => Ok(Value::Integer(i, crate::parser::ast::IntegerType::I64)),
            Err(_) => Err(StriaError::runtime(format!(
                "Cannot convert '{}' to integer",
                s
            ))),
        },
        Value::Boolean(b) => Ok(Value::Integer(
            if *b { 1 } else { 0 },
            crate::parser::ast::IntegerType::I64,
        )),
        _ => Err(StriaError::runtime(
            "Cannot convert value to integer".to_string(),
        )),
    }
}

fn builtin_float(args: &[Value]) -> StriaResult<Value> {
    if args.len() != 1 {
        return Err(StriaError::runtime(
            "float requires exactly one argument".to_string(),
        ));
    }

    match &args[0] {
        Value::Integer(i, _) => Ok(Value::Float(*i as f64, crate::parser::ast::FloatType::F64)),
        Value::Float(f, _) => Ok(Value::Float(*f, crate::parser::ast::FloatType::F64)),
        Value::String(s) => match s.parse::<f64>() {
            Ok(f) => Ok(Value::Float(f, crate::parser::ast::FloatType::F64)),
            Err(_) => Err(StriaError::runtime(format!(
                "Cannot convert '{}' to float",
                s
            ))),
        },
        _ => Err(StriaError::runtime(
            "Cannot convert value to float".to_string(),
        )),
    }
}

fn builtin_get_env(args: &[Value]) -> StriaResult<Value> {
    if args.len() != 1 {
        return Err(StriaError::runtime(
            "getEnv requires exactly one argument".to_string(),
        ));
    }

    match &args[0] {
        Value::String(key) => match env::var(key) {
            Ok(value) => Ok(Value::Optional(Some(Box::new(Value::String(value))))),
            Err(_) => Ok(Value::Optional(None)),
        },
        _ => Err(StriaError::runtime(
            "getEnv requires a string argument".to_string(),
        )),
    }
}

fn builtin_random(args: &[Value]) -> StriaResult<Value> {
    if !args.is_empty() {
        return Err(StriaError::runtime("random takes no arguments".to_string()));
    }

    // Simple random implementation using system time
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    // Simple linear congruential generator
    let a = 1664525u64;
    let c = 1013904223u64;
    let m = 2u64.pow(32);

    let next = (a.wrapping_mul(seed).wrapping_add(c)) % m;
    let normalized = next as f64 / m as f64;

    Ok(Value::Float(normalized, FloatType::F64))
}
// TODO: Implement complete standard library
