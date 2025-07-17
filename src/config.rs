use crate::output::OutputFormat;

/// Compilation options for Stria
#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub format: OutputFormat,
    pub validate_only: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Json,
            validate_only: false,
        }
    }
}
