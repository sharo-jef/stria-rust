use crate::error::StriaError;

/// Diagnostic information for error reporting
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub severity: Severity,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, line: usize, column: usize, length: usize) -> Self {
        Self {
            message: message.into(),
            line,
            column,
            length,
            severity: Severity::Error,
        }
    }

    pub fn warning(message: impl Into<String>, line: usize, column: usize, length: usize) -> Self {
        Self {
            message: message.into(),
            line,
            column,
            length,
            severity: Severity::Warning,
        }
    }

    /// Format diagnostic in Rust-style error format
    pub fn format_error(&self, source: &str, filename: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let line_content = lines.get(self.line.saturating_sub(1)).unwrap_or(&"");

        let error_code = match self.severity {
            Severity::Error => "E001",
            Severity::Warning => "W001",
            Severity::Info => "I001",
        };

        let severity_name = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };

        format!(
            "{severity_name}[{error_code}]: {message}\n  --> {filename}:{line}:{column}\n   |\n{line:4} | {line_content}\n   | {padding}{carets}\n",
            severity_name = severity_name,
            error_code = error_code,
            message = self.message,
            filename = filename,
            line = self.line,
            column = self.column,
            line_content = line_content,
            padding = " ".repeat(self.column.saturating_sub(1)),
            carets = "^".repeat(self.length.max(1))
        )
    }
}

impl From<Diagnostic> for StriaError {
    fn from(diagnostic: Diagnostic) -> Self {
        match diagnostic.severity {
            Severity::Error => StriaError::SemanticError(diagnostic.message),
            Severity::Warning => StriaError::SemanticError(diagnostic.message),
            Severity::Info => StriaError::SemanticError(diagnostic.message),
        }
    }
}
