mod ast;
mod utils;
pub use ast::*;
use combine::EasyParser;

// Parser entry point for the program
pub fn parse_program(src: &str) -> Result<Vec<Function>, String> {
    use utils::program_;
    program_()
        .easy_parse(src)
        .map(|(funcs, _rest)| funcs)
        .map_err(|err| err.to_string())
}
