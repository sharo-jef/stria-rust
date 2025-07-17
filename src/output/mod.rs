use crate::error::StriaResult;
use serde_json::Value as JsonValue;

/// Output format for compiled Stria configuration
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Yaml,
    Toml,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<OutputFormat> {
        match s.to_lowercase().as_str() {
            "json" => Some(OutputFormat::Json),
            "yaml" | "yml" => Some(OutputFormat::Yaml),
            "toml" => Some(OutputFormat::Toml),
            _ => None,
        }
    }
}

/// Configuration output generator
pub struct OutputGenerator;

impl OutputGenerator {
    /// Generate output in the specified format
    pub fn generate(data: &JsonValue, format: OutputFormat) -> StriaResult<String> {
        match format {
            OutputFormat::Json => Self::generate_json(data),
            OutputFormat::Yaml => Self::generate_yaml(data),
            OutputFormat::Toml => Self::generate_toml(data),
        }
    }

    fn generate_json(data: &JsonValue) -> StriaResult<String> {
        serde_json::to_string_pretty(data).map_err(|e| {
            crate::error::StriaError::RuntimeError(format!("JSON serialization failed: {}", e))
        })
    }

    fn generate_yaml(data: &JsonValue) -> StriaResult<String> {
        serde_yaml::to_string(data).map_err(|e| {
            crate::error::StriaError::RuntimeError(format!("YAML serialization failed: {}", e))
        })
    }

    fn generate_toml(data: &JsonValue) -> StriaResult<String> {
        // Convert JsonValue to toml::Value
        let toml_value = json_to_toml(data)?;
        toml::to_string(&toml_value).map_err(|e| {
            crate::error::StriaError::RuntimeError(format!("TOML serialization failed: {}", e))
        })
    }
}

fn json_to_toml(value: &JsonValue) -> StriaResult<toml::Value> {
    match value {
        JsonValue::Null => Ok(toml::Value::String("null".to_string())),
        JsonValue::Bool(b) => Ok(toml::Value::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Ok(toml::Value::String(n.to_string()))
            }
        }
        JsonValue::String(s) => Ok(toml::Value::String(s.clone())),
        JsonValue::Array(arr) => {
            let mut toml_arr = Vec::new();
            for item in arr {
                toml_arr.push(json_to_toml(item)?);
            }
            Ok(toml::Value::Array(toml_arr))
        }
        JsonValue::Object(obj) => {
            let mut toml_table = toml::value::Table::new();
            for (key, value) in obj {
                toml_table.insert(key.clone(), json_to_toml(value)?);
            }
            Ok(toml::Value::Table(toml_table))
        }
    }
}
