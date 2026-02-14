# Nand2Tetris

This repository contains my complete implementation of the Nand2Tetris course projects from *The Elements of Computing Systems* by Noam Nisan and Shimon Schocken.

Nand2tetris is a hands-on journey that builds a complete computer system from the ground up, starting with basic logic gates and culminating in a high-level programming language and operating system. Each project builds upon the previous ones, demonstrating how complex systems emerge from simple components.

## Technologies Used

- **HDL** (For chapters 1, 2, 3, 5)
- **Hack assembly** (chapter 4)
- **Rust** (For the assembler, vm_translator and compiler)
- **Jack** (For the OS)

## Project Overview

### Part 1: Boolean Logic

Implementation of fundamental logic gates (AND, OR, NOT, XOR, MUX, DMUX) using only NAND gates. Built increasingly complex chips including multi-bit variants and multi-way gates.

### Part 2: Boolean Arithmetic

Construction of arithmetic circuits including a Half Adder, Full Adder, 16-bit Adder, and the complete Arithmetic Logic Unit (ALU) that powers the Hack computer.

### Part 3: Sequential Logic

Development of memory elements from Data Flip-Flops (DFF) to registers, RAM units, and a Program Counter. This chapter introduces time and state into the computer architecture.

### Part 4: Machine Language

Low-level programming in Hack assembly language. Wrote two programs in asm to get a better understanding of the hardware-software interface.

### Part 5: Computer Architecture

Integration of the CPU, memory, and I/O into the complete Hack computer platform. Implemented the CPU chip that executes Hack machine language instructions.

### Part 6: Assembler

Built an assembler that translates symbolic Hack assembly code into binary machine code. It handles symbols, labels, and variables to generate executable programs.

### Part 7: Virtual Machine I - Stack Arithmetic

Developed the first part of a VM translator that converts VM commands into Hack assembly. Implemented stack arithmetic and memory access commands.

### Part 8: Virtual Machine II - Program Control

Extended the VM translator to handle program flow (branching, loops) and function calls. Completed the full VM-to-assembly translation layer.

### Part 9: High-Level Language

Designed and implemented a simple "Hangman" application Jack programming language, in order to understand the languages constructs better before implementing the compiler.

### Part 10: Compiler I - Syntax Analysis

Built a syntax analyzer that parses Jack programs and generates an XML parse tree. Implemented tokenization and recursive descent parsing following the Jack grammar.

### Part 11: Compiler II - Code Generation

Extended the Jack compiler to its full functionality. Used the syntax analyzer from Project 10 to recursively navigate the parse tree, generating corresponding VM instructions. Built hierarchical symbol tables to track class-level (field | static) and subroutine-level (argument | local) variables with proper scoping. Implemented expression compilation using recursive descent, handling operator precedence and evaluating expressions in postfix order. Statement compilation covered all control structures—variable assignments, conditionals, loops, function calls, and returns.

### Part 12: Operating System

Implemented the Jack OS library, a collection of eight classes providing essential system services for Jack applications. The Memory class handles dynamic memory allocation through alloc() and deAlloc() functions, implementing a first-fit algorithm to efficiently manage heap memory. Here I also added the optional defrag() function that merges fragmented memory into one big memory block. The Math class handles basic Math operations. Since the Jack "hardware" doesn't handle multiplication or division, its implemented in the hardware with two efficient algorithms( O(n) where n is the number of bits). The String class enables string representation, manipulation, and conversion between strings and integers. The Output class manages character rendering to the screen using an 11-row character bitmap, handling cursor positioning and screen updates. I used bit masking in order to represent two characters in one word. The Screen class implements graphics primitives for the 512×256 monochrome display, including pixel drawing, line rendering using Bresenham's algorithm, and shapes like rectangles and circles. The Keyboard class provides character input handling with functions for detecting key presses and reading characters, lines, and integers from user input.
