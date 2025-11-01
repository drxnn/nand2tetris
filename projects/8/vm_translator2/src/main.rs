#![allow(unused)]
use std::fmt::format;
use std::fs::{self, File, OpenOptions};
use std::i16;
mod code_writer;
mod parser;
use code_writer::CodeWriter;
use parser::{Parser, VMCOMMAND};

use std::io::{BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;
use std::{env, io};

fn process_file(file_name: &Path, mut code_writer: &mut CodeWriter) -> io::Result<()> {
    let buffer = fs::read_to_string(file_name)?;

    let mut parser = Parser::new(&buffer);

    while parser.has_more_commands() {
        parser.advance();

        let command_type = parser.command_type();

        match command_type {
            VMCOMMAND::CArithmetic => {
                let command_arg = parser.current.as_ref().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing command arg")
                })?;
                println!("command is: {}", command_arg);
                code_writer.write_arithmetic(&command_arg)?;
            }
            VMCOMMAND::CPop | VMCOMMAND::CPush => {
                let command_arg = parser.current.as_ref().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing command arg")
                })?;
                let mut command_iter = command_arg.split_whitespace();
                let command_name = command_iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing command name")
                })?;
                let segment = command_iter.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing segment name")
                })?;
                let index_str = command_iter
                    .next()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing index"))?;

                let index = index_str.parse::<usize>().ok().unwrap();

                println!(
                    "THE command name is: {}. The segment is: {}. The index is {}.",
                    command_name, segment, index
                );
                code_writer.write_push_pop(command_name, segment, index)?;
            }
            VMCOMMAND::CLabel => {
                let label = parser.arg_two().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing label name")
                })?;

                code_writer.write_label(label)?;
            }
            VMCOMMAND::CFunction => {
                let f_name = parser.arg_one().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing function name")
                })?;

                let n_locals: i16 = parser
                    .arg_two()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "missing numLocals in function")
                    })?
                    .parse()
                    .map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidInput, "invalid numLocals value")
                    })?;
                code_writer.write_function(&f_name, n_locals)?;
            }
            VMCOMMAND::CCall => {
                let n_args: i16 = parser
                    .arg_two()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "missing numLocals in function")
                    })?
                    .parse()
                    .map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidInput, "invalid numLocals value")
                    })?;

                let callee = parser.arg_one().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing function callee name")
                })?;

                code_writer.write_call(callee, n_args)?;
            }
            VMCOMMAND::CReturn => {
                code_writer.write_return()?;
            }
            VMCOMMAND::CIf => {
                let label = parser.arg_two().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing label name")
                })?;
                code_writer.write_if_goto(label)?;
            }
            VMCOMMAND::CGoto => {
                let label = parser.arg_two().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing label name")
                })?;
                code_writer.write_goto(label)?;
            }
            VMCOMMAND::None => {}
        }
    }
    Ok(())
}
fn main() -> io::Result<()> {
    let mut code_writer = CodeWriter::new()?;

    let mut args = env::args();
    args.next();

    let input_name = args.next().expect("Please provide a filename as argument");
    let input_path = PathBuf::from(input_name);
    if input_path.is_file() {
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid filename"))?
            .to_string();

        code_writer.current_file = Some(file_stem);
        process_file(&input_path, &mut code_writer)?;
    } else if input_path.is_dir() {
        for entry in input_path.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().unwrap().to_str().unwrap() == "vm" {
                    let file_stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid filename"))?
                        .to_string();
                    code_writer.current_file = Some(file_stem);

                    process_file(&path, &mut code_writer)?;
                }
            }
        }
    }

    code_writer.close()?;
    Ok(())
}
