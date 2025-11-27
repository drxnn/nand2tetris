#![allow(unused, dead_code, non_camel_case_types)]
use regex::Regex;
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

#[derive(PartialEq)]
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
    tokens: Vec<String>,
    current_token: Option<&'a str>,
    pos: usize,
}

impl<'a> jack_tokenizer<'a> {
    fn new(file: &'a str) -> Self {
        // sanitize lines

        let mut tokens: Vec<String> = Vec::new();

        let re = Regex::new(r#""[^"]*"|\S+"#).unwrap(); // need to fix regex to include ; as as symbol and also ( / ) as separate symbols

        // regex is:  this means inside the "(dont include " here)" meaning everything else other than that | S = any nonwhitespace char
        // ?<! is negative lookbehind and exludes ; at the end

        for line in file.lines() {
            let sanitized_line = line.split("//").next().unwrap_or("").trim();

            println!("line is {}", sanitized_line);
            if !sanitized_line.is_empty() {
                re.find_iter(sanitized_line).for_each(|x| {
                    let mut token = x.as_str().to_string();

                    if token.ends_with(";") {
                        token.pop();
                    }
                    println!("current token is : {}", token);
                    tokens.push(token);
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

    fn advance(&'a mut self) {
        if self.has_more_tokens() {
            self.current_token = Some(self.tokens[self.pos].as_ref());
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

    fn keyword(&self) -> Option<&str> {
        match self.token_type() {
            TOKEN_TYPE::KEYWORD => self.current_token,
            _ => None,
        }
    }

    fn symbol(&self) -> Option<&str> {
        match self.token_type() {
            TOKEN_TYPE::SYMBOL => self.current_token,
            _ => None,
        }
    }
    fn identifier(&self) -> Option<&str> {
        match self.token_type() {
            TOKEN_TYPE::IDENTIFIER => self.current_token,
            _ => None,
        }
    }
    fn int_val(&self) -> Option<u32> {
        match self.token_type() {
            TOKEN_TYPE::INT_CONST => self.current_token.and_then(|x| x.parse::<u32>().ok()),
            _ => None,
        }
    }
    fn string_val(&self) -> Option<&str> {
        match self.token_type() {
            TOKEN_TYPE::STRING_CONST => self.current_token,
            _ => None,
        }
    }
}

struct compilation_engine {}
