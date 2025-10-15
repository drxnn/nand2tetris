use std::collections::HashMap;
use std::env;

use std::fs::{self, File};
use std::io::{self, Read, Write};

#[derive(PartialEq)]
enum CommandType {
    Acommand,
    Ccommand,
    Lcommand,
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

    fn command_type(&self) -> CommandType {
        match self.current.as_ref().and_then(|x| x.chars().next()) {
            Some('@') => CommandType::Acommand,
            Some('(') => CommandType::Lcommand,
            _ => CommandType::Ccommand,
        }
    }

    fn symbol(&self) -> String {
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
        match self.current.as_ref()?.split_once("=") {
            Some((dest, _)) => Some(dest.to_string()),
            _ => None,
        }
    }

    fn comp(&self) -> Option<String> {
        let current = self.current.as_ref()?;
        let right = current.split_once("=").map(|(_, c)| c).unwrap_or(current);

        let temp = right.split_once(";").map(|(c, _)| c).unwrap_or(right);

        Some(temp.to_string())
    }
    fn jump(&self) -> Option<String> {
        match self.current.as_ref()?.split_once(";") {
            Some((_, jump)) => Some(jump.to_string()),
            _ => None,
        }
    }
}

struct CodeBinary;

impl CodeBinary {
    fn comp_to_binary(comp: &str) -> (u32, u32, u32, u32, u32, u32, u32) {
        let mut output: (u32, u32, u32, u32, u32, u32, u32) = (0, 0, 0, 0, 0, 0, 0);

        if comp == "1"
            || comp == "!D"
            || comp == "!A"
            || comp == "-D"
            || comp == "-A"
            || comp == "D+1"
            || comp == "A+1"
            || comp == "D-A"
            || comp == "A-D"
            || comp == "D|A"
            || comp == "!M"
            || comp == "-M"
            || comp == "M+1"
            || comp == "D-M"
            || comp == "M-D"
            || comp == "D|M"
        {
            output.6 = 1
        };

        if comp == "1"
            || comp == "0"
            || comp == "-1"
            || comp == "-D"
            || comp == "-A"
            || comp == "D+1"
            || comp == "A+1"
            || comp == "D-1"
            || comp == "A-1"
            || comp == "D+A"
            || comp == "D-A"
            || comp == "A-D"
            || comp == "-M"
            || comp == "M+1"
            || comp == "M-1"
            || comp == "D+M"
            || comp == "D-M"
            || comp == "M-D"
        {
            output.5 = 1
        }
        if comp == "1"
            || comp == "D"
            || comp == "-D"
            || comp == "!D"
            || comp == "D+1"
            || comp == "D-1"
            || comp == "A+1"
            || comp == "A-D"
            || comp == "D|A"
            || comp == "M+1"
            || comp == "M-D"
            || comp == "D|A"
            || comp == "D|M"
        {
            output.4 = 1
        };
        if comp == "0"
            || comp == "1"
            || comp == "-1"
            || comp == "D"
            || comp == "!D"
            || comp == "-D"
            || comp == "D+1"
            || comp == "D-1"
        {
            output.3 = 1
        };
        if comp == "1"
            || comp == "A"
            || comp == "-1"
            || comp == "!A"
            || comp == "-A"
            || comp == "A-1"
            || comp == "D+1"
            || comp == "A+1"
            || comp == "D-A"
            || comp == "D|A"
            || comp == "!M"
            || comp == "M"
            || comp == "-M"
            || comp == "M+1"
            || comp == "M-1"
            || comp == "D-M"
            || comp == "D|A"
            || comp == "D|M"
        {
            output.2 = 1
        };
        if comp == "1"
            || comp == "0"
            || comp == "-1"
            || comp == "!A"
            || comp == "-A"
            || comp == "A"
            || comp == "A-1"
            || comp == "A+1"
            || comp == "!M"
            || comp == "M"
            || comp == "-M"
            || comp == "M+1"
            || comp == "M-1"
        {
            output.1 = 1
        };
        if comp == "M"
            || comp == "!M"
            || comp == "-M"
            || comp == "M+1"
            || comp == "M-1"
            || comp == "D+M"
            || comp == "D-M"
            || comp == "M-D"
            || comp == "D&M"
            || comp == "D|M"
        {
            output.0 = 1
        }

        output
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
    fn dest_to_binary(dest: &str) -> (u32, u32, u32) {
        let mut output: (u32, u32, u32) = (0, 0, 0);
        dest.split("").for_each(|x| match x {
            "M" => output.2 = 1,
            "D" => output.1 = 1,
            "A" => output.0 = 1,
            _ => (),
        });

        output
    }
}

fn main() -> io::Result<()> {
    let mut args = env::args();

    let file_name = args.nth(1).unwrap_or(" ".to_string());
    println!("argument is {}", file_name);
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
    let mut symbol_table: HashMap<_, u32> = predefined_symbols
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

    let mut arguments = env::args();
    arguments.next();

    let file_to_compile = arguments.next().unwrap();
    let mut f = File::open(file_to_compile)?;

    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    let mut parser = Parser::new(&buffer);

    let mut binary_file: Vec<String> = Vec::new();

    let mut rom_address = 0;

    for i in 0..parser.lines.len() {
        if parser.command_type() == CommandType::Lcommand {
            let label = parser.symbol();
            if !symbol_table.contains_key(&label) {
                symbol_table.insert(label.to_string(), rom_address.clone());
            }
        } else {
            rom_address += 1;
            if i == 0 {
                // unnecessary -> change
                rom_address = 0;
            }
        }
        parser.advance();
    }

    let mut parser = Parser::new(&buffer);

    parser.advance();

    let mut ram_address: u32 = 16;
    for _i in 0..parser.lines.len() {
        if parser.command_type() == CommandType::Acommand {
            let label = parser.symbol();
            let num_to_binary: u32;

            if symbol_table.contains_key(&label) {
                num_to_binary = *symbol_table.get(&label).unwrap();
                let binary_str = format!("{:016b}", num_to_binary);
                binary_file.push(binary_str);
            } else {
                if label.clone().parse::<u32>().is_ok() {
                    let label_address = label.parse::<u32>();
                    symbol_table.insert(label.clone(), *label_address.as_ref().unwrap());
                    let binary_str = format!("{:016b}", label_address.unwrap());
                    binary_file.push(binary_str);
                } else {
                    symbol_table.insert(label, ram_address);
                    let binary_str = format!("{:016b}", ram_address);
                    ram_address += 1;

                    binary_file.push(binary_str);
                }
            }
        } else if parser.command_type() == CommandType::Ccommand {
            let comp = parser.comp().unwrap_or(" ".to_string());
            let dest = parser.dest().unwrap_or(" ".to_string());
            let jump = parser.jump().unwrap_or(" ".to_string());

            let comp_bits = CodeBinary::comp_to_binary(&comp);

            let dest_bits = CodeBinary::dest_to_binary(&dest);
            let jump_bits = CodeBinary::jump_to_binary(&jump);

            let mut s = String::from("111");
            s.push_str(&comp_bits.0.to_string());
            s.push_str(&comp_bits.1.to_string());
            s.push_str(&comp_bits.2.to_string());
            s.push_str(&comp_bits.3.to_string());
            s.push_str(&comp_bits.4.to_string());
            s.push_str(&comp_bits.5.to_string());
            s.push_str(&comp_bits.6.to_string());
            s.push_str(&dest_bits.0.to_string());
            s.push_str(&dest_bits.1.to_string());
            s.push_str(&dest_bits.2.to_string());
            s.push_str(&jump_bits.0.to_string());
            s.push_str(&jump_bits.1.to_string());
            s.push_str(&jump_bits.2.to_string());
            binary_file.push(s);
        }

        parser.advance();
    }

    let mut file_to_create = file_name.strip_suffix(".asm").unwrap().to_string();
    file_to_create.push_str(".hack");

    let mut f = File::create_new(file_to_create)?;
    for line in binary_file {
        writeln!(f, "{}", line)?;
    }

    Ok(())
}
