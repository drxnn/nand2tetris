#![allow(unused, dead_code, non_camel_case_types)]
use regex::Regex;

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::thread::ThreadId;
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

// unnecessary right now

// enum TERM_TYPES {
//     INTEGER_CONST,
//     STRING_CONST,
//     KEYWORD_CONST,
//     VAR_NAME,
//     VAR_NAME_EXPRESSION,
//     SUBROUTINE_CALL,
//     EXPRESSION,
//     UNARY_OP,
// }

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
    /* token validation is missing
    assume files are valid Jack
        will add later at the end
     */
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

    fn expect_any_advance(&mut self, expected: &[&str]) -> io::Result<Token> {
        let tok = self.advance().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("expected one of {:?}", expected),
            )
        })?;
        if expected.contains(&&tok.value.as_str()) {
            Ok(tok)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "expected one of {:?}, found '{}' instead",
                    expected, tok.value
                ),
            ))
        }
    }

    fn is_operator(&mut self) -> bool {
        let operators: [&str; 9] = ["+", "-", "*", "/", "&", "|", "<", ">", "="];
        self.peek_ahead()
            .is_some_and(|x| operators.contains(&x.value.as_str()))
    }
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }
    fn peek_ahead(&self) -> Option<Token> {
        self.tokens.get(self.pos + 1).cloned()
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
        while let Some(tok) = self.advance() {
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
        /*
        subroutine example:
            function void main(  parameterList missing here) {  <------------ all start like this

          var SquareGame game;  <------------ varDecs
          let game = SquareGame.new(); <------------ varDec
          do game.run();    <------------ do statement
          do game.dispose(); <------------ do statement
          return;             <------------ return statement
        } */
        self.write_open_tag("subroutineDec")?;
        let kw_1 = self
            .advance()
            .expect("expected kw_1, -> from subroutine method");
        self.write_token(&kw_1.value, kw_1.kind.as_str())?;

        let kw_2 = self
            .advance()
            .expect("expected kw_2, -> from subroutine method");
        self.write_token(&kw_2.value, kw_2.kind.as_str())?;

        let identifier = self
            .advance()
            .expect("expected method identifier, -> from subroutine method");
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

        // subroutineBody handle:
        let opening_bracket = self
            .advance()
            .expect("expected { , -> from subroutine method");
        self.write_token(&opening_bracket.value, opening_bracket.kind.as_str())?;

        self.compile_var_dec(); // loop inside to compile statements all vars

        self.compile_statements(); // loop inside to compile all statements

        let closing_bracket = self
            .advance()
            .expect("expected } , -> from subroutine method");
        self.write_token(&closing_bracket.value, closing_bracket.kind.as_str())?;

        self.write_close_tag("subroutineDec")?;

        Ok(())
    }
    fn compile_parameter_list(&mut self) -> io::Result<()> {
        self.write_open_tag("parameterList")?;

        while let Some(t) = self.peek() {
            if t.value == ")" {
                break;
            }

            let tok = self
                .advance()
                .expect("from compile_paramlist -> expected token");
            self.write_token(&tok.value, tok.kind.as_str())?;
        }
        self.write_close_tag("parameterList")?;

        Ok(())
    }
    fn compile_var_dec(&mut self) -> io::Result<()> {
        // varDec: 'var' type varName (',' varName)* ';'

        // add token validation later
        while let Some(tok) = self.peek() {
            if tok.value == "var" {
                self.write_open_tag("varDec")?;
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
                self.write_close_tag("varDec")?;
            } else {
                break;
            }
        }

        Ok(())
    }
    fn compile_statements(&mut self) -> io::Result<()> {
        self.write_open_tag("statements")?;
        while let Some(tok) = self.peek() {
            if ["let", "if", "do", "while", "return"].contains(&tok.value.as_str()) {
                // statement belw
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

        self.write_close_tag("statements")?;
        Ok(())
    }

    fn compile_let(&mut self) -> io::Result<()> {
        // 'let' varName ('[' expression ']')? '=' expression ';'
        // ex:     let game = SquareGame.new();
        //ex:      let arr[i+1] = x;
        self.write_open_tag("letStatement")?;

        let let_tok = self.advance().unwrap();
        self.write_token(&let_tok.value, let_tok.kind.as_str())?;

        let varname_tok = self.advance().unwrap();
        self.write_token(&varname_tok.value, varname_tok.kind.as_str())?;

        if let Some(t) = self.peek() {
            if t.value == "[" {
                let open_bracket = self.advance().unwrap();
                self.write_token(&open_bracket.value, open_bracket.kind.as_str())?;
                self.compile_expression()?;
                let close_bracket = self.advance().unwrap();
                self.write_token(&close_bracket.value, close_bracket.kind.as_str())?;
            }
        }

        let eq_token = self.advance().unwrap();
        self.write_token(&eq_token.value, eq_token.kind.as_str())?;
        self.compile_expression()?;
        let semic_token = self.advance().unwrap();
        self.write_token(&semic_token.value, semic_token.kind.as_str())?;

        self.write_close_tag("letStatement")?;
        Ok(())
    }
    fn compile_if(&mut self) -> io::Result<()> {
        // 'if''(' expression ')''{' statements '}' ('else''{' statements '}' )?
        // ex:  if (key = 81)  { let exit = true; }     // q key
        self.write_open_tag("ifStatement")?;

        let if_tok = self.advance().unwrap();
        self.write_token(&if_tok.value, if_tok.kind.as_str())?;
        let open_paren_token = self.advance().unwrap();
        self.write_token(&open_paren_token.value, open_paren_token.kind.as_str())?;

        self.compile_expression()?;
        let close_paren_token = self.advance().unwrap();
        self.write_token(&close_paren_token.value, close_paren_token.kind.as_str())?;

        let open_bracket_token = self.advance().unwrap();
        self.write_token(&open_bracket_token.value, open_bracket_token.kind.as_str())?;

        self.compile_statements()?; // statements //

        let close_bracket_token = self.advance().unwrap();
        self.write_token(
            &close_bracket_token.value,
            close_bracket_token.kind.as_str(),
        )?;

        if let Some(t) = self.peek() {
            if t.value == "else" {
                let else_tok = self.advance().unwrap();
                self.write_token(&else_tok.value, else_tok.kind.as_str())?;

                let open_bracket_token = self.advance().unwrap();
                self.write_token(&open_bracket_token.value, open_bracket_token.kind.as_str())?;
                self.compile_statements()?;
                let close_bracket_token = self.advance().unwrap();

                self.write_token(
                    &close_bracket_token.value,
                    close_bracket_token.kind.as_str(),
                )?;
            }
        }

        self.write_close_tag("ifStatement")?;
        Ok(())
    }
    fn compile_while(&mut self) -> io::Result<()> {
        // 'while''(' expression ')''{' statements '}'
        self.write_open_tag("whileStatement")?;

        let while_tok = self.expect_value("while")?;
        self.write_token(&while_tok.value, while_tok.kind.as_str())?;

        let open_paren_token = self.advance().unwrap();

        self.write_token(&open_paren_token.value, open_paren_token.kind.as_str())?;

        self.compile_expression()?;

        let close_paren_token = self.advance().unwrap();

        self.write_token(&close_paren_token.value, close_paren_token.kind.as_str())?;

        let open_bracket_token = self.advance().unwrap();
        self.write_token(&open_bracket_token.value, open_bracket_token.kind.as_str())?;

        self.compile_statements()?;

        let close_bracket_token = self.advance().unwrap();
        self.write_token(
            &close_bracket_token.value,
            close_bracket_token.kind.as_str(),
        )?;

        self.write_close_tag("whileStatement")?;

        Ok(())
    }
    fn compile_return(&mut self) -> io::Result<()> {
        //‘return’ expression? ';'
        self.write_open_tag("returnStatement")?;

        let return_tok = self.expect_value("return")?;
        self.write_token(&return_tok.value, return_tok.kind.as_str())?;
        if let Some(t) = self.peek() {
            if t.value != ";" {
                self.compile_expression()?;
            }
        }
        let semic_token = self.expect_value(";")?;
        self.write_token(&semic_token.value, semic_token.kind.as_str())?;

        self.write_close_tag("returnStatement")?;
        Ok(())
    }

    fn compile_do(&mut self) -> io::Result<()> {
        //doStatement: ‘do' subroutineCall ';'
        self.write_open_tag("doStatement")?;

        let do_tok = self.expect_value("do")?;
        self.write_token(&do_tok.value, do_tok.kind.as_str())?;

        self.compile_subroutine();

        let semic_token = self.expect_value(";")?;
        self.write_token(&semic_token.value, semic_token.kind.as_str())?;

        self.write_close_tag("doStatement")?;

        Ok(())
    }

    fn compile_expression(&mut self) -> io::Result<()> {
        /*
             term: integerConstant | stringConstant | keywordConstant | varName
             | varName '[' expression ']' | subroutineCall |'(' expression ')'| unaryOp term
        */
        self.write_open_tag("expression")?;
        self.compile_term()?;
        // maybe use while loop in case of multiple (op term)*

        while self.is_operator() {
            let operator_tok = self.advance().unwrap(); // ok because we looked ahead
            self.write_token(&operator_tok.value, operator_tok.kind.as_str())?;
            self.compile_term()?;
        }

        self.write_close_tag("expression")?;
        Ok(())
    }

    fn compile_term(&mut self) -> io::Result<()> {
        self.write_open_tag("term")?;

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
                                self.compile_expression_list()?;
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

        self.write_close_tag("term")?;

        Ok(())
    }
    fn compile_expression_list(&mut self) -> io::Result<()> {
        //(expression (',' expression)* )?
        self.write_open_tag("expressionList")?;

        // recursively check here

        self.write_close_tag("expressionList")?;
        Ok(())
    }
}
