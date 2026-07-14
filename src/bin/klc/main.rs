// This will be the primary entry point of our compiler.
// Just open files, read them into memory and find a way of ateast parsing them.
mod ast;
mod ast_lowering;
mod dialect;
mod jit;
mod klir_lowering;

use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

use clap::Parser;

#[derive(Parser)]
#[command(version, about = "Kisumu_lang JIT example", long_about = None)]
struct Cli {
    /// Input Kaleidoscope source file
    #[arg(long = "input", value_name = "FILE")]
    input: PathBuf,

    /// Function name to execute from the source module
    #[arg(long = "fn", default_value = "main")]
    function: String,

    /// Integer argument passed to the JIT function
    #[arg(long = "arg", short = 'a')]
    arg: i64,

    #[arg(long = "output", short = 'o', default_value = "false")]
    output: bool,

    #[arg(long = "llvm-ir", short = 'r', default_value = "false")]
    llvm_ir: bool,
}

// this routine helps as have machine code now rather than later but its rather useless.
// An objdump of the file can yield more information.!
fn compile_llvm_ir<P: AsRef<Path>>(llvm_ir: &str, output: P) -> std::io::Result<()> {
    let mut child = Command::new("clang")
        .args([
            "--target=riscv64-unknown-linux-gnu",
            "-v",
            // "--gcc-toolchain=/home/r/riscv",
            // "--sysroot=/home/rojunga/riscv/sysroot",
            // "--ld-path=/home/rojunga/.cargo/bin/wild",
            "--ld-path=/home/gevurah/.cargo/bin/wild",
            "-x",
            "ir",
            "-",
            "-o",
        ])
        .arg(output.as_ref())
        .stdin(Stdio::piped())
        .spawn()?;

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(llvm_ir.as_bytes())?;

    let status = child.wait()?;

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "clang failed",
        ));
    }

    Ok(())
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let src = std::fs::read_to_string(&cli.input)?;
    let file_name = &cli.input.file_prefix().unwrap();
    let (result, llvm_out) = jit::exec_fn(&src, &cli.function, cli.arg)?;
    // generate binary.
    if cli.output {
        compile_llvm_ir(llvm_out.as_str(), file_name)?;
    }
    if cli.llvm_ir {
        println!("{}", llvm_out);
    }
    println!("JIT result ({}({})): {}", cli.function, cli.arg, result);
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
