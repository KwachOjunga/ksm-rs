pub mod ast;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod token;
/*
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut file_path = None;

    // A simple command-line arguments parser supporting "-file path" and "--file path"
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-file" | "--file" => {
                if i + 1 < args.len() {
                    file_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: missing value for file option");
                    std::process::exit(1);
                }
            }
            // support positional file path argument as well
            val => {
                if !val.starts_with('-') {
                    file_path = Some(val.to_string());
                }
                i += 1;
            }
        }
    }

    if let Some(path) = file_path {
        if let Err(err) = repl::read_file(&path) {
            eprintln!("File execution failed: {}", err);
            std::process::exit(1);
        }
    } else {
        println!("No input file provided - entering interactive mode");
        if let Err(err) = repl::start() {
            eprintln!("REPL failed: {}", err);
            std::process::exit(1);
        }
    }
}*/
