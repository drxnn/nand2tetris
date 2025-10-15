use std::i16;

fn main() {

    // a parser module
    // codewriter module
}
// goes through vm commands and generates assembly code
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

    fn arg_one(&self) -> Option<&String> {
        if self.command_type() == VMCOMMAND::CArithmetic {
            self.current.as_ref()
        } else {
            None
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

struct CodeWriter;
