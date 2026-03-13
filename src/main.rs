mod cli;
mod lexer;
mod parser;
mod syntax;

use std::{fs, path::Path, process::Command};

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::{Context, Result, eyre};
use thiserror::Error;

use crate::lexer::Lexer;

#[derive(Debug, Error)]
#[error("Could not convert to assembly")]
struct AssemblyConversionError;

// fn tokens_to_asm(tokens: Vec<Token>) -> Result<String, AssemblyConversionError> {
//     let mut output = "\
//         global _start\n\
//         _start:\n\
//         "
//     .to_string();
//
//     for (idx, token) in tokens.iter().enumerate() {
//         match token {
//             Token::Keyword(keyword) => match keyword {
//                 Keyword::Return => {
//                     if tokens[idx + 2] != Token::Semicolon {
//                         return Err(AssemblyConversionError);
//                     }
//
//                     let next_token = &tokens[idx + 1];
//
//                     if !matches!(
//                         next_token,
//                         Token::Literal { kind: _, value: _ } | Token::Identifier(_)
//                     ) {
//                         return Err(AssemblyConversionError);
//                     }
//
//                     output.push_str(&format!(
//                         "\
//                         \tmov rax, 60\n\
//                         \tmov rdi, {}\n\
//                         \tsyscall\n",
//                         next_token
//                     ));
//                 }
//                 Keyword::Struct => {} // println!("Struct!"),
//                 _ => {}
//             },
//             Token::Semicolon => {} // println!("Semicolon!"),
//             _ => {}
//         }
//     }
//
//     Ok(output)
// }

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
    println!("{tokens:?}");

    // let asm = tokens_to_asm(tokens)?;
    let asm = "
        global _start\n\
        _start:\n\
        \tmov rax, 60\n\
        \tmov rdi, 69\n\
        \tsyscall\n\
        ";
    print!("{asm}");

    let asm_file = format!("{output_file}.asm");
    fs::write(&asm_file, asm)?;

    if !Command::new("nasm")
        .arg("-felf64")
        .arg(asm_file)
        .status()?
        .success()
    {
        return Err(eyre!("Failed to assemble"));
    };

    if !Command::new("ld")
        .arg(format!("{output_file}.o"))
        .arg("-o")
        .arg(output_file)
        .status()?
        .success()
    {
        return Err(eyre!("Failed to link object file"));
    };

    Ok(())
}
