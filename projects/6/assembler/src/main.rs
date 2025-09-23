use std::collections::HashMap;
use std::env;

use std::fs::File;
use std::io::{self, Read};

enum CommandType {
    Acommand, //for @Xxx where xxx is either a symbol or a decimal number
    Ccommand, // for dest=comp; jump
    Lcommand, // for (xxx) where Xxx is a symbol. like (LOOP)
}
struct Parser {
    lines: Vec<String>,
    pos: usize,              // line pos
    current: Option<String>, // command
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
        }
    }

    fn command_type(&self) -> CommandType {
        match self.current.as_ref().unwrap().chars().next().unwrap() {
            '@' => CommandType::Acommand,
            '(' => CommandType::Lcommand,
            _ => CommandType::Ccommand,
        }
    }

    fn symbol(&self) -> String {
        // right now it can only be called if its A or L command and its up to the caller to decide, change in the future maybe to Option<>
        if self.current.as_ref().unwrap().starts_with('@') {
            return self.current.as_ref().unwrap()[1..].to_string();
        } else if self.current.as_ref().unwrap().starts_with('(')
            && self.current.as_ref().unwrap().ends_with(')')
        {
            return self.current.as_ref().unwrap()[1..self.current.as_ref().unwrap().len() - 1]
                .to_string();
        } else {
            return "".to_string();
        }
    }

    fn dest(&self) -> Option<String> {
        // read into split_once to make code a little better
        match self.current.as_ref().unwrap().split_once("=") {
            Some((dest, _)) => Some(dest.to_string()),
            _ => None,
        }
    }

    fn comp(&self) -> Option<String> {
        // fix
        // handle if it has jump instruction then strip it
        let current = self.current.as_ref().unwrap();
        let right = current.split_once("=").map(|(_, c)| c).unwrap_or(current);

        let temp = right.split_once(";").map(|(c, _)| c).unwrap_or(right);

        Some(temp.to_string())
    }
    fn jump(&self) -> Option<String> {
        match self.current.as_ref().unwrap().split_once(";") {
            Some((_, jump)) => Some(jump.to_string()),
            _ => None,
        }
    }
}

// translate mnemonics to binary
struct CodeBinary;

impl CodeBinary {
    // A is the address register
    // M means RAM[A]
    // D is the data register
    // fn dest_to_binary(dest: &str) -> (u32, u32, u32) {
    //     match dest {
    //         "D" => (0, 1, 0),
    //     }
    // }
    fn comp_to_binary(comp: String) {
        // return binary
    }
    fn jump_to_binary(jump: &str) -> (u32, u32, u32) {
        match jump {
            "JGT" => (0, 0, 1),
            "JEQ" => (0, 1, 0),
            "JGE" => (0, 1, 1),
            "JLT" => (1, 0, 0),
            "JNE" => (1, 0, 1),
            "JLE" => (1, 1, 0),
            "JMP" => (1, 1, 1),
            _ => (0, 0, 0),
        }
    }
}

fn main() -> io::Result<()> {
    let predefined_symbols = vec![
        ("SP", 0),
        ("LCL", 1),
        ("ARG", 2),
        ("THIS", 3),
        ("THAT", 4),
        ("R0", 0),
        ("R1", 1),
        ("R2", 2),
        ("R3", 3),
        ("R4", 4),
        ("R5", 5),
        ("R6", 6),
        ("R7", 7),
        ("R8", 8),
        ("R9", 9),
        ("R10", 10),
        ("R11", 11),
        ("R12", 12),
        ("R13", 13),
        ("R14", 14),
        ("R15", 15),
        ("SCREEN", 16384),
        ("KBD", 24576),
    ];
    let symbol_table: HashMap<_, u32> = predefined_symbols.into_iter().collect();

    let mut arguments = env::args();
    arguments.next();

    let file_to_compile = arguments.next().unwrap();
    let mut f = File::open(file_to_compile)?;

    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    let mut parser = Parser::new(&buffer);
    let mut binary_file: Vec<&Vec<u32>> = Vec::new();

    println!("here is file to process: {}", buffer);

    for line in buffer.lines() {}

    Ok(())
}
