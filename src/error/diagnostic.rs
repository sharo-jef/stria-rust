use crate::error::StriaError;
use crate::lexer::token::Span;

/// Diagnostic information for error reporting
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Diagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub severity: Severity,
    pub span: Option<Span>,
    pub filename: Option<String>,
    pub help: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Diagnostic {
    #[allow(dead_code)]
    pub fn error(message: impl Into<String>, line: usize, column: usize, length: usize) -> Self {
        Self {
            message: message.into(),
            line,
            column,
            length,
            severity: Severity::Error,
            span: None,
            filename: None,
            help: None,
        }
    }

    pub fn error_with_span(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            line: span.line,
            column: span.column,
            length: span.end - span.start,
            severity: Severity::Error,
            span: Some(span),
            filename: None,
            help: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    #[allow(dead_code)]
    pub fn warning(message: impl Into<String>, line: usize, column: usize, length: usize) -> Self {
        Self {
            message: message.into(),
            line,
            column,
            length,
            severity: Severity::Warning,
            span: None,
            filename: None,
            help: None,
        }
    }

    /// Format diagnostic in Rust-style error format
    pub fn format_error(&self, source: &str) -> String {
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

        let filename = self.filename.as_deref().unwrap_or("<stdin>");

        let mut result = format!(
            "{severity_name}[{error_code}]: {message}\n  --> {filename}:{line}:{column}\n   |\n{line:4} | {line_content}\n   | {padding}{carets}",
            severity_name = severity_name,
            error_code = error_code,
            message = self.message,
            filename = filename,
            line = self.line,
            column = self.column + 1, // Convert to 1-based for display
            line_content = line_content,
            padding = " ".repeat(self.column),
            carets = "^".repeat(self.length.max(1))
        );

        if let Some(help) = &self.help {
            result.push_str(&format!("\n   |\nhelp: {}", help));
        }

        result
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
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
