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

        // Colors for different severities
        let (severity_color, reset_color) = match self.severity {
            Severity::Error => ("\x1b[31m", "\x1b[0m"),   // Red
            Severity::Warning => ("\x1b[33m", "\x1b[0m"), // Yellow
            Severity::Info => ("\x1b[34m", "\x1b[0m"),    // Blue
        };

        // Calculate the width needed for the line number
        let line_num_width = self.line.to_string().len();
        let line_prefix_spaces = " ".repeat(line_num_width);

        let mut result = format!(
            "{severity_color}{severity_name}[{error_code}]{reset_color}: {message}\n  {arrow_color}-->{reset_color} {filename}:{line}:{column}\n{line_prefix_spaces} {pipe_color}|{reset_color}\n{line_number} {pipe_color}|{reset_color} {line_content}\n{line_prefix_spaces} {pipe_color}|{reset_color} {padding}{caret_color}{carets}{reset_color}",
            severity_color = severity_color,
            severity_name = severity_name,
            error_code = error_code,
            reset_color = reset_color,
            message = self.message,
            arrow_color = "\x1b[34m",  // Blue for arrow
            filename = filename,
            line = self.line,
            column = self.column + 1, // Convert to 1-based for display
            pipe_color = "\x1b[34m",  // Blue for pipe
            line_prefix_spaces = line_prefix_spaces,
            line_number = self.line,  // Left-aligned line number
            line_content = line_content,
            padding = " ".repeat(self.column),
            caret_color = "\x1b[31m",  // Red for carets
            carets = "^".repeat(self.length.max(1))
        );

        if let Some(help) = &self.help {
            result.push_str(&format!(
                "\n{line_prefix_spaces} \x1b[34m|\x1b[0m\n\x1b[32mhelp\x1b[0m: {help}",
                line_prefix_spaces = line_prefix_spaces,
                help = help
            ));
        }

        result
    }

    /// Format diagnostic in LSP-compatible format
    pub fn format_lsp(&self, _source: &str) -> String {
        let filename = self.filename.as_deref().unwrap_or("<stdin>");

        // LSP format: filename:line:column: severity: message
        format!(
            "{}:{}:{}: {}: {}",
            filename,
            self.line,
            self.column + 1, // Convert to 1-based for display
            match self.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
            },
            self.message
        )
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
