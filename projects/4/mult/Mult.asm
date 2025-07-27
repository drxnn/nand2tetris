// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
// The algorithm is based on repetitive addition.

//// Replace this comment with your code.

@R0
D=M // D = Memory[R0]
@i
M=D // Memory[i] = Memory[R0]
@R1
D=M
@R2
M=0


(LOOP)
@i
D=M
@END
D;JEQ


@R1
D=M
@R2
M=D+M


@i
M=M-1 // decrease i and check if its 0
D=M


@LOOP
0;JMP
@END
0;JMP