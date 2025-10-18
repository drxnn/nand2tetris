#![allow(unused)]
use std::fmt::format;
use std::fs::{self, File, OpenOptions};
use std::i16;

use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::{env, io};

#[allow(dead_code)]
fn main() -> io::Result<()> {
    let mut code_writer = CodeWriter::new();

    let mut args = env::args();
    args.next();

    let file_name = args.next().expect("Please provide a filename as argument");
    let buffer = fs::read_to_string(file_name)?;

    let mut parser = Parser::new(&buffer);

    for i in 0..=parser.lines.len() {
        let command_type = parser.command_type();
        println!(
            "current line is:, {}",
            parser.current.as_ref().unwrap_or(&"default".to_string())
        );

        /*------------------------------------------------------------------------------------------------------------------------------ */

        // uncluster this later
        if command_type == VMCOMMAND::CArithmetic {
            let command_arg = parser.current.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing command arg")
            })?;
            code_writer.write_arithmetic(&command_arg);
        } else if command_type == VMCOMMAND::CPop || command_type == VMCOMMAND::CPush {
            let command_arg = parser.current.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing command arg")
            })?;
            let mut command_iter = command_arg.split_whitespace();
            let command_name = command_iter.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing command name")
            })?;
            let segment = command_iter.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing command name")
            })?;
            let index_str = command_iter
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing index"))?;

            let index = index_str.parse::<i16>().ok().unwrap();

            code_writer.write_push_pop(command_name, segment, index);
        };
        parser.advance();
    }

    Ok(())
}

#[derive(PartialEq, Debug)]
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
            let temp = self.lines[self.pos].clone();
            self.current = Some(temp);
            self.pos += 1;
        } else {
            self.current = None;
        }
    }

    fn command_type(&self) -> VMCOMMAND {
        match &self.current {
            Some(c) if c.contains("pop") => VMCOMMAND::CPop,
            Some(c) if c.contains("push") => VMCOMMAND::CPush,
            Some(c)
                if ["add", "sub", "lt", "eq", "gt", "and", "or", "not", "neg"]
                    .iter()
                    .any(|&x| c.contains(x)) =>
            {
                VMCOMMAND::CArithmetic
            }
            _ => VMCOMMAND::CReturn, // just for now
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
    label_index: usize,
}

impl CodeWriter {
    fn new() -> Self {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .write(true)
            .create(true)
            .open("output.asm")
            .unwrap();

        Self {
            file: Some(BufWriter::new(file)),
            current_file: None,
            label_index: 0,
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

        let mut machine_code = String::from("");

        let mut true_label = String::from("TRUE_");
        let mut end_label = String::from("END_");

        true_label.push_str(command);
        true_label.push_str(&self.label_index.to_string());
        end_label.push_str(command);
        end_label.push_str(&self.label_index.to_string());

        match command {
            "not" => machine_code.push_str("@SP\nA=M-1\nM=!M\n"),
            "neg" => machine_code.push_str("@SP\nA=M-1\nM=-M\n"),
            "add" => machine_code.push_str("@SP\nA=M-1\nD=M\nA=A-1\nD=D+M\nM=D\n@SP\nD=M-1\nM=D\n"),
            "sub" => machine_code.push_str("@SP\nA=M-1\nD=M\nA=A-1\nD=M-D\nM=D\n@SP\nD=M-1\nM=D\n"),
            "and" => machine_code.push_str("@SP\nA=M-1\nD=M\nA=A-1\nD=D&M\nM=D\n@SP\nD=M-1\nM=D\n"),
            "or" => machine_code.push_str("@SP\nA=M-1\nD=M\nA=A-1\nD=D|M\nM=D\n@SP\nD=M-1\nM=D\n"),
            "eq" => {
                machine_code = format!(
                    "@SP\nAM=M-1\nD=M\nA=A-1\nD=M-D\n@{true_label}\nD;JEQ\n@SP\nA=M-1\nM=0\n@{end_label}\n0;JMP\n({true_label})\n@SP\nA=M-1\nM=-1\n({end_label})\n"
                )
            }

            "lt" => {
                machine_code = format!(
                    "@SP\nAM=M-1\nD=M\nA=A-1\nD=M-D\n@{true_label}\nD;JLT\n@SP\nA=M-1\nM=0\n@{end_label}\n0;JMP\n({true_label})\n@SP\nA=M-1\nM=-1\n({end_label})\n"
                )
            }

            "gt" => {
                machine_code = format!(
                    "@SP\nAM=M-1\nD=M\nA=A-1\nD=M-D\n@{true_label}\nD;JGT\n@SP\nA=M-1\nM=0\n@{end_label}\n0;JMP\n({true_label})\n@SP\nA=M-1\nM=-1\n({end_label})\n"
                )
            }

            _ => panic!("Unknown arithmetic command"),
        };

        self.label_index += 1;

        self.file
            .as_mut()
            .unwrap()
            .write_all(machine_code.as_bytes())?;
        self.file.as_mut().unwrap().flush()?;

        Ok(())
    }
    fn write_push_pop(&mut self, command: &str, segment: &str, index: i16) -> io::Result<()> {
        let mut machine_code = String::from("");
        match command {
            /*

            @{segment}\nD=M\n@{index}\nA={index}\nD=D+A\n@SP\nA=M\nM=D\n@SP\nM=M+1



             */
            "push" => {
                // make sure if theres no index or segment that you just push a constant

                machine_code = if segment == "constant" {
                    format!("@{index}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", index = index)
                } else {
                    format!(
                        "@{segment}\nD=M\n@{index}\nA={index}\nD=D+A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                        segment = segment,
                        index = index,
                    )
                };
            }

            "pop" => {
                // so pop retrieves the last element from the stack, then moves SP so it points to the element below it
                // memory segments: LCL(local), ARG, THIS, THAT
                let machine_code = format!("@{segment}\n");
            }
            _ => todo!(),
        };

        self.file
            .as_mut()
            .unwrap()
            .write_all(machine_code.as_bytes())?;
        self.file.as_mut().unwrap().flush()?;

        Ok(())
    }

    fn close() {
        // close the file
    }
}

//
