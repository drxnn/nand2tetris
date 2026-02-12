#![allow(dead_code, non_snake_case)]
use regex::Regex;

use std::ffi::OsStr;
use std::fmt::format;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};

use std::path::{Path, PathBuf};

use std::{env, io, usize};

fn main() -> io::Result<()> {
    let mut args = env::args();
    args.next();

    // if filename -> run compile_engine
    // if folder -> run compilation_engine per jack file and output fileName.vm
    let input_name = args
        .next()
        .expect("Please provide a file or folder as an argument");

    let p = Path::new(&input_name);

    if p.is_dir() {
        for entry in std::fs::read_dir(&input_name)? {
            let entry = entry?;
            if let Some(ext) = entry.path().extension() {
                if ext == "jack" {
                    compile(entry.path())?;
                }
            }
        }
    } else {
        compile(p.to_path_buf())?; // single f
    }
    let input_path = PathBuf::from(input_name.clone());
    let buffer = fs::read_to_string(input_name)?;
    if input_path.is_file() {
        let mut jack_t = jack_tokenizer::new(&buffer);

        while jack_t.has_more_tokens() {
            jack_t.advance();
            jack_t.write_token_file();
        }

        // currently only does 1 file
    }

    Ok(())
}
fn compile(f_path: PathBuf) -> io::Result<()> {
    let buffer = fs::read_to_string(&f_path)?;
    // f_path is file.jack -> just pass file to vm_out_file
    if f_path.is_file() {
        let mut jack_t = jack_tokenizer::new(&buffer);

        while jack_t.has_more_tokens() {
            jack_t.advance();
            jack_t.write_token_file();
        }
        let vm_output_file: &OsStr = f_path.file_stem().unwrap(); // add .vm to it
        fs::create_dir_all("./output")?;
        let final_path = format!("./output/{}.vm", vm_output_file.to_str().unwrap());
        let path = Path::new(&final_path);
        println!("path is {:?}", path);
        // currently only does 1 file
        let mut c_engine = compilation_engine::new(jack_t.tokens, path).unwrap();

        c_engine.compile_class()?;
    }

    Ok(())
}

#[derive(PartialEq, Clone, Debug)]
enum TOKEN_TYPE {
    KEYWORD,
    SYMBOL,
    IDENTIFIER,
    INT_CONST,
    STRING_CONST,
}

impl TOKEN_TYPE {
    fn as_str(&self) -> &str {
        match self {
            TOKEN_TYPE::KEYWORD => "keyword",
            TOKEN_TYPE::SYMBOL => "symbol",
            TOKEN_TYPE::IDENTIFIER => "identifier",
            TOKEN_TYPE::INT_CONST => "integerConstant",
            TOKEN_TYPE::STRING_CONST => "stringConstant",
        }
    }
}

#[derive(Clone)]
struct Token {
    value: String,
    kind: TOKEN_TYPE,
}

// enum Identifier_Type{

// }
// #[derive(Clone)]
struct IdentifierEntry {
    name: String,
    type_name: String,
    kind: Identifier_Kind,
    index: usize,
}

enum Identifier_Kind {
    STATIC, // scope class
    FIELD,  // scope class
    ARG,    // scope subroutine
    VAR,    // scope subroutine
}

impl Identifier_Kind {
    fn kind_to_segment(&self) -> &'static str {
        match self {
            Identifier_Kind::STATIC => "static",
            Identifier_Kind::FIELD => "this",
            Identifier_Kind::ARG => "argument",
            Identifier_Kind::VAR => "local",
        }
    }
}
struct symbol_table {
    class_scope: Vec<IdentifierEntry>, // name, type, kind/segment, index
    subroutine_scope: Vec<IdentifierEntry>,
    static_index: usize,
    field_index: usize,
    arg_index: usize,
    var_index: usize,
}

