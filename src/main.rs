use clap::Parser;
use std::fs;
use std::process;

mod error;
mod interpreter;
mod lexer;
mod output;
mod parser;
mod semantic;

use crate::output::{OutputFormat, OutputGenerator};
use error::StriaError;
use interpreter::executor::Executor;
use lexer::tokenizer::Tokenizer;
use parser::parser::Parser as StriaParser;
use semantic::analyzer::SemanticAnalyzer;

#[derive(Parser)]
#[command(name = "stria-rust")]
#[command(about = "A Stria language interpreter")]
struct Args {
    /// Input file
    input: String,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    format: OutputFormatArg,

    /// Enable LSP mode (output LSP-compatible diagnostics)
    #[arg(long)]
    lsp: bool,
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormatArg {
    Json,
    Yaml,
    Toml,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Json => OutputFormat::Json,
            OutputFormatArg::Yaml => OutputFormat::Yaml,
            OutputFormatArg::Toml => OutputFormat::Toml,
        }
    }
}

fn main() {
    let args = Args::parse();

    let source = match fs::read_to_string(&args.input) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file {}: {}", args.input, err);
            process::exit(1);
        }
    };

    if let Err(err) = run_stria(&source, &args.input, args.format.into()) {
        let error_msg = if args.lsp {
            err.format_lsp(&source, &args.input)
        } else {
            err.format_rich(&source, &args.input)
        };
        eprintln!("{}", error_msg);
        process::exit(1);
    }
}

fn run_stria(source: &str, filename: &str, format: OutputFormat) -> Result<(), StriaError> {
    // Tokenize
    let mut tokenizer = Tokenizer::new();
    let tokens = tokenizer.tokenize(source)?;

    // Parse
    let mut parser = StriaParser::new(tokens);
    let ast = parser.parse()?;

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast, Some(filename))?;

    // Execute
    let mut executor = Executor::new();
    executor.set_structs(analyzer.get_structs().clone());
    let result = executor.execute(&ast)?;

    // Output in the specified format
    let output = OutputGenerator::generate(&result, format)?;
    println!("{}", output);

    Ok(())
}
