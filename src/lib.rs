pub mod config;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod output;
pub mod parser;
pub mod semantic;

pub use config::CompileOptions;
pub use error::{StriaError, StriaResult};
pub use output::OutputFormat;

use std::path::Path;

/// Compile a Stria source string with the given options
pub fn compile(source: &str, options: CompileOptions) -> StriaResult<String> {
    use crate::interpreter::executor::Executor;
    use crate::lexer::tokenizer::Tokenizer;
    use crate::output::OutputGenerator;
    use crate::parser::Parser;
    use crate::semantic::analyzer::SemanticAnalyzer;
    use serde_json::Value as JsonValue;

    // Tokenize
    let mut tokenizer = Tokenizer::new();
    let tokens = tokenizer.tokenize(source)?;

    // Parse
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast, None)?;

    // Execute
    let mut executor = Executor::new();
    println!("Debug: About to transfer structs from analyzer to executor");
    println!("Debug: Analyzer has {} structs", analyzer.get_structs().len());
    println!("Debug: Analyzer struct keys: {:?}", analyzer.get_structs().keys().collect::<Vec<_>>());
    executor.set_structs(analyzer.get_structs().clone());
    executor.execute(&ast)?;

    // Generate output
    let output = JsonValue::Object(serde_json::Map::new());

    if options.validate_only {
        Ok("Valid".to_string())
    } else {
        OutputGenerator::generate(&output, options.format)
    }
}

/// Compile a Stria file with the given options
pub fn compile_file<P: AsRef<Path>>(path: P, options: CompileOptions) -> StriaResult<String> {
    let source =
        std::fs::read_to_string(path.as_ref()).map_err(|e| StriaError::IoError(e.to_string()))?;
    compile(&source, options)
}

/// Validate a Stria source string without generating output
pub fn validate(source: &str) -> StriaResult<()> {
    let options = CompileOptions {
        format: OutputFormat::Json,
        validate_only: true,
    };
    compile(source, options)?;
    Ok(())
}

/// Validate a Stria file without generating output
pub fn validate_file<P: AsRef<Path>>(path: P) -> StriaResult<()> {
    let source =
        std::fs::read_to_string(path.as_ref()).map_err(|e| StriaError::IoError(e.to_string()))?;
    validate(&source)
}