struct VM_Writer {
    file: Option<BufWriter<File>>,
}
impl VM_Writer {
    fn new(path: &Path) -> io::Result<Self> {
        println!("path is in vm writer:{:?}", path);
        let file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(path)?;
        println!("path has passed file creation :{:?}", path);

        Ok(Self {
            file: Some(BufWriter::new(file)),
        })
    }
    fn write_push(&mut self, segment: &str, index: usize) -> io::Result<()> {
        let vm_to_write = format!("push {} {}\n", segment, index);
        self.write_to_file(vm_to_write)?;

        Ok(())
    }
    fn write_pop(&mut self, segment: &str, index: usize) -> io::Result<()> {
        let vm_to_write = format!("pop {} {}\n", segment, index);
        self.write_to_file(vm_to_write)?;

        Ok(())
    }
    fn write_arithmetic(&mut self, command: &str) -> io::Result<()> {
        let vm_to_write = format!("{}\n", command);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_label(&mut self, label: &str) -> io::Result<()> {
        let vm_to_write = format!("label {}\n", label);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_goto(&mut self, label: &str) -> io::Result<()> {
        let vm_to_write = format!("goto {}\n", label);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_if(&mut self, label: &str) -> io::Result<()> {
        let vm_to_write = format!("if-goto {}\n", label);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_call(&mut self, name: &str, n_args: usize) -> io::Result<()> {
        let vm_to_write = format!("call {} {}\n", name, n_args);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_function(&mut self, name: &str, n_locals: usize) -> io::Result<()> {
        let vm_to_write = format!("function {} {}\n", name, n_locals);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_return(&mut self, with_expression: bool) -> io::Result<()> {
        let vm_to_write = if with_expression == true {
            format!("return\n")
        } else {
            // push dummy value
            format!("push constant 0\n return\n")
        };
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_to_file(&mut self, str_to_write: String) -> io::Result<()> {
        if let Some(f) = self.file.as_mut() {
            f.write_all(str_to_write.as_bytes())?;
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No Output File"));
        }
        Ok(())
    }
    fn close(&mut self) -> io::Result<()> {
        if let Some(f) = self.file.as_mut() {
            f.flush()?;
        }
        Ok(())
    }
}
enum SEGMENTS {
    STATIC,
    THIS,
    ARG,
    LOCAL,
}
impl symbol_table {
    fn new() -> Self {
        Self {
            class_scope: Vec::new(),
            subroutine_scope: Vec::new(),
            static_index: 0,
            field_index: 0,
            arg_index: 0,
            var_index: 0,
        }
    }
    fn start_subroutine(&mut self) {
        // Starts a new subroutine scope = erases all names in the previous subroutine’s scope.
        self.subroutine_scope.clear();
        self.arg_index = 0;
        self.var_index = 0;
    }
    fn define(&mut self, name: &str, ty: &str, kind: &str) {
        // Defines a new identifier of a given name, type and kind and assigns it a running index. STATIC and FIELD identifiers
        // have a class scope, while ARG and VAR identifiers have a subroutine scope.
        let index = self.var_count(&kind);
        let identifier_kind = match kind {
            "static" => Identifier_Kind::STATIC,
            "field" => Identifier_Kind::FIELD,
            "var" => Identifier_Kind::VAR,
            "argument" => Identifier_Kind::ARG,
            _ => panic!("Invalid identifier kind: {}", kind),
        };
        match identifier_kind {
            Identifier_Kind::ARG => {
                self.subroutine_scope.push(IdentifierEntry {
                    name: name.to_string(),
                    type_name: ty.to_string(),
                    kind: identifier_kind,
                    index,
                });
                self.arg_index += 1;
            }
            Identifier_Kind::VAR => {
                self.subroutine_scope.push(IdentifierEntry {
                    name: name.to_string(),
                    type_name: ty.to_string(),
                    kind: identifier_kind,
                    index,
                });
                self.var_index += 1
            }
            Identifier_Kind::STATIC => {
                self.class_scope.push(IdentifierEntry {
                    name: name.to_string(),
                    type_name: ty.to_string(),
                    kind: identifier_kind,
                    index,
                });
                self.static_index += 1
            }
            Identifier_Kind::FIELD => {
                self.class_scope.push(IdentifierEntry {
                    name: name.to_string(),
                    type_name: ty.to_string(),
                    kind: identifier_kind,
                    index,
                });
                self.field_index += 1
            }
        }
    }

    fn var_count(&self, kind: &str) -> usize {
        // Returns the number of variables of the given kind already defined in the current scope.
        let identifier_kind = match kind {
            "static" => Identifier_Kind::STATIC,
            "field" => Identifier_Kind::FIELD,
            "var" => Identifier_Kind::VAR,
            "argument" => Identifier_Kind::ARG,
            _ => panic!("Invalid identifier kind: {}", kind),
        };
        match identifier_kind {
            Identifier_Kind::ARG => self.arg_index,
            Identifier_Kind::VAR => self.var_index,
            Identifier_Kind::FIELD => self.field_index,
            Identifier_Kind::STATIC => self.static_index,
        }
    }
    fn kind_of(&self, name: &str) -> Option<&Identifier_Kind> {
        // Returns the kind of the named identifier in the current scope. If the identifier is unknown in the current scope, returns NONE.
        // check if its in subroutine first
        self.subroutine_scope
            .iter()
            .find(|x| x.name == name)
            .and_then(|x| Some(&x.kind))
            .or_else(|| {
                self.class_scope
                    .iter()
                    .find(|x| x.name == name)
                    .and_then(|x| Some(&x.kind))
            })
    }
    fn type_of(&self, name: &str) -> Option<&String> {
        self.subroutine_scope
            .iter()
            .find(|x| x.name == name)
            .and_then(|x| Some(&x.type_name))
            .or_else(|| {
                self.class_scope
                    .iter()
                    .find(|x| x.name == name)
                    .and_then(|x| Some(&x.type_name))
            })
    }
    fn index_of(&self, name: &str) -> Option<usize> {
        self.subroutine_scope
            .iter()
            .find(|x| x.name == name)
            .and_then(|x| Some(x.index))
            .or_else(|| {
                self.class_scope
                    .iter()
                    .find(|x| x.name == name)
                    .and_then(|x| Some(x.index))
            })
        // Returns the index assigned to the named identifier.
    }
    fn clear_subroutine_scope(&mut self) {
        self.subroutine_scope.clear();
    }
}
struct jack_tokenizer {
    tokens: Vec<Token>,
    current_token: Option<String>,
    pos: usize,
    tokens_file: Option<File>,
}

impl jack_tokenizer {
    fn new(file: &str) -> Self {
        let tokens: Vec<Token> = Vec::new();

        let file_tokens = OpenOptions::new()
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
            // need to add sanitization for /* block level comments */
            let sanitized_line = line.split("//").next().unwrap_or("").trim();

            if !sanitized_line.is_empty() {
                re.find_iter(sanitized_line).for_each(|x| {
                    let token = x.as_str().to_string();

                    j_tokenizer.current_token = Some(token.clone());
                    let token_kind = j_tokenizer.token_type();
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
        let token_type = self.token_type_as_str();

        let token = self.current_token.as_ref().unwrap();

        let write_token = format!("<{token_type}> {token} </{token_type}>\n");
        let _ = self
            .tokens_file
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

            _ => TOKEN_TYPE::IDENTIFIER,
        }
    }

    fn token_type_as_str(&self) -> &str {
        match self.token_type() {
            TOKEN_TYPE::IDENTIFIER => "identifier",
            TOKEN_TYPE::KEYWORD => "keyword",
            TOKEN_TYPE::STRING_CONST => "stringConstant",
            TOKEN_TYPE::INT_CONST => "integerConstant",
            TOKEN_TYPE::SYMBOL => "symbol",
        }
    }
}

struct compilation_engine {
    file: Option<BufWriter<File>>,
    tokens: Vec<Token>,
    pos: usize,
    indentation: usize,
    symbol_table: symbol_table,

    vm_writer: VM_Writer,
    label_index: usize,
    curr_class: Option<String>,
}

impl compilation_engine {
    fn new(tokens: Vec<Token>, path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open("output.xml")
            .unwrap();

        let symbol_table = symbol_table::new();
        println!("path is here {:?}", path);
        let vm_writer = VM_Writer::new(path)?; // will fix later 
        println!("path is passing vm writer {:?}", path);
        let writer = Self {
            symbol_table,
            file: Some(BufWriter::new(file)),
            tokens: tokens,
            pos: 0,
            indentation: 0,
            vm_writer,
            label_index: 0,
            curr_class: None,
        };
        Ok(writer)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos >= self.tokens.len() {
            return None;
        }
        let idx = self.pos;
        self.pos += 1;
        Some(self.tokens[idx].clone())
    }

    fn expect_kind(&mut self, expected_kind: &str) -> io::Result<Token> {
        let token_to_check = self.advance().ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, format!("token missing, eof"))
        })?;

        if token_to_check.kind.as_str() == expected_kind {
            Ok(token_to_check)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unexpected token kind: '{}' (value: '{}'), expected '{}'",
                    token_to_check.kind.as_str(),
                    token_to_check.value,
                    expected_kind
                ),
            ))
        }
    }

    fn expect_value(&mut self, expected_value: &str) -> io::Result<Token> {
        let token_to_check = self.advance().ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, format!("token missing, eof"))
        })?;

        if token_to_check.value == expected_value {
            Ok(token_to_check)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unexpected value: {}, expected: {}",
                    token_to_check.value, expected_value
                ),
            ))
        }
    }
    fn expect_type(&mut self) -> io::Result<Token> {
        let tok = self.advance().ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, format!("token missing, eof"))
        })?;

        match tok.kind {
            TOKEN_TYPE::KEYWORD => {
                if tok.value == "int"
                    || tok.value == "char"
                    || tok.value == "boolean"
                    || tok.value == "void"
                {
                    Ok(tok)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unexpected keyword {}; expected type", tok.value),
                    ))
                }
            }
            TOKEN_TYPE::IDENTIFIER => Ok(tok),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected keyword {}; expected type", tok.value),
            )),
        }
    }

    fn is_operator(&mut self) -> bool {
        let operators: [&str; 9] = ["+", "-", "*", "/", "&", "|", "<", ">", "="];
        self.peek()
            .is_some_and(|x| operators.contains(&x.value.as_str()))
    }
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn peek_ahead(&self) -> Option<Token> {
        self.tokens.get(self.pos + 1).cloned()
    }

    fn write_open_tag(&mut self, s: &str) -> io::Result<()> {
        let indentation = " ".repeat(self.indentation);
        let s = format!("{indentation}<{s}>\n");
        if let Some(f) = self.file.as_mut() {
            f.write_all(s.as_bytes())?;
            f.flush()?;
        }
        Ok(())
    }
    fn write_close_tag(&mut self, s: &str) -> io::Result<()> {
        let indentation = " ".repeat(self.indentation);
        let s = format!("{indentation}</{s}>\n");

        if let Some(f) = self.file.as_mut() {
            f.write_all(s.as_bytes())?;
            f.flush()?;
        };

        Ok(())
    }
    fn write_token(&mut self, token: &str, tag: &str) -> io::Result<()> {
        let indentation = " ".repeat(self.indentation);
        let s = format!("{indentation}<{tag}> {token} </{tag}>\n");
        if let Some(f) = self.file.as_mut() {
            f.write_all(s.as_bytes())?;
            f.flush()?;
        }
        Ok(())
    }

    fn normalize_symbol(&mut self) {
        if let Some(s) = self.peek() {
            match s.value.as_str() {
                "<" => self.tokens[self.pos].value = "lt".to_string(),
                "=" => self.tokens[self.pos].value = "eq".to_string(),
                ">" => self.tokens[self.pos].value = "gt".to_string(),
                "\"" => self.tokens[self.pos].value = "&quot;".to_string(),
                "&" => self.tokens[self.pos].value = "and".to_string(),
                "+" => self.tokens[self.pos].value = "add".to_string(),
                "*" => self.tokens[self.pos].value = "multiply".to_string(),
                "/" => self.tokens[self.pos].value = "divide".to_string(),
                "-" => self.tokens[self.pos].value = "sub".to_string(),

                _ => (),
            }
        }
    }

    fn compile_class(&mut self) -> io::Result<()> {
        self.write_open_tag("class")?;
        self.indentation += 2;
        let class_tok = self.expect_value("class")?;

        self.write_token(&class_tok.value, class_tok.kind.as_str())?;
        let class_identifier = self.expect_kind("identifier")?; // className
        self.curr_class = Some(class_identifier.value.clone());
        self.write_token(&class_identifier.value, class_identifier.kind.as_str())?;
        let open_bracket = self.expect_value("{")?;
        self.write_token(&open_bracket.value, open_bracket.kind.as_str())?;

        while let Some(tok) = self.peek() {
            if tok.value == "static" || tok.value == "field" {
                self.compile_class_var_dec()?;
            } else if tok.value == "constructor" || tok.value == "function" || tok.value == "method"
            {
                self.compile_subroutine(&class_identifier.value)?;
            } else if tok.value == "}" && tok.kind.as_str() == "symbol" {
                let ending_bracket = self.expect_value("}")?;

                self.write_token(&ending_bracket.value, ending_bracket.kind.as_str())?;
                break;
            }
        }
        self.indentation -= 2;
        self.write_close_tag("class")?;

        Ok(())
    }
    fn compile_class_var_dec(&mut self) -> io::Result<()> {
        self.write_open_tag("classVarDec")?;
        self.indentation += 2;

        let t = self.advance().unwrap();
        let kind = t.value.clone();
        self.write_token(&t.value, t.kind.as_str())?;

        // Second token is the type
        let t = self.advance().unwrap();
        let var_type = t.value.clone();
        self.write_token(&t.value, t.kind.as_str())?;

        loop {
            match self.advance() {
                Some(t) => {
                    if t.value == ";" {
                        self.write_token(&t.value, t.kind.as_str())?;
                        break;
                    } else if t.value != "," {
                        // token is name
                        self.symbol_table.define(&t.value, &var_type, &kind);
                    }
                    self.write_token(&t.value, t.kind.as_str())?;
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "unexpected eof error while parsing classVarDec",
                    ));
                }
            }
        }
        self.indentation -= 2;
        self.write_close_tag("classVarDec")?;
        Ok(())
    }
    fn compile_subroutine(&mut self, className: &str) -> io::Result<()> {
        self.symbol_table.start_subroutine();
        self.write_open_tag("subroutineDec")?;
        self.indentation += 2;
        let subroutine_kind = self.expect_kind("keyword")?;

        self.write_token(&subroutine_kind.value, subroutine_kind.kind.as_str())?;

        let kw_2 = self.expect_type()?;
        self.write_token(&kw_2.value, kw_2.kind.as_str())?;

        let f_name = self.expect_kind("identifier")?; // f_name 
        if subroutine_kind.value == "method" {
            self.symbol_table.define("this", className, "argument");
        }

        self.write_token(&f_name.value, f_name.kind.as_str())?;

        let opening_parenthesis = self
            .advance()
            .expect("expected ( , -> from subroutine method");

        self.write_token(
            &opening_parenthesis.value,
            opening_parenthesis.kind.as_str(),
        )?;

        self.write_open_tag("parameterList")?;
        self.indentation += 2;

        let mut num_of_params: usize = 0;
        if let Some(t) = self.peek() {
            if t.value != ")" {
                self.compile_parameter_list()?;
                num_of_params += 1;
                while let Some(t) = self.peek() {
                    if t.value == "," {
                        let t = self.expect_value(",")?;
                        self.write_token(&t.value, t.kind.as_str())?;
                        self.compile_parameter_list()?;
                        num_of_params += 1;
                    } else {
                        break;
                    }
                }
            }
        }

        self.indentation -= 2;
        self.write_close_tag("parameterList")?;

        let closing_parenthesis = self
            .advance()
            .expect("expected ) , -> from subroutine method");
        self.write_token(
            &closing_parenthesis.value,
            closing_parenthesis.kind.as_str(),
        )?;

        self.write_open_tag("subroutineBody")?;
        self.indentation += 2;
        let opening_bracket = self.expect_value("{")?;
        self.write_token(&opening_bracket.value, opening_bracket.kind.as_str())?;

        let num_of_vars = self.compile_var_dec()?;

        // need to declare function here
        // need to keep count of how many variables are declared in a function
        // function functionName num_of_vars
        let full_f_name = format!("{}.{}", self.curr_class.clone().unwrap(), f_name.value);
        self.vm_writer.write_function(&full_f_name, num_of_vars)?;
        if subroutine_kind.value == "constructor" {
            // push num of args
            // call Memory alloc
            // pop pointer
            // memory alloc allocates amount of vars so get num of fields for class:

            self.vm_writer
                .write_push("constant", self.symbol_table.field_index)?;
            self.vm_writer.write_call("Memory.alloc", 1)?;
            self.vm_writer.write_pop("pointer", 0)?;
        } else if subroutine_kind.value == "method" {
            // arg + 0 holds the object ref, pop pointer 0 will store it in THIS register
            self.vm_writer.write_push("argument", 0)?;
            self.vm_writer.write_pop("pointer", 0)?;
        }

        self.compile_statements()?;

        let closing_bracket = self
            .advance()
            .expect("expected } , -> from subroutine method");
        self.write_token(&closing_bracket.value, closing_bracket.kind.as_str())?;
        self.indentation -= 2;
        self.write_close_tag("subroutineBody")?;

        self.indentation -= 2;
        self.write_close_tag("subroutineDec")?;

        Ok(())
    }
    fn compile_parameter_list(&mut self) -> io::Result<()> {
        // compiles one param but is called in a loop for all of them
        let t_token = self.expect_type()?;
        self.write_token(&t_token.value, t_token.kind.as_str())?;
        let n_token = self.expect_kind("identifier")?;
        self.write_token(&n_token.value, n_token.kind.as_str())?;
        self.symbol_table
            .define(&n_token.value, &t_token.value, "argument");

        Ok(())
    }
    fn compile_var_dec(&mut self) -> io::Result<usize> {
        let mut t_type = String::new();
        let mut n_vars = 0;

        while let Some(tok) = self.peek() {
            if tok.value == "var" {
                self.write_open_tag("varDec")?;
                self.indentation += 2;

                let var_token = self.expect_value("var")?;
                self.write_token(&var_token.value, var_token.kind.as_str())?;
                let type_token = self.expect_type()?;
                t_type = type_token.value.clone();
                self.write_token(&type_token.value, type_token.kind.as_str())?;

                loop {
                    // here its: varName , otherVarName , varName2 , lastVarName ;
                    match self.advance() {
                        Some(t) => {
                            if t.kind == TOKEN_TYPE::IDENTIFIER {
                                self.symbol_table.define(&t.value, t_type.as_str(), "var");
                                n_vars += 1;
                            } else if t.value == ";" {
                                self.write_token(&t.value, t.kind.as_str())?;
                                break;
                            }
                            self.write_token(&t.value, t.kind.as_str())?;
                        }
                        None => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::UnexpectedEof,
                                "unexpected eof error while parsing VarDec",
                            ));
                        }
                    }
                }
                self.indentation -= 2;
                self.write_close_tag("varDec")?;
            } else {
                break;
            }
        }

        Ok(n_vars)
    }
    fn compile_statements(&mut self) -> io::Result<()> {
        self.write_open_tag("statements")?;
        self.indentation += 2;
        while let Some(tok) = self.peek() {
            if ["let", "if", "do", "while", "return"].contains(&tok.value.as_str()) {
                match tok.value.as_str() {
                    "let" => {
                        self.compile_let()?;
                    }
                    "if" => {
                        self.compile_if()?;
                    }
                    "do" => {
                        self.compile_do()?;
                    }
                    "while" => {
                        self.compile_while()?;
                    }
                    "return" => {
                        self.compile_return()?;
                    }
                    _ => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "unexpected token error while parsing statment in compile_statemens()",
                        ));
                    }
                }
            } else {
                break;
            }
        }
        self.indentation -= 2;
        self.write_close_tag("statements")?;
        Ok(())
    }

    fn get_variable_from_scope(
        &self,
        var_to_look_for: &str,
    ) -> Result<&IdentifierEntry, std::io::Error> {
        let variable = self
            .symbol_table
            .subroutine_scope
            .iter()
            .find(|entry| entry.name == var_to_look_for)
            .or_else(|| {
                self.symbol_table
                    .class_scope
                    .iter()
                    .find(|entry| entry.name == var_to_look_for)
            });

        return variable.ok_or(std::io::Error::new(
            io::ErrorKind::NotFound,
            format!("Something went wrong. Variable no found in either method or class scopes. Var that we looked for is: {}", var_to_look_for),
        ));
    }
    fn compile_let(&mut self) -> io::Result<()> {
        self.write_open_tag("letStatement")?;
        self.indentation += 2;

        let let_tok = self.expect_value("let")?;
        self.write_token(&let_tok.value, let_tok.kind.as_str())?;

        let varname_tok = self.expect_kind("identifier")?;
        // look it up on the table, put in a helper function later
        self.write_token(&varname_tok.value, varname_tok.kind.as_str())?;

        // we pop this at the end vm_writer.pop
        let (var_to_pop_i, var_to_pop_segment) = {
            let variable_to_pop = self.get_variable_from_scope(&varname_tok.value)?;
            (
                variable_to_pop.index,
                variable_to_pop.kind.kind_to_segment(),
            )
        };

        let is_arr_assignment = if let Some(t) = self.peek() {
            if t.value == "[" {
                // we are an array
                let open_bracket = self.expect_value("[")?;
                self.write_token(&open_bracket.value, open_bracket.kind.as_str())?;

                self.compile_expression()?;

                self.vm_writer
                    .write_push(&var_to_pop_segment, var_to_pop_i)?;
                self.vm_writer.write_arithmetic("add")?;

                let close_bracket = self.expect_value("]")?;
                self.write_token(&close_bracket.value, close_bracket.kind.as_str())?;
                true
            } else {
                false
            }
        } else {
            false
        };

        let eq_token = self.expect_value("=")?;
        self.write_token(&eq_token.value, eq_token.kind.as_str())?;
        // handle let x = arr[4] case
        self.compile_expression()?;
        let semic_token = self.expect_value(";")?;
        self.write_token(&semic_token.value, semic_token.kind.as_str())?;

        self.indentation -= 2;
        self.write_close_tag("letStatement")?;

        if is_arr_assignment {
            // store value from the rhs in temp, store lhs arr in THAT
            // push temp to stack, pop temp and put in value inside THAT
            self.vm_writer.write_pop("temp", 0)?;
            self.vm_writer.write_pop("pointer", 1)?;
            self.vm_writer.write_push("temp", 0)?;

            self.vm_writer.write_pop("that", 0)?;
        } else {
            self.vm_writer.write_pop(var_to_pop_segment, var_to_pop_i)?;
        }

        Ok(())
    }

    fn compile_if(&mut self) -> io::Result<()> {
        self.write_open_tag("ifStatement")?;
        self.indentation += 2;

        let if_tok = self.expect_value("if")?;
        self.write_token(&if_tok.value, if_tok.kind.as_str())?;
        let open_paren_token = self.expect_value("(")?;
        self.write_token(&open_paren_token.value, open_paren_token.kind.as_str())?;

        self.compile_expression()?;
        self.vm_writer.write_arithmetic("not")?;

        let label_idx = self.label_index;
        self.label_index += 1;

        let if_false_label = format!("IF_FALSE${}", label_idx); // if condition is false run else statements
        let if_true_label = format!("IF_TRUE${}", label_idx); // we use this label to jump over the false statements after running the first statements

        self.vm_writer.write_if(&if_false_label)?;

        let close_paren_token = self.expect_value(")")?;
        self.write_token(&close_paren_token.value, close_paren_token.kind.as_str())?;

        let open_bracket_token = self.expect_value("{")?;
        self.write_token(&open_bracket_token.value, open_bracket_token.kind.as_str())?;

        self.compile_statements()?;

        self.vm_writer.write_goto(&if_true_label)?;

        let close_bracket_token = self.expect_value("}")?;
        self.write_token(
            &close_bracket_token.value,
            close_bracket_token.kind.as_str(),
        )?;

        self.vm_writer.write_label(&if_false_label)?;
        if let Some(t) = self.peek() {
            if t.value == "else" {
                let else_tok = self.expect_value("else")?;
                self.write_token(&else_tok.value, else_tok.kind.as_str())?;

                let open_bracket_token = self.expect_value("{")?;
                self.write_token(&open_bracket_token.value, open_bracket_token.kind.as_str())?;
                self.compile_statements()?;

                let close_bracket_token = self.expect_value("}")?;

                self.write_token(
                    &close_bracket_token.value,
                    close_bracket_token.kind.as_str(),
                )?;
            }
        }
        self.vm_writer.write_label(&if_true_label)?;
        self.indentation -= 2;
        self.write_close_tag("ifStatement")?;
        Ok(())
    }
    fn compile_while(&mut self) -> io::Result<()> {
        self.write_open_tag("whileStatement")?;
        self.indentation += 2;

        let while_tok = self.expect_value("while")?;
        self.write_token(&while_tok.value, while_tok.kind.as_str())?;

        let open_paren_token = self.expect_value("(")?;
        self.write_token(&open_paren_token.value, open_paren_token.kind.as_str())?;

        let label_idx = self.label_index;
        self.label_index += 1;

        let if_false_label = format!("IF_FALSE${}", label_idx);
        let if_true_label = format!("IF_TRUE${}", label_idx);

        self.vm_writer.write_label(&if_true_label)?; // while start
        self.compile_expression()?;
        self.vm_writer.write_arithmetic("not")?;
        self.vm_writer.write_if(&if_false_label)?; // skip if false expression

        let close_paren_token = self.expect_value(")")?;

        self.write_token(&close_paren_token.value, close_paren_token.kind.as_str())?;

        let open_bracket_token = self.expect_value("{")?;
        self.write_token(&open_bracket_token.value, open_bracket_token.kind.as_str())?;

        self.compile_statements()?;
        self.vm_writer.write_goto(&if_true_label)?;

        self.vm_writer.write_label(&if_false_label)?;
        let close_bracket_token = self.expect_value("}")?;
        self.write_token(
            &close_bracket_token.value,
            close_bracket_token.kind.as_str(),
        )?;
        self.indentation -= 2;
        self.write_close_tag("whileStatement")?;

        Ok(())
    }
    fn compile_return(&mut self) -> io::Result<()> {
        self.write_open_tag("returnStatement")?;
        self.indentation += 2;

        let return_tok = self.expect_value("return")?;
        self.write_token(&return_tok.value, return_tok.kind.as_str())?;
        if let Some(t) = self.peek() {
            if t.value != ";" {
                self.compile_expression()?;
                self.vm_writer.write_return(true)?;
            } else {
                self.vm_writer.write_return(false)?;
            }
        }
        let semic_token = self.expect_value(";")?;
        self.write_token(&semic_token.value, semic_token.kind.as_str())?;
        self.indentation -= 2;
        self.write_close_tag("returnStatement")?;
        Ok(())
    }

    fn compile_do(&mut self) -> io::Result<()> {
        self.write_open_tag("doStatement")?;
        self.indentation += 2;

        let do_tok = self.expect_value("do")?;
        self.write_token(&do_tok.value, do_tok.kind.as_str())?;

        self.compile_expression()?; // handles functions here

        self.vm_writer.write_pop("temp", 0)?;

        let semic_token = self.expect_value(";")?;
        self.write_token(&semic_token.value, semic_token.kind.as_str())?;

        self.indentation -= 2;
        self.write_close_tag("doStatement")?;

        Ok(())
    }

    fn compile_expression(&mut self) -> io::Result<()> {
        self.write_open_tag("expression")?;
        self.indentation += 2;

        self.compile_term()?;

        while self.is_operator() {
            self.normalize_symbol();
            let operator_tok = self.expect_kind("symbol")?;
            self.write_token(&operator_tok.value, operator_tok.kind.as_str())?;
            self.compile_term()?;

            if operator_tok.value == "multiply" {
                self.vm_writer.write_call("Math.multiply", 2)?;
            } else if operator_tok.value == "divide" {
                self.vm_writer.write_call("Math.divide", 2)?;
            } else {
                self.vm_writer.write_arithmetic(&operator_tok.value)?;
            }
        }
        self.indentation -= 2;

        self.write_close_tag("expression")?;
        Ok(())
    }

    fn compile_term(&mut self) -> io::Result<()> {
        self.write_open_tag("term")?;
        self.indentation += 2;
        // let game = SquareGame.new()
        //       here

        if let Some(first_tok) = self.advance() {
            match first_tok.kind.as_str() {
                "identifier" => {
                    if let Some(symbol_ahead) = self.peek() {
                        match symbol_ahead.value.as_str() {
                            // let a[b[a[3]]] = a[a[5]] * b[((7 - a[3]) - Main.double(2)) + 1];
                            "[" => {
                                /*
                                push local 0

                                */
                                let var_name = self.get_variable_from_scope(&first_tok.value)?;
                                self.vm_writer
                                    .write_push(var_name.kind.kind_to_segment(), var_name.index)?;

                                let open_bracket = self.expect_value("[")?;
                                self.write_token(&open_bracket.value, open_bracket.kind.as_str())?;

                                self.compile_expression()?;
                                self.vm_writer.write_arithmetic("add")?;
                                self.vm_writer.write_pop("pointer", 1)?;

                                self.vm_writer.write_push("that", 0)?;

                                let close_bracket = self.expect_value("]")?;
                                self.write_token(
                                    &close_bracket.value,
                                    close_bracket.kind.as_str(),
                                )?;
                            }
                            "(" => {
                                let f_name = first_tok.value.clone();
                                let open_paren = self.expect_value("(")?;
                                self.write_token(&open_paren.value, open_paren.kind.as_str())?;

                                self.vm_writer.write_push("pointer", 0)?;
                                let n_args = self.compile_expression_list()?;
                                let curr_class = self.curr_class.clone().unwrap();

                                let full_name = format!("{}.{}", curr_class, f_name);
                                self.vm_writer.write_call(&full_name, n_args + 1)?;

                                let close_paren = self.expect_value(")")?;
                                self.write_token(&close_paren.value, close_paren.kind.as_str())?;
                            }
                            "." => {
                                let dot_token = self.expect_value(".")?;
                                self.write_token(&dot_token.value, dot_token.kind.as_str())?;
                                let subroutine_name_token = self.expect_kind("identifier")?;

                                self.write_token(
                                    &subroutine_name_token.value,
                                    subroutine_name_token.kind.as_str(),
                                )?;

                                let open_paren = self.expect_value("(")?;
                                self.write_token(&open_paren.value, open_paren.kind.as_str())?;

                                if let Ok(entry) = self.get_variable_from_scope(&first_tok.value) {
                                    let type_name = entry.type_name.to_owned();
                                    let index = entry.index;
                                    let segment = entry.kind.kind_to_segment().to_owned();

                                    self.vm_writer.write_push(segment.as_str(), index)?;

                                    let n_args = self.compile_expression_list()?;
                                    let full_subroutine_name =
                                        format!("{}.{}", type_name, &subroutine_name_token.value);
                                    self.vm_writer
                                        .write_call(&full_subroutine_name, n_args + 1)?;
                                } else {
                                    //
                                    let n_args = self.compile_expression_list()?;
                                    let full_subroutine_name = format!(
                                        "{}.{}",
                                        first_tok.value, subroutine_name_token.value
                                    );
                                    self.vm_writer.write_call(&full_subroutine_name, n_args)?;
                                }

                                let close_paren = self.expect_value(")")?;
                                self.write_token(&close_paren.value, close_paren.kind.as_str())?;
                            }
                            _ => {
                                let entry = self.get_variable_from_scope(&first_tok.value)?;
                                let segment = entry.kind.kind_to_segment();
                                let index = entry.index;
                                self.vm_writer.write_push(segment, index)?;
                            }
                        }
                    }
                }
                "integerConstant" => {
                    self.write_token(&first_tok.value, first_tok.kind.as_str())?;
                    let parsedIndex = first_tok.value.parse::<usize>().unwrap(); // will come back here
                    self.vm_writer.write_push("constant", parsedIndex)?;
                }
                "stringConstant" => {
                    // ? handle
                    let s_len = first_tok.value.len();
                    self.vm_writer.write_push("constant", s_len)?;
                    self.vm_writer.write_call("String.new", 1)?;
                    // use String.appendChar(char) for each char
                    // iterate over the string

                    // fix later

                    for c in first_tok.value.chars() {
                        // unicode scalar val but we only handle asciis
                        // self.vm_writer.write_push(seg, index)?;
                        self.vm_writer.write_push("constant", c as usize)?;
                        self.vm_writer.write_call("String.appendChar", 2)?;
                    }

                    self.write_token(&first_tok.value, first_tok.kind.as_str())?;
                }
                "symbol" => match first_tok.value.as_str() {
                    "(" => {
                        self.write_token(&first_tok.value, first_tok.kind.as_str())?;
                        self.compile_expression()?;
                        let close_paren = self.expect_value(")")?;
                        self.write_token(&close_paren.value, close_paren.kind.as_str())?;
                    }
                    "-" | "~" => {
                        self.write_token(&first_tok.value, first_tok.kind.as_str())?;
                        self.compile_term()?;
                        let unaryOp = if first_tok.value == "-" { "neg" } else { "not" };
                        self.vm_writer.write_arithmetic(unaryOp)?;
                    }
                    _ => {}
                },
                _ => {
                    const KEYWORD_CONST: [&str; 4] = ["true", "false", "null", "this"];
                    if KEYWORD_CONST.contains(&first_tok.value.as_str()) {
                        self.write_token(&first_tok.value, "keyword")?;
                        match first_tok.value.as_str() {
                            "true" => {
                                self.vm_writer.write_push("constant", 1)?;
                                self.vm_writer.write_arithmetic("neg")?;
                            }
                            "false" => {
                                self.vm_writer.write_push("constant", 0)?;
                            }
                            "null" => {
                                self.vm_writer.write_push("constant", 0)?;
                            }
                            "this" => {
                                self.vm_writer.write_push("pointer", 0)?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        self.indentation -= 2;
        self.write_close_tag("term")?;

        Ok(())
    }
    fn compile_expression_list(&mut self) -> io::Result<usize> {
        self.write_open_tag("expressionList")?;
        self.indentation += 2;
        let mut n_args: usize = 0;

        if let Some(tok) = self.peek() {
            match tok.value.as_str() {
                ")" => {}
                _ => {
                    self.compile_expression()?;
                    n_args += 1;

                    loop {
                        if let Some(t) = self.peek() {
                            match t.value.as_str() {
                                "," => {
                                    let comma = self.expect_value(",")?;
                                    self.write_token(&comma.value, comma.kind.as_str())?;
                                    self.compile_expression()?;
                                    n_args += 1;
                                }
                                _ => break,
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        self.indentation -= 2;
        self.write_close_tag("expressionList")?;
        Ok(n_args)
    }
}
