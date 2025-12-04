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
        // move stuff around later
        let c_engine = compilation_engine::new(jack_t.tokens);
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

#[derive(PartialEq, Clone, Debug)]
enum TOKEN_TYPE {
    KEYWORD,
    SYMBOL,
    IDENTIFIER,
    INT_CONST,
    STRING_CONST,
    NONE, // REMOVE NONE LATER !!!
}

impl TOKEN_TYPE {
    fn as_str(&self) -> &str {
        match self {
            TOKEN_TYPE::KEYWORD => "keyword",
            TOKEN_TYPE::SYMBOL => "symbol",
            TOKEN_TYPE::IDENTIFIER => "identifier",
            TOKEN_TYPE::INT_CONST => "integerConstant",
            TOKEN_TYPE::STRING_CONST => "stringConstant",
            _ => "",
        }
    }
}

#[derive(Clone)]
struct Token {
    value: String,
    kind: TOKEN_TYPE,
}
struct jack_analyzer {}

struct jack_tokenizer {
    tokens: Vec<Token>,
    current_token: Option<String>,
    pos: usize,
    tokens_file: Option<File>,
}

impl jack_tokenizer {
    fn new(file: &str) -> Self {
        // sanitize lines

        let mut tokens: Vec<Token> = Vec::new();

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
                    let mut token_kind = j_tokenizer.token_type();

                    j_tokenizer.tokens.push(Token {
                        value: token,
                        kind: token_kind,
                    });
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
        println!("token is {}", token);

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
            self.current_token = Some(self.tokens[self.pos].value.clone());
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
            "=", "~", "&lt;", "sgt;", "squot;", "samp;",
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
                "<" => Some("&lt;".to_string()),
                ">" => Some("sgt;".to_string()),
                "\"" => Some("squot;".to_string()),
                "&" => Some("samp;".to_string()),
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
    tokens: Vec<Token>,
    pos: usize,
}

/*Six nonterminal grammar elements ( subroutineCall, subroutineName, varName, className, type, statement )
have no corresponding compilexxx methods;
these terminals are parsed directly, by other parsing methods that handle them; */
impl compilation_engine {
    fn new(tokens: Vec<Token>) -> io::Result<Self> {
        // move tokens from tokenizer to compilation engine
        let file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open("output.txt")
            .unwrap();

        let mut writer = Self {
            file: Some(BufWriter::new(file)),
            tokens: tokens,
            pos: 0,
        };
        Ok(writer)
    }

    fn has_more_tokens(&self) -> bool {
        self.pos < self.tokens.len()
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos >= self.tokens.len() {
            return None;
        }
        let idx = self.pos;
        self.pos += 1;
        Some(self.tokens[idx].clone())
    }
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn write_open_tag(&mut self, s: &str) -> io::Result<()> {
        let s = format!("<{s}>");
        if let Some(f) = self.file.as_mut() {
            f.write_all(s.as_bytes())?;
        }
        Ok(())
    }
    fn write_close_tag(&mut self, s: &str) -> io::Result<()> {
        let s = format!("</{s}>");
        if let Some(f) = self.file.as_mut() {
            f.write_all(s.as_bytes())?;
        };

        Ok(())
    }
    fn write_token(&mut self, token: &str, tag: &str) -> io::Result<()> {
        let s = format!("<{tag}> {token} </{tag}>");
        if let Some(f) = self.file.as_mut() {
            f.write_all(s.as_bytes())?;
        }
        Ok(())
    }

    fn compile_class(&mut self) -> io::Result<()> {
        //class:          class'className '{'classVarDec* subroutineDec* '}'
        //classVarDec:    (‘static'|'field') type varName (',' varName)* ';'
        // only call compile_class if curr_token is class

        self.write_open_tag("class");
        let class_tok = self.advance().expect("expected 'class' token");

        self.write_token(&class_tok.value, class_tok.kind.as_str());
        let class_identifier = self.advance().expect("expected 'class' name "); // word after class, then moves pos
        self.write_token(&class_identifier.value, class_identifier.kind.as_str());
        let open_bracket = self.advance().expect("expected '{' bracket ");
        self.write_token(&open_bracket.value, open_bracket.kind.as_str());

        // peek might panic, will fix later
        while let Some(tok) = self.peek() {
            if tok.value == "static" || tok.value == "field" {
                self.write_open_tag("classVarDec")?;
                loop {
                    match self.advance() {
                        Some(t) => {
                            self.write_token(&t.value, t.kind.as_str())?;
                            if t.value == ";" {
                                break;
                            }
                        }
                        None => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::UnexpectedEof,
                                "unexpected eof error while parsing classVarDec",
                            ));
                        }
                    }
                }
                self.write_close_tag("classVarDec")?;
            } else if tok.value == "constructor" || tok.value == "function" || tok.value == "method"
            {
                self.compile_subroutine();
            }
        }

        self.write_close_tag("class");

        Ok(())
    }
    fn compile_class_var_dec() {
        // put code thats in compile_class here later
        unimplemented!()
    }
    fn compile_subroutine(&mut self) -> io::Result<()> {
        self.write_open_tag("subroutineDec")?;
        let kw_1 = self
            .advance()
            .expect("expected kw_1, -> from subroutine method");
        self.write_token(&kw_1.value, kw_1.kind.as_str())?;

        // do

        self.write_close_tag("subroutineDec")?;
        Ok(())
    }
    fn compile_parameter_list(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_var_dec(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_statements(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_do(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_let(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_while(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_return(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_if(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_expression(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_term(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn compile_expression_list(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}
