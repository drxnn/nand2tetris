fn main() {

    // a parser module
    // codewriter module
}
// goes through vm commands and generates assembly code

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
        todo!()
    }

    fn arg_one(&self) -> &str {
        todo!()
    }
    fn arg_two(&self) -> i16 {
        todo!()
    }
}

// arithmetic commands:

fn add(a: i16, b: i16) -> i16 {
    a + b
}
fn sub(a: i16, b: i16) -> i16 {
    a - b
}

fn neg(x: i16) -> i16 {
    -x
}

fn eq(a: i16, b: i16) -> i16 {
    if a == b { -1 } else { 0 }
}

fn gt(a: i16, b: i16) -> i16 {
    if a > b { -1 } else { 0 }
}
fn lt(a: i16, b: i16) -> i16 {
    if a < b { -1 } else { 0 }
}

fn and(a: i16, b: i16) -> i16 {
    a & b
}

fn or(a: i16, b: i16) -> i16 {
    a | b
}

fn not(x: i16) -> i16 {
    !x
}
