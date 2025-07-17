use std::env;
use std::fs;
use std::process;

mod error;
mod interpreter;
mod lexer;
mod output;
mod parser;
mod semantic;

use error::StriaError;
use interpreter::executor::Executor;
use lexer::tokenizer::Tokenizer;
use parser::parser::Parser;
use semantic::analyzer::SemanticAnalyzer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.stria>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file {}: {}", filename, err);
            process::exit(1);
        }
    };

    if let Err(err) = run_stria(&source) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run_stria(source: &str) -> Result<(), StriaError> {
    // Tokenize
    let mut tokenizer = Tokenizer::new();
    let tokens = tokenizer.tokenize(source)?;

    // Parse
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast)?;

    // Execute
    let mut executor = Executor::new();
    executor.execute(&ast)?;

    Ok(())
}
