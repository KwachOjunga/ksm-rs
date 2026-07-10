use crate::interpreter::Interpreter;
use crate::lexer::{Config, Lexer};
use crate::parser::Parser;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

const PROMPT: &str = ">>> ";

pub fn start() -> io::Result<()> {
    println!("Welcome to the Kisumu Lang REPL!");
    println!("Type your code below. Type 'exit' to quit.");
    println!("------------------------------------------");

    let mut interpreter = Interpreter::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("{}", PROMPT);
        stdout.flush()?;

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            // EOF reached
            println!("\nGoodbye!");
            break;
        }

        let trimmed = line.trim();
        if trimmed == "exit" {
            println!("Exiting REPL. Goodbye!");
            break;
        }

        if trimmed.is_empty() {
            println!("Please enter some code or type 'exit' to quit.");
            continue;
        }

        let mut lexer = Lexer::new(&line, Config::default());
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();

        let errors = parser.errors();
        if !errors.is_empty() {
            println!("Parser errors:");
            for err in errors {
                println!("  {}", err);
            }
            continue;
        }

        let result = interpreter.eval(&program);
        println!("{}", result.inspect());
    }

    Ok(())
}

pub fn read_file<P: AsRef<Path>>(filename: P) -> io::Result<()> {
    let clean_path = filename.as_ref();
    
    // Check if path is valid (mimicking the path validation)
    if clean_path.to_str().is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid file path"));
    }

    println!("Reading from file: {}", clean_path.display());

    let file = File::open(clean_path)?;
    let reader = io::BufReader::new(file);
    let mut content = String::new();

    for line in reader.lines() {
        content.push_str(&line?);
        content.push('\n');
    }

    let mut lexer = Lexer::new(&content, Config::default());
    let mut parser = Parser::new(&mut lexer);
    let program = parser.parse_program();

    let errors = parser.errors();
    if !errors.is_empty() {
        println!("Parser errors:");
        for err in errors {
            println!("  {}", err);
        }
        return Err(io::Error::new(io::ErrorKind::InvalidData, "parsing failed"));
    }

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(&program);
    println!("{}", result.inspect());

    Ok(())
}
