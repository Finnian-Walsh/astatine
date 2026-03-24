mod cli;
mod generator;
mod lexer;
mod parser;
mod syntax;

use std::{fs, path::Path, process::Command};

use clap::Parser as ClapParser;
use cli::Cli;
use color_eyre::eyre::{Context, Result, eyre};

use crate::{generator::Generator, lexer::Lexer, parser::Parser};

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();
    let file_path = fs::canonicalize(&args.file_name)?;

    let output_file = args.output.map(Ok).unwrap_or_else(|| {
        file_path
            .file_name()
            .map(|s| match Path::new(s).file_stem() {
                Some(stem) => stem,
                None => s,
            })
            .map(|s| s.to_string_lossy())
            .map(|s| s.to_string())
            .ok_or_else(|| eyre!("File name could not be resolved"))
    })?;

    let contents = fs::read_to_string(&file_path)
        .wrap_err_with(|| format!("Failed to read file `{}`", args.file_name))?;

    let lexer = Lexer::new(&contents);
    let tokens = lexer.tokenize()?;

    if args.tokens {
        let tokens_file = format!("{output_file}.tokens");
        fs::write(tokens_file, format!("{tokens:#?}"))?;
    }

    let ast = Parser::new(&tokens).parse()?;

    if args.ast {
        let ast_file = format!("{output_file}.ast");
        fs::write(ast_file, format!("{ast:#?}"))?;
    }

    let asm = "
        global _start\n\
        _start:\n\
        \tmov rax, 60\n\
        \tmov rdi, 69\n\
        \tsyscall\n\
        ";

    let generator = Generator::new(&ast);
    dbg!(generator);

    let asm_file = format!("{output_file}.asm");
    fs::write(&asm_file, asm)?;

    if !Command::new("nasm")
        .arg("-felf64")
        .arg(&asm_file)
        .status()?
        .success()
    {
        return Err(eyre!("Failed to assemble"));
    };

    if !args.asm {
        fs::remove_file(asm_file)?;
    }

    let object_file = format!("{output_file}.o");

    if !Command::new("ld")
        .arg(&object_file)
        .arg("-o")
        .arg(output_file)
        .status()?
        .success()
    {
        return Err(eyre!("Failed to link object file"));
    };

    if !args.object {
        fs::remove_file(object_file)?;
    }

    Ok(())
}
