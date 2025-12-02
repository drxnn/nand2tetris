#![allow(unused, dead_code, non_camel_case_types)]
use regex::Regex;

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::{env, io};

fn main() -> io::Result<()> {
    println!("Hello, world!");
    let mut args = env::args();
    args.next();

    let input_name = args.next().expect("Please provide a filename as argument");
    let input_path = PathBuf::from(input_name.clone());
    let buffer = fs::read_to_string(input_name)?;
    if input_path.is_file() {
        let mut jack_t = jack_tokenizer::new(&buffer);
        while jack_t.has_more_tokens() {
            jack_t.advance();
            jack_t.write_token_file();
        }
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

struct jack_tokenizer {
    tokens: Vec<String>,
    current_token: Option<String>,
    pos: usize,
    tokens_file: Option<File>,
}

impl jack_tokenizer {
    fn new(file: &str) -> Self {
        // sanitize lines

        let mut tokens: Vec<String> = Vec::new();

        let mut file_tokens = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open("tokens.xml")
            .unwrap();

        let mut j_tokenizer = Self {
            tokens,
            current_token: None,
            pos: 0,
            tokens_file: Some(file_tokens),
        };
        let re = Regex::new(r#""[^"]*"|\w+|[^\w\s]"#).unwrap();

        for line in file.lines() {
            let sanitized_line = line.split("//").next().unwrap_or("").trim();

            println!("line is {}", sanitized_line);
            if !sanitized_line.is_empty() {
                re.find_iter(sanitized_line).for_each(|x| {
                    let mut token = x.as_str().to_string();

                    j_tokenizer.tokens.push(token);
                });
            }
        }

        j_tokenizer
    }

    fn write_token_file(&mut self) {
        if self.token_type() == TOKEN_TYPE::SYMBOL {
            self.symbol();
        }

        let token_type = self.token_type_as_str();
        println!("token type is: {}", token_type);

        let token = self.current_token.as_ref().unwrap();

        let write_token = format!("<{token_type}> {token} </{token_type}>\n");
        self.tokens_file
            .as_mut()
            .unwrap()
            .write_all(write_token.as_bytes());
    }

    fn has_more_tokens(&self) -> bool {
        self.pos < self.tokens.len()
    }

    fn advance(&mut self) {
        if self.has_more_tokens() {
            self.current_token = Some(self.tokens[self.pos].clone());
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
        const SYMBOLS: [&str; 23] = [
            "{", "}", "(", ")", "[", "]", ".", ",", ";", "+", "-", "*", "/", "&", "|", "<", ">",
            "=", "~", "&1t;", "sgt;", "squot;", "samp;",
        ];

        match &self.current_token {
            Some(x) if KEYWORDS.contains(&x.as_ref()) => TOKEN_TYPE::KEYWORD,
            Some(x) if SYMBOLS.contains(&x.as_ref()) => TOKEN_TYPE::SYMBOL,
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

    fn token_type_as_str(&self) -> &str {
        match self.token_type() {
            TOKEN_TYPE::IDENTIFIER => "identifier",
            TOKEN_TYPE::KEYWORD => "keyword",
            TOKEN_TYPE::STRING_CONST => "stringConstant",
            TOKEN_TYPE::INT_CONST => "integerConstant",
            TOKEN_TYPE::SYMBOL => "symbol",
            TOKEN_TYPE::NONE => "",
        }
    }

    fn keyword(&self) -> Option<String> {
        match self.token_type() {
            TOKEN_TYPE::KEYWORD => self.current_token.clone(),
            _ => None,
        }
    }

    fn symbol(&mut self) -> Option<String> {
        /*
                                                                                                                        four of the symbols used in the Jack language (<, >, ", «) are also used for XML markup,
and thus they cannot appear as data in XML files. 
To solve the problem, we require the tokenizer to output these tokens as &1t;, sgt;, squot;, and samp;, respectively. */
        if let TOKEN_TYPE::SYMBOL = self.token_type() {
            let new_val = match self.current_token.as_ref().unwrap().as_ref() {
                "<" => Some("&1t;".to_string()),
                ">" => Some("sgt;".to_string()),
                "\"" => Some("squot;".to_string()),
                "*" => Some("samp;".to_string()),
                _ => self.current_token.clone(),
            };
            self.current_token = new_val.clone();

            new_val
        } else {
            None
        }
    }
    fn identifier(&self) -> Option<String> {
        match self.token_type() {
            TOKEN_TYPE::IDENTIFIER => self.current_token.clone(),
            _ => None,
        }
    }
    fn int_val(&self) -> Option<u32> {
        match self.token_type() {
            TOKEN_TYPE::INT_CONST => self
                .current_token
                .clone()
                .and_then(|x| x.parse::<u32>().ok()),
            _ => None,
        }
    }
    fn string_val(&self) -> Option<String> {
        match self.token_type() {
            TOKEN_TYPE::STRING_CONST => self.current_token.clone(),
            _ => None,
        }
    }
}

/*
CompilationEngine: Effects the actual compilation output. Gets its input from a JackTokenizer and
emits its parsed structure into an output file/stream. The output is generated by a series of compilexxx () routines,
one for every syntactic element xxx of the Jack grammar. The contract between these routines is that each compilexxx ()
routine should read the syntactic construct xxx from the input, advance () the tokenizer exactly beyond xxx, and output the parsing of xxx.
Thus, compilexxx () may only be called if indeed xxx is the next syntactic element of the input.
 */

struct compilation_engine {
    file: Option<BufWriter<File>>,
}

/*Six nonterminal grammar elements ( subroutineCall, subroutineName, varName, className, type, statement )
have no corresponding compilexxx methods;
these terminals are parsed directly, by other parsing methods that handle them; */
impl compilation_engine {
    fn new() -> io::Result<Self> {
        let file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open("output.txt")
            .unwrap();

        let mut writer = Self {
            file: Some(BufWriter::new(file)),
        };
        Ok(writer)
    }

    fn process(str: &str) {
        // helper function
        unimplemented!()
    }

    fn compile_class() {
        unimplemented!()
    }
    fn compile_class_var_dec() {
        unimplemented!()
    }
    fn compile_subroutine() {
        unimplemented!()
    }
    fn compiler_parameter_list() {
        unimplemented!()
    }
    fn compile_var_dec() {
        unimplemented!()
    }
    fn compile_statements() {
        unimplemented!()
    }
    fn compile_do() {
        unimplemented!()
    }
    fn compile_let() {
        unimplemented!()
    }
    fn compile_while() {
        unimplemented!()
    }
    fn compile_return() {
        unimplemented!()
    }
    fn compile_if() {
        unimplemented!()
    }
    fn compile_expression() {
        unimplemented!()
    }
    fn compile_term() {
        unimplemented!()
    }
    fn compile_expression_list() {
        unimplemented!()
    }
}
