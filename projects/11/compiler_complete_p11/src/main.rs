#![allow(dead_code, non_snake_case)]
use regex::Regex;

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};

use std::path::PathBuf;

use std::{env, io};

fn main() -> io::Result<()> {
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

        let mut c_engine = compilation_engine::new(jack_t.tokens).unwrap();

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
    fn kind_to_segment(&self) -> &str {
        match self {
            Identifier_Kind::STATIC => "static",
            Identifier_Kind::FIELD => "this",
            Identifier_Kind::ARG => "arg",
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
    fn new(filename: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(filename)?;

        Ok(Self {
            file: Some(BufWriter::new(file)),
        })
    }
    fn write_push(&mut self, segment: &str, index: usize) -> io::Result<()> {
        let vm_to_write = format!(r#"push {} {}\n"#, segment, index);
        self.write_to_file(vm_to_write)?;

        Ok(())
    }
    fn write_pop(&mut self, segment: &str, index: usize) -> io::Result<()> {
        let vm_to_write = format!(r#"pop {} {}\n"#, segment, index);
        self.write_to_file(vm_to_write)?;

        Ok(())
    }
    fn write_arithmetic(&mut self, command: &str) -> io::Result<()> {
        let vm_to_write = format!(r#"{}\n"#, command);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_label(&mut self, label: &str) -> io::Result<()> {
        let vm_to_write = format!(r#"label {}\n"#, label);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_goto(&mut self, label: &str) -> io::Result<()> {
        let vm_to_write = format!(r#"goto {}\n"#, label);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_if(&mut self, label: &str) -> io::Result<()> {
        let vm_to_write = format!(r#"if-goto {}\n"#, label);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_call(&mut self, name: &str, n_args: usize) -> io::Result<()> {
        let vm_to_write = format!(r#"call {} {}\n"#, name, n_args);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_function(&mut self, name: &str, n_locals: usize) -> io::Result<()> {
        let vm_to_write = format!(r#"function {} {}\n"#, name, n_locals);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_return(&mut self) -> io::Result<()> {
        let vm_to_write = format!(r#"return\n"#);
        self.write_to_file(vm_to_write)?;
        Ok(())
    }
    fn write_to_file(&mut self, str_to_write: String) -> io::Result<()> {
        if let Some(f) = self.file.as_mut() {
            // repetetive maybe refactor into a function
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
}

impl compilation_engine {
    fn new(tokens: Vec<Token>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open("output.xml")
            .unwrap();

        let symbol_table = symbol_table::new();
        let vm_writer = VM_Writer::new("output.vm")?; // will fix later 
        let writer = Self {
            symbol_table,
            file: Some(BufWriter::new(file)),
            tokens: tokens,
            pos: 0,
            indentation: 0,
            vm_writer,
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

    fn is_operator(&mut self) -> bool {
        let operators: [&str; 9] = ["+", "-", "*", "/", "&", "|", "<", ">", "="];
        self.peek()
            .is_some_and(|x| operators.contains(&x.value.as_str()))
    }
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
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
                "<" => self.tokens[self.pos].value = "&lt;".to_string(),
                ">" => self.tokens[self.pos].value = "&gt;".to_string(),
                "\"" => self.tokens[self.pos].value = "&quot;".to_string(),
                "&" => self.tokens[self.pos].value = "&amp;".to_string(),
                _ => (),
            }
        }
    }

    fn compile_class(&mut self) -> io::Result<()> {
        self.write_open_tag("class")?;
        self.indentation += 2;
        let class_tok = self.expect_value("class")?;

        self.write_token(&class_tok.value, class_tok.kind.as_str())?;
        let class_identifier = self.expect_kind("identifier")?;
        self.write_token(&class_identifier.value, class_identifier.kind.as_str())?;
        let open_bracket = self.expect_value("{")?;
        self.write_token(&open_bracket.value, open_bracket.kind.as_str())?;

        while let Some(tok) = self.peek() {
            if tok.value == "static" || tok.value == "field" {
                self.compile_class_var_dec()?;
            } else if tok.value == "constructor" || tok.value == "function" || tok.value == "method"
            {
                self.compile_subroutine()?;
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
                    // here we are looping over tokens, looks like this for example: int x, y, z;
                    // is it possible to get name(x then y then z) and type without refactoring the loop?

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
    fn compile_subroutine(&mut self) -> io::Result<()> {
        self.write_open_tag("subroutineDec")?;
        self.indentation += 2;
        let kw_1 = self.expect_kind("keyword")?;
        self.write_token(&kw_1.value, kw_1.kind.as_str())?;

        let kw_2 = self.advance().expect("expected return type or 'void'");
        self.write_token(&kw_2.value, kw_2.kind.as_str())?;

        let identifier = self.expect_kind("identifier")?;

        self.write_token(&identifier.value, identifier.kind.as_str())?;

        let opening_parenthesis = self
            .advance()
            .expect("expected ( , -> from subroutine method");

        self.write_token(
            &opening_parenthesis.value,
            opening_parenthesis.kind.as_str(),
        )?;

        self.compile_parameter_list()?;

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

        self.compile_var_dec()?;

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
        self.write_open_tag("parameterList")?;
        self.indentation += 2;
        while let Some(t) = self.peek() {
            if t.value == ")" {
                break;
            }

            let tok = self
                .advance()
                .expect("from compile_paramlist -> expected token");
            self.write_token(&tok.value, tok.kind.as_str())?;
        }
        self.indentation -= 2;
        self.write_close_tag("parameterList")?;

        Ok(())
    }
    fn compile_var_dec(&mut self) -> io::Result<()> {
        while let Some(tok) = self.peek() {
            if tok.value == "var" {
                self.write_open_tag("varDec")?;
                self.indentation += 2;
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
                self.indentation -= 2;
                self.write_close_tag("varDec")?;
            } else {
                break;
            }
        }

        Ok(())
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

    fn compile_let(&mut self) -> io::Result<()> {
        self.write_open_tag("letStatement")?;
        self.indentation += 2;

        let let_tok = self.expect_value("let")?;
        self.write_token(&let_tok.value, let_tok.kind.as_str())?;

        let varname_tok = self.expect_kind("identifier")?;
        self.write_token(&varname_tok.value, varname_tok.kind.as_str())?;

        if let Some(t) = self.peek() {
            if t.value == "[" {
                let open_bracket = self.expect_value("[")?;
                self.write_token(&open_bracket.value, open_bracket.kind.as_str())?;
                self.compile_expression()?;
                let close_bracket = self.expect_value("]")?;
                self.write_token(&close_bracket.value, close_bracket.kind.as_str())?;
            }
        }

        let eq_token = self.expect_value("=")?;
        self.write_token(&eq_token.value, eq_token.kind.as_str())?;
        self.compile_expression()?;
        let semic_token = self.expect_value(";")?;
        self.write_token(&semic_token.value, semic_token.kind.as_str())?;

        self.indentation -= 2;
        self.write_close_tag("letStatement")?;
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
        let close_paren_token = self.expect_value(")")?;
        self.write_token(&close_paren_token.value, close_paren_token.kind.as_str())?;

        let open_bracket_token = self.expect_value("{")?;
        self.write_token(&open_bracket_token.value, open_bracket_token.kind.as_str())?;

        self.compile_statements()?;

        let close_bracket_token = self.expect_value("}")?;
        self.write_token(
            &close_bracket_token.value,
            close_bracket_token.kind.as_str(),
        )?;

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

        self.compile_expression()?;

        let close_paren_token = self.expect_value(")")?;

        self.write_token(&close_paren_token.value, close_paren_token.kind.as_str())?;

        let open_bracket_token = self.expect_value("{")?;
        self.write_token(&open_bracket_token.value, open_bracket_token.kind.as_str())?;

        self.compile_statements()?;

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

        let name_identifier = self.expect_kind("identifier")?;
        self.write_token(&name_identifier.value, name_identifier.kind.as_str())?;

        if let Some(t) = self.peek() {
            match t.value.as_str() {
                "(" => {
                    let open_paren = self.expect_value("(")?;
                    self.write_token(&open_paren.value, open_paren.kind.as_str())?;
                    self.compile_expression_list()?;
                    let close_paren = self.expect_value(")")?;
                    self.write_token(&close_paren.value, close_paren.kind.as_str())?;
                }
                "." => {
                    let dot_tok = self.expect_value(".")?;
                    self.write_token(&dot_tok.value, dot_tok.kind.as_str())?;
                    let subroutine_name = self.expect_kind("identifier")?;
                    self.write_token(&subroutine_name.value, subroutine_name.kind.as_str())?;
                    let open_paren = self.expect_value("(")?;
                    self.write_token(&open_paren.value, open_paren.kind.as_str())?;
                    self.compile_expression_list()?;
                    let close_paren = self.expect_value(")")?;
                    self.write_token(&close_paren.value, close_paren.kind.as_str())?;
                }
                _ => {}
            }
        }

        let semic_token = self.expect_value(";")?;
        self.write_token(&semic_token.value, semic_token.kind.as_str())?;

        self.indentation -= 2;
        self.write_close_tag("doStatement")?;

        Ok(())
    }

    fn compile_expression(&mut self) -> io::Result<()> {
        self.write_open_tag("expression")?;
        self.indentation += 2;
        //         if exp is "term1 op1 term2 op2 term3 op3 ... termn ":
        // compileTerm(term1)
        // compileTerm(term2)
        // output "op1"
        // so need to output two terms then operator, if exp is just term by itself such as term, then just compile term
        // look ahead and see if the next token is a operator, if no, just compile term.
        // if nextToken == operator then output: push curr, push curr+1, push operator
        self.compile_term()?;

        while self.is_operator() {
            self.normalize_symbol();
            let operator_tok = self.expect_kind("symbol")?;
            self.write_token(&operator_tok.value, operator_tok.kind.as_str())?;
            self.compile_term()?;
        }
        self.indentation -= 2;

        self.write_close_tag("expression")?;
        Ok(())
    }

    fn compile_term(&mut self) -> io::Result<()> {
        self.write_open_tag("term")?;
        self.indentation += 2;

        if let Some(first_tok) = self.advance() {
            match first_tok.kind.as_str() {
                "identifier" => {
                    self.write_token(&first_tok.value, first_tok.kind.as_str())?;
                    if let Some(symbol_ahead) = self.peek() {
                        match symbol_ahead.value.as_str() {
                            "[" => {
                                let open_bracket = self.expect_value("[")?;
                                self.write_token(&open_bracket.value, open_bracket.kind.as_str())?;

                                self.compile_expression()?;
                                let close_bracket = self.expect_value("]")?;
                                self.write_token(
                                    &close_bracket.value,
                                    close_bracket.kind.as_str(),
                                )?;
                            }
                            "(" => {
                                let open_paren = self.expect_value("(")?;
                                self.write_token(&open_paren.value, open_paren.kind.as_str())?;
                                self.compile_expression()?;
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
                                self.compile_expression_list()?;
                                let close_paren = self.expect_value(")")?;
                                self.write_token(&close_paren.value, close_paren.kind.as_str())?;
                            }
                            _ => {}
                        }
                    }
                }
                "integerConstant" | "stringConstant" => {
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
                    }
                    _ => {}
                },
                _ => {
                    const KEYWORD_CONST: [&str; 4] = ["true", "false", "null", "this"];
                    if KEYWORD_CONST.contains(&first_tok.value.as_str()) {
                        self.write_token(&first_tok.value, "keyword")?;
                    }
                }
            }
        }
        self.indentation -= 2;
        self.write_close_tag("term")?;

        Ok(())
    }
    fn compile_expression_list(&mut self) -> io::Result<()> {
        self.write_open_tag("expressionList")?;
        self.indentation += 2;

        if let Some(tok) = self.peek() {
            match tok.value.as_str() {
                ")" => {}
                _ => {
                    self.compile_expression()?;

                    loop {
                        if let Some(t) = self.peek() {
                            match t.value.as_str() {
                                "," => {
                                    let comma = self.expect_value(",")?;
                                    self.write_token(&comma.value, comma.kind.as_str())?;
                                    self.compile_expression()?;
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
        Ok(())
    }
}
