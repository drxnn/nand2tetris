#![allow(unused)]
use std::fmt::format;
use std::fs::{self, File, OpenOptions};
use std::i16;

use std::io::{BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;
use std::{env, io};

#[allow(dead_code)]

fn process_file(file_name: &Path, mut code_writer: &mut CodeWriter) -> io::Result<()> {
    println!("beginning of process_file");
    let buffer = fs::read_to_string(file_name)?;

    let mut parser = Parser::new(&buffer);

    if code_writer.current_function.is_none() {
        if let Some(file_stem) = code_writer.current_file.clone() {
            code_writer.current_function = Some(file_stem);
        }
    }

    parser.advance();
    while parser.has_more_commands() {
        println!("current is: {:?}", parser.current);
        println!("current file is {:?}", code_writer.current_file);
        println!("current function is {:?}", code_writer.current_function);
        let command_type = parser.command_type();

        match command_type {
            VMCOMMAND::CArithmetic => {
                let command_arg = parser.current.as_ref().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "missing command arg")
                })?;
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

        parser.advance();
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
                    println!("we are in file: {:?}", path);
                    process_file(&path, &mut code_writer)?;
                }
            }
        }
    }

    code_writer.close()?;
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
    None,
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
            Some(c) if c.starts_with("function") => VMCOMMAND::CFunction,
            Some(c) if c.starts_with("label") => VMCOMMAND::CLabel,
            Some(c) if c.starts_with("if-goto") => VMCOMMAND::CIf,
            Some(c) if c.starts_with("goto") => VMCOMMAND::CGoto,
            Some(c) if c.starts_with("call") => VMCOMMAND::CCall,
            Some(c) if c.starts_with("return") => VMCOMMAND::CReturn,
            Some(_) => unreachable!("All cases should be handled"),
            None => VMCOMMAND::None,
        }
    }

    fn arg_one(&self) -> Option<&str> {
        match self.command_type() {
            VMCOMMAND::CArithmetic => self
                .current
                .as_deref()
                .and_then(|s| s.split_whitespace().next()),
            VMCOMMAND::CReturn => None,
            _ => self
                .current
                .as_deref()
                .and_then(|s| s.split_whitespace().nth(1)),
        }
    }
    fn arg_two(&self) -> Option<&str> {
        let ct = self.command_type();
        match ct {
            VMCOMMAND::CCall | VMCOMMAND::CFunction | VMCOMMAND::CPop | VMCOMMAND::CPush => {
                let s = self.current.as_ref()?;

                s.split_whitespace().nth(2)
            }
            VMCOMMAND::CIf | VMCOMMAND::CGoto | VMCOMMAND::CLabel => {
                let s = self.current.as_ref()?;
                s.split_whitespace().nth(1)
            }
            _ => None,
        }
    }
}

struct CodeWriter {
    file: Option<BufWriter<File>>,
    current_file: Option<String>,
    label_index: usize,
    current_function: Option<String>,
}

impl CodeWriter {
    fn new() -> io::Result<Self> {
        let file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open("output.asm")
            .unwrap();

        let mut writer = Self {
            file: Some(BufWriter::new(file)),
            current_file: None,
            label_index: 0,
            current_function: None,
        };
        writer.write_init()?;

        Ok(writer)
    }

