# Rusted brainfuck 🧠
A simple rust interpreter for brainfuck written in Rust. Implements a version with 8-bit unsigned integers as cell values and memory array of length 30,000. This interpreter now supports serialisation of memory blocks for increased capacity!

The default memory block size is 30,000, and there is support for 2^128 (u128) blocks. This yields about 10 tredecillion cells, or:
 
 10,208,471,007,628,153,903,901,238,222,953,046,343,680,000 cells :)

Reference to brainfuck used for this project: https://esolangs.org/wiki/Brainfuck 
