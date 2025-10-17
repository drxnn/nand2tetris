#![allow(unused)]
use std::fs::{self, File, OpenOptions};
use std::i16;

use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::{env, io};

#[allow(dead_code)]
fn main() -> io::Result<()> {
    let code_writer = CodeWriter::new();
    // a parser module
    // codewriter module

    // or if a folder is given as input, create a new parser for each file. 1 Codewriter does the job
    let mut args = env::args();
    args.next();

    let file_name = args.next().expect("Please provide a filename as argument");
    let buffer = fs::read_to_string(file_name)?;

    let mut parser = Parser::new(&buffer);

    Ok(())
}

#[derive(PartialEq)]
enum VMCOMMAND {
    CArithmetic,
    CPush,
    CPop,
    CLabel,
    CGoto,
    CIf,
    CFunction,
    CReturn,
    CCall,
}

struct Parser {
    lines: Vec<String>,
    pos: usize,
    current: Option<String>,
}

impl Parser {
    fn new(file: &str) -> Self {
        let mut lines = Vec::new();

        for line in file.lines() {
            let sanitized_line = line.split("//").next().unwrap_or("").trim();
            if !sanitized_line.is_empty() {
                lines.push(sanitized_line.to_string());
            }
        }

        Self {
            lines,
            pos: 0,
            current: None,
        }
    }
    fn has_more_commands(&self) -> bool {
        self.pos < self.lines.len()
    }

    fn advance(&mut self) {
        if self.has_more_commands() {
            // advance to next current and update pos
            let temp = self.lines[self.pos].clone();
            self.current = Some(temp);
            self.pos += 1;
        } else {
            self.current = None;
        }
    }

    fn command_type(&self) -> VMCOMMAND {
        let c = self.current.as_ref().unwrap().to_string();

        match c {
            c if c.contains("pop") => VMCOMMAND::CPop,
            c if c.contains("push") => VMCOMMAND::CPush,
            c if ["add", "sub", "lt", "eq", "gt", "and", "or", "not", "neg"]
                .iter()
                .any(|&x| c.contains(x)) =>
            {
                VMCOMMAND::CArithmetic
            }
            _ => panic!("Unknown command: {}", c),
        }
    }

    fn arg_one(&self) -> Option<String> {
        if self.command_type() == VMCOMMAND::CArithmetic {
            self.current.clone()
        } else if self.command_type() == VMCOMMAND::CReturn {
            None
        } else {
            self.current
                .as_ref()
                .and_then(|x| x.split_whitespace().nth(1))
                .map(|x| x.to_string())
        }
    }
    fn arg_two(&self) -> Option<i16> {
        let ct = self.command_type();
        match ct {
            VMCOMMAND::CCall | VMCOMMAND::CFunction | VMCOMMAND::CPop | VMCOMMAND::CPush => {
                let s = self.current.as_ref()?;
                s.split_whitespace()
                    .nth(2)
                    .and_then(|x| x.parse::<i16>().ok())
            }
            _ => None,
        }
    }
}

struct CodeWriter {
    file: Option<BufWriter<File>>,
    current_file: Option<String>,
}

impl CodeWriter {
    fn new() -> Self {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .open("output.asm")
            .unwrap();

        Self {
            file: Some(BufWriter::new(file)),
            current_file: None,
        }
    }
    fn set_file_name(&mut self, fname: &str) {
        self.current_file = Some(fname.to_string());
    }
    fn write_arithmetic(&mut self, command: &str) -> io::Result<()> {
        let writer = self
            .file
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "output file not opened "))?;

        let machine_code = String::from("");

        /*

              if command == not
              @SP
              A=M-1
              M=!M
              -------------------------------------------------------------------------------------------------------------------
              if command == neg
                              @SP
                              A=M-1
                              M=-M
              -------------------------------------------------------------------------------------------------------------------
              if command == add
              @SP
              A=M-1
              D=M
              A=A-1
              D=D+M
              M=D
              @SP
              D=M-1
              M=D
              // if command == sub

              @SP
              A=M-1
              D=M
              A=A-1
              D=D-M
              M=D
              @SP
              D=M-1
              M=D

              // if command == lt
                @SP
        A=M-1
        D=M
        A=A-1
        D=D+M
        M=D
        @SP
        D=M-1
        M=D


              -------------------------------------------------------------------------------------------------------------------

              */

        Ok(())
    }
    fn write_push_pop(&mut self, command: VMCOMMAND, segment: &str, index: i16) {
        todo!()
    }

    fn close() {
        // close the file
    }
}

//