    fn write_init(&mut self) -> io::Result<()> {
        let asm_to_write = format!(
            r#"
@256
D=A
@SP
M=D
"#
        );
        if let Some(f) = self.file.as_mut() {
            f.write_all(asm_to_write.as_bytes())?;
            f.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "no output file"));
        }

        self.write_call("Sys.init", 0)?;

        Ok(())
    }
    fn write_label(&mut self, label: &str) -> io::Result<()> {
        let f_name = match self.current_function.as_deref() {
            Some(n) => n,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "label is outside a function",
                ));
            }
        };
        let asm_to_write = format!("({}${})\n", f_name, label);
        if let Some(f) = self.file.as_mut() {
            f.write_all(asm_to_write.as_bytes())?;
            f.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "no output file"));
        }

        Ok(())
    }
    fn write_goto(&mut self, label: &str) -> io::Result<()> {
        let f_name = self
            .current_function
            .as_deref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "goto outside function"))?;
        let asm_to_write = format!("@{f_name}${label}\n0;JMP\n", f_name = f_name, label = label);
        if let Some(f) = self.file.as_mut() {
            f.write_all(asm_to_write.as_bytes())?;
            f.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "no output file"));
        }

        Ok(())
    }
    fn write_if_goto(&mut self, label: &str) -> io::Result<()> {
        let f_name = self
            .current_function
            .as_deref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "if-goto outside function"))?;
        let asm_to_write = format!(
            "@SP\nAM=M-1\nD=M\n@{f_name}${label}\nD;JNE\n",
            f_name = f_name,
            label = label
        );
        if let Some(f) = self.file.as_mut() {
            f.write_all(asm_to_write.as_bytes())?;
            f.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "no output file"));
        }

        Ok(())
    }
    fn write_call(&mut self, callee: &str, n_args: i16) -> io::Result<()> {
        let caller = self
            .current_function
            .as_deref()
            .unwrap_or("BOOT")
            .to_string();
        let return_label = self.generate_return_label(&caller);
        let asm_to_write = format!(
            r#"
@{return_label}
D=A
@SP
A=M
M=D
@SP
M=M+1
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
D=M 
@{n_args}
D=D-A 
@5
D=D-A 
@ARG
M=D 
@SP
D=M
@LCL
M=D
@{callee}
0;JMP
({return_label})
        "#
        );

        if let Some(f) = self.file.as_mut() {
            f.write_all(asm_to_write.as_bytes())?;
            f.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "no output file"));
        }

        Ok(())
    }
    fn write_return(&mut self) -> io::Result<()> {
        let asm_to_write = format!(
            r#"
@LCL
D=M
@R13 
M=D 
@5
D=D-A 
A=D
D=M
@R14
M=D 
@SP
AM=M-1
D=M 
@ARG
A=M 
M=D 
@ARG
D=M 
D=D+1
@SP
M=D
@R13
AM=M-1
D=M
@THAT
M=D 
@R13
AM=M-1
D=M
@THIS
M=D  
@R13
AM=M-1
D=M
@ARG
M=D 
@R13
AM=M-1
D=M
@LCL
M=D 
@R14
A=M
0;JMP
"#
        );

        if let Some(f) = self.file.as_mut() {
            f.write_all(asm_to_write.as_bytes())?;
            f.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "no output file"));
        }

        Ok(())
    }
    fn write_function(&mut self, f_name: &str, n_locals: i16) -> io::Result<()> {
        self.current_function = Some(f_name.to_string());

        let mut push_locals = String::new();

        for _i in 0..n_locals {
            push_locals.push_str("@0\nD=A\nM=D\n@SP\nM=M+1\n");
        }

        let asm_to_write = format!(
            "({f_name})\n{push_locals}",
            f_name = f_name,
            push_locals = push_locals,
        );
        if let Some(f) = self.file.as_mut() {
            f.write_all(asm_to_write.as_bytes())?;
            f.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "no output file"));
        }

        Ok(())
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
            "add" => machine_code.push_str("@SP\nA=M-1\nD=M\nA=A-1\nD=D+M\nM=D\nD=A+1\n@SP\nM=D\n"),
            "sub" => machine_code.push_str("@SP\nA=M-1\nD=M\nA=A-1\nD=M-D\nM=D\nD=A+1\n@SP\nM=D\n"),
            "and" => machine_code.push_str("@SP\nA=M-1\nD=M\nA=A-1\nD=D&M\nM=D\nD=A+1\n@SP\nM=D\n"),
            "or" => machine_code.push_str("@SP\nA=M-1\nD=M\nA=A-1\nD=D|M\nM=D\nD=A+1\n@SP\nM=D\n"),
            "eq" => {
                machine_code = format!(
                    "@SP\nAM=M-1\nD=M\nA=A-1\nD=M-D\n@{true_label}\nD;JEQ\n@SP\nA=M-1\nM=0\n@{end_label}\n0;JMP\n({true_label})\n@SP\nA=M-1\nM=-1\n({end_label})\n",
                    true_label = true_label,
                    end_label = end_label
                )
            }

            "lt" => {
                machine_code = format!(
                    "@SP\nAM=M-1\nD=M\nA=A-1\nD=M-D\n@{true_label}\nD;JLT\n@SP\nA=M-1\nM=0\n@{end_label}\n0;JMP\n({true_label})\n@SP\nA=M-1\nM=-1\n({end_label})\n",
                    true_label = true_label,
                    end_label = end_label
                )
            }

            "gt" => {
                machine_code = format!(
                    "@SP\nAM=M-1\nD=M\nA=A-1\nD=M-D\n@{true_label}\nD;JGT\n@SP\nA=M-1\nM=0\n@{end_label}\n0;JMP\n({true_label})\n@SP\nA=M-1\nM=-1\n({end_label})\n",
                    true_label = true_label,
                    end_label = end_label
                )
            }

            _ => panic!("Unknown arithmetic command"),
        };

        self.label_index += 1;

        if let Some(f) = self.file.as_mut() {
            f.write_all(machine_code.as_bytes())?;
            f.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "no output file"));
        }

        Ok(())
    }
    fn write_push_pop(&mut self, command: &str, segment: &str, index: usize) -> io::Result<()> {
        let segment = segment.to_lowercase();
        println!("segment is: {}", segment);
        let mut machine_code = String::from("");

        match command {
            "push" => {
                match segment.as_str() {
                    "constant" => {
                        machine_code =
                            format!("@{index}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", index = index);
                    }
                    "static" => {
                        println!("we are in static seg");
                        let f_name = self.current_file.as_ref().ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "no current file")
                        })?;

                        println!("after f name");
                        machine_code = format!(
                            "@{f_name}.{index}\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                            f_name = f_name,
                            index = index
                        );
                    }
                    "this" => {
                        machine_code = format!(
                            "@THIS\nD=M\n@{index}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                            index = index
                        );
                    }
                    "that" => {
                        machine_code = format!(
                            "@THAT\nD=M\n@{index}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                            index = index
                        );
                    }

                    "argument" => {
                        machine_code = format!(
                            "@ARG\nD=M\n@{index}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                            index = index
                        )
                    }
                    "pointer" => {
                        machine_code = format!(
                            "@3\nD=A\n@{index}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                            index = index,
                        )
                    }
                    "temp" => {
                        machine_code = format!(
                            "@R5\nD=A\n@{index}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                            index = index,
                        )
                    }
                    "local" => {
                        machine_code = format!(
                            "@LCL\nD=M\n@{index}\nA=D+A\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                            index = index,
                        )
                    }
                    _ => {
                        panic!("invalid push seg: {}", segment);
                    }
                };
            }

            "pop" => match segment.as_str() {
                "static" => {
                    let f_name = self
                        .current_file
                        .as_ref()
                        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no current file"))?;

                    machine_code = format!(
                        "@{f_name}.{index}\nD=A\n@R15\nM=D\n@SP\nAM=M-1\nD=M\n@R15\nA=M\nM=D\n",
                        f_name = f_name,
                        index = index
                    );
                }
                "local" => {
                    machine_code = format!(
                        "@LCL\nD=M\n@{index}\nD=D+A\n@R15\nM=D\n@SP\nAM=M-1\nD=M\n@R15\nA=M\nM=D\n",
                        index = index
                    );
                }
                "pointer" => {
                    machine_code = format!(
                        "@3\nD=A\n@{index}\nD=D+A\n@R15\nM=D\n@SP\nAM=M-1\nD=M\n@R15\nA=M\nM=D\n",
                        index = index
                    );
                }
                "temp" => {
                    machine_code = format!(
                        "@5\nD=A\n@{index}\nD=D+A\n@R15\nM=D\n@SP\nAM=M-1\nD=M\n@R15\nA=M\nM=D\n",
                        index = index
                    );
                }
                "argument" => {
                    machine_code = format!(
                        "@ARG\nD=M\n@{index}\nD=D+A\n@R15\nM=D\n@SP\nAM=M-1\nD=M\n@R15\nA=M\nM=D\n",
                        index = index
                    );
                }
                "this" => {
                    machine_code = format!(
                        "@THIS\nD=M\n@{index}\nD=D+A\n@R15\nM=D\n@SP\nAM=M-1\nD=M\n@R15\nA=M\nM=D\n",
                        index = index
                    );
                }
                "that" => {
                    machine_code = format!(
                        "@THAT\nD=M\n@{index}\nD=D+A\n@R15\nM=D\n@SP\nAM=M-1\nD=M\n@R15\nA=M\nM=D\n",
                        index = index
                    );
                }
                _ => {
                    panic!("invalid pop seg: {}", segment);
                }
            },
            _ => {
                panic!("invalid command : {}", command);
            }
        };

        println!("machine code is: {}", machine_code);
        if let Some(f) = self.file.as_mut() {
            f.write_all(machine_code.as_bytes())?;
            f.flush()?;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "no output file"));
        }

        Ok(())
    }

    fn generate_return_label(&mut self, f_name: &str) -> String {
        let label_to_return = format!("{f_name}$ret.{}", self.label_index, f_name = f_name);
        self.label_index += 1;

        label_to_return
    }

    fn close(&mut self) -> io::Result<()> {
        if let Some(mut w) = self.file.take() {
            w.flush()?;
        }
        Ok(())
    }
}
