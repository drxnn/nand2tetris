// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Fill.asm

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.

//// Replace this comment with your code.





@SCREEN
D=A
@i
M=D // M=16384


(LOOP)
@KBD
D=M
@MAKEWHITE
D;JEQ

// MAKE BLACK
@i
A=M
M=-1

@i
D=M+1
M=D

@LOOP
0;JMP



@MAKEWHITE
@KBD
D=M

@LOOP
0;JGT



@i
A=M
M=0

@i
D=M-1
M=D

@LOOP
0;JMP

