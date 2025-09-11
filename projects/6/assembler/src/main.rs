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

use std::collections::HashMap;

enum CommandType {
    ACommand, //for @Xxx where xxx is either a symbol or a decimal number
    BCommand, // for dest=comp; jump
    LCommand, // for (xxx) where Xxx is a symbol. like (LOOP)
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

fn main() {
    let symbol_table: HashMap<String, u32> = HashMap::new();
    //Takes in an .asm file and returns a .hack file and reads its line by line twice
}
