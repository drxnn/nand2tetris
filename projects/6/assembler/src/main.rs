/*Assembler to be written in rust
Takes in an .asm file and returns a .hack file
Constants and symbols:
Constants must be non-negative and are written in decimal notation.
A user-defined symbol can be any sequence of letters,
digits, underscore (_), dot (.), dollar sign ($), and colon (:)
that does not begin with a digit.

Comments:
Text beginning with two slashes (//) and ending at the end of
the line is considered a comment and is ignored.

White Space:
Space characters are ignored. Empty lines are ignored.

Case Conventions:
All the assembly mnemonics must be written in uppercase.
The rest (user-defined labels and variable names) is case sensitive.
The convention is to use uppercase for labels and lowercase for
variable names.

Proposed architecture:
Parser module that parses the input,
Code module that provides the binary codes of all the assembly mnemonics,
SymbolTable module that handles symbols,
main program that drives the entire translation process.

- Use a hashmap to store symbols and the address they point to
Remember:
Labels like (LOOP) or (END) should mark a ROM address.
Variables like @i, @sum — should mark RAM locations starting at address 16. */

// cargo run -- file.asm -> file.hack

//

use std::collections::HashMap;
use std::env;

use std::fs::File;
use std::io::{self, Read};

enum CommandType {
    Undefined,
    Acommand, //for @Xxx where xxx is either a symbol or a decimal number
    Ccommand, // for dest=comp; jump
    Lcommand, // for (xxx) where Xxx is a symbol. like (LOOP)
}
struct Parser {
    command_type: CommandType,
    symbol: String, //Returns the symbol or decimal xxx of the current command @Xxx or (Xxx). Should be called only when commandType() is A or L
    dest: String,   //Returns the dest mnemonic in the current C-command
    comp: String, // Returns the comp mnemonic in the current C-command, should only be called when C command
    jump: String, // Returns the jump mnemonic
}

// translate mnemonics to binary
struct CodeBinary;

impl CodeBinary {
    fn dest_to_binary(dest: String) {
        // return binary
    }
    fn comp_to_binary(comp: String) {
        // return binary
    }
    fn jump_to_binary(jump: String) {
        // return binary
    }
}

fn process_line(line: &str) {
    let mut Test = Parser {
        command_type: CommandType::Undefined,
        symbol: "".to_string(),
        dest: "".to_string(),
        comp: "".to_string(),
        jump: "".to_string(),
    };
    // first check what kind of instruction it is: A, C, L
    let first_char = line.chars().next().unwrap();
    if first_char == '@' {
        Test.command_type = CommandType::Acommand;
        Test.symbol = first_char.to_string();
    } else if first_char == '(' {
        Test.command_type = CommandType::Lcommand;
        Test.symbol = first_char.to_string();
    } else {
        Test.command_type = CommandType::Ccommand
    }
}

fn main() -> io::Result<()> {
    let predefined_symbols = vec![
        ("SP", 0),
        ("LCL", 1),
        ("ARG", 2),
        ("THIS", 3),
        ("THAT", 4),
        ("R0", 0),
        ("R1", 1),
        ("R2", 2),
        ("R3", 3),
        ("R4", 4),
        ("R5", 5),
        ("R6", 6),
        ("R7", 7),
        ("R8", 8),
        ("R9", 9),
        ("R10", 10),
        ("R11", 11),
        ("R12", 12),
        ("R13", 13),
        ("R14", 14),
        ("R15", 15),
        ("SCREEN", 16384),
        ("KBD", 24576),
    ];
    let symbol_table: HashMap<_, u32> = predefined_symbols.into_iter().collect();

    let mut arguments = env::args();
    arguments.next();

    let file_to_compile = arguments.next().unwrap();
    let mut f = File::open(file_to_compile)?;

    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    println!("here is file to process: {}", buffer);

    Ok(())
}
