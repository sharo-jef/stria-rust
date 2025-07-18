pub mod diagnostic;

use thiserror::Error;

pub type StriaResult<T> = Result<T, StriaError>;

#[derive(Error, Debug)]
pub enum StriaError {
    #[error("Lexer error: {0}")]
    LexerError(String),

    #[error("Parser error: {0}")]
    ParserError(String),

    #[error("Semantic error: {0}")]
    SemanticError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("IO error: {0}")]
    #[allow(dead_code)]
    IoError(String),

    #[error("Type error: {0}")]
    #[allow(dead_code)]
    TypeError(String),

    #[error("Validation error: {0}")]
    #[allow(dead_code)]
    ValidationError(String),

    #[error("{0}")]
    Diagnostic(diagnostic::Diagnostic),
}

impl StriaError {
    pub fn lexer(msg: impl Into<String>) -> Self {
        StriaError::LexerError(msg.into())
    }

    pub fn parser(msg: impl Into<String>) -> Self {
        StriaError::ParserError(msg.into())
    }

    pub fn semantic(msg: impl Into<String>) -> Self {
        StriaError::SemanticError(msg.into())
    }

    pub fn runtime(msg: impl Into<String>) -> Self {
        StriaError::RuntimeError(msg.into())
    }

    #[allow(dead_code)]
    pub fn diagnostic(diagnostic: diagnostic::Diagnostic) -> Self {
        StriaError::Diagnostic(diagnostic)
    }

    pub fn parser_with_span(msg: impl Into<String>, span: crate::lexer::token::Span) -> Self {
        StriaError::Diagnostic(diagnostic::Diagnostic::error_with_span(msg, span))
    }

    pub fn parser_with_span_and_help(
        msg: impl Into<String>,
        span: crate::lexer::token::Span,
        help: impl Into<String>,
    ) -> Self {
        StriaError::Diagnostic(diagnostic::Diagnostic::error_with_span(msg, span).with_help(help))
    }

    #[allow(dead_code)]
    pub fn type_error(msg: impl Into<String>) -> Self {
        StriaError::TypeError(msg.into())
    }

    #[allow(dead_code)]
    pub fn validation(msg: impl Into<String>) -> Self {
        StriaError::ValidationError(msg.into())
    }

    pub fn format_rich(&self, source: &str, filename: &str) -> String {
        match self {
            StriaError::Diagnostic(diagnostic) => {
                let mut d = diagnostic.clone();
                d.filename = Some(filename.to_string());
                d.format_error(source)
            }
            _ => format!("{}", self),
        }
    }

    pub fn format_lsp(&self, source: &str, filename: &str) -> String {
        match self {
            StriaError::Diagnostic(diagnostic) => {
                let mut d = diagnostic.clone();
                d.filename = Some(filename.to_string());
                d.format_lsp(source)
            }
            _ => format!("{}", self),
        }
    }
}
