#[derive(PartialEq, Debug)]
pub enum VMCOMMAND {
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

pub struct Parser {
    pub lines: Vec<String>,
    pub pos: usize,
    pub current: Option<String>,
}

impl Parser {
    pub fn new(file: &str) -> Self {
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
    pub fn has_more_commands(&self) -> bool {
        self.pos < self.lines.len()
    }

    pub fn advance(&mut self) {
        if self.has_more_commands() {
            let temp = self.lines[self.pos].clone();
            self.current = Some(temp);
            self.pos += 1;
        } else {
            self.current = None;
        }
    }

    pub fn command_type(&self) -> VMCOMMAND {
        match &self.current {
            Some(c) if c.contains("pop") => VMCOMMAND::CPop,
            Some(c) if c.contains("push") => VMCOMMAND::CPush,
            Some(c)
                if ["add", "sub", "lt", "eq", "gt", "and", "or", "not", "neg"]
                    .iter()
                    .any(|&x| c.split_whitespace().next().unwrap() == x) =>
            // come up with something better
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

    pub fn arg_one(&self) -> Option<&str> {
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
    pub fn arg_two(&self) -> Option<&str> {
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
