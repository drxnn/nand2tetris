#![allow(unused, dead_code, non_camel_case_types)]
use std::io::{BufWriter, Write};
use std::path::Path;
use std::path::PathBuf;
use std::{env, io};
use std::{file, fs};

fn main() -> io::Result<()> {
    println!("Hello, world!");
    let mut args = env::args();
    args.next();

    let input_name = args.next().expect("Please provide a filename as argument");
    let input_path = PathBuf::from(input_name.clone());
    let buffer = fs::read_to_string(input_name)?;
    if input_path.is_file() {
        let jack_t = jack_tokenizer::new(&buffer);
    }

    Ok(())
    /*
    jack_analyzer
    jack_tokenizer
    compilation_engine





    The analyzer will:
    for each source file:
    create a jack_tokenizer from the Xxx.jack input file;
    create an output file called Xxx.xml and prepare it for writing
    Use the compilation_engine to compiler the input jack_tokenizer into the output file

     */
}

enum TOKEN_TYPE {
    KEYWORD,
    SYMBOL,
    IDENTIFIER,
    INT_CONST,
    STRING_CONST,
    NONE,
}
struct jack_analyzer {}

struct jack_tokenizer<'a> {
    tokens: Vec<&'a str>,
    current_token: Option<&'a str>,
    pos: usize,
}

impl<'a> jack_tokenizer<'a> {
    fn new(file: &'a str) -> Self {
        // sanitize lines

        let mut tokens = Vec::new();
        for line in file.lines() {
            let sanitized_line = line.split("//").next().unwrap_or("").trim();
            println!("line is {}", sanitized_line);
            if !sanitized_line.is_empty() {
                sanitized_line.split(" ").for_each(|x| {
                    println!("x is {}", x);
                    tokens.push(x);
                });
            }
        }

        Self {
            tokens,
            current_token: None,
            pos: 0,
        }
    }

    fn has_more_tokens(&self) -> bool {
        self.pos < self.tokens.len()
    }

    fn advance(&mut self) {
        if self.has_more_tokens() {
            self.current_token = Some(self.tokens[self.pos]);
            self.pos += 1;
        } else {
            self.current_token = None
        }
    }

    fn token_type(&self) -> TOKEN_TYPE {
        // returns one of enum
        // figure out
        const KEYWORDS: [&str; 21] = [
            "class",
            "constructor",
            "function",
            "method",
            "field",
            "static",
            "var",
            "int",
            "char",
            "boolean",
            "void",
            "true",
            "false",
            "null",
            "this",
            "let",
            "do",
            "if",
            "else",
            "while",
            "return",
        ];
        const SYMBOLS: [&str; 19] = [
            "{", "}", "(", ")", "[", "]", ".", ",", ";", "+", "-", "*", "/", "&", "|", "<", ">",
            "=", "~",
        ];

        match self.current_token {
            Some(x) if KEYWORDS.contains(&x) => TOKEN_TYPE::KEYWORD,
            Some(x) if x.len() == 1 && SYMBOLS.contains(&x) => TOKEN_TYPE::SYMBOL,
            Some(x) if x.starts_with('"') && x.ends_with('"') && x.len() >= 2 => {
                TOKEN_TYPE::STRING_CONST
            }
            Some(x) if x.parse::<u32>().is_ok() => TOKEN_TYPE::INT_CONST,
            Some(x)
                if {
                    let mut chars = x.chars();
                    match chars.next() {
                        Some(c) if c.is_alphabetic() || c == '_' => {
                            chars.all(|ch| ch.is_alphanumeric() || ch == '_')
                        }
                        _ => false,
                    }
                } =>
            {
                TOKEN_TYPE::IDENTIFIER
            }

            _ => TOKEN_TYPE::NONE,
        }
    }
}

struct compilation_engine {}
