use std::{env, fs, io};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use serde_bytes::Bytes;

// Valid characters for brainfuck commands as a constant.
const BRAINFUCK_CHARS: [char; 8] = ['>', '<', '+', '-', '.', ',' ,'[',']'];
// Valid memory address size for brainfuck.
const MAX_MEMORY_INDEX: usize = 30_000;

fn main() {
    ////////////////////////////// READ FILE /////////////////////////////////

    let args: Vec<String> = env::args().collect();

    // Accept one argument only. This is the file name or name and directory.
    if args.len() != 2 {
        panic!("[ERROR] Incorrect number of arguments \
                provided. 1 expected, {} provided.",
               args.len() - 1);
    }
    let file_name: String = args[1].clone();

    let path: &Path = Path::new(&file_name);
    let display = path.display();

    // Open file from filename provided and read commands to string.
    let mut file: File = match File::open(&path) {
        Err(why) => panic!("[ERROR] Could not open {}. {}", display, why),
        Ok(file) => file,
    };
    let mut commands = String::new();
    match file.read_to_string(&mut commands) {
        Err(why) => panic!("[ERROR] Could not read {}. {}", display, why),
        _ => {}
    }
    commands.retain(|c| BRAINFUCK_CHARS.contains(&c));

    /////////////////////////////// SETUP ENV  //////////////////////////////

    let mut memory: Vec<u8> = vec![0; MAX_MEMORY_INDEX];
    let mut pointer: usize = 0;
    let mut read_head: usize = 0;
    let mut changing_file_number: u128 = 0;
    let mut current_file_number: u128 = 0;
    let command_list: Vec<char> = commands.chars().collect();

    //////////////////////////////// INTERPRET //////////////////////////////

    // Compute the matching brackets positions for each bracket in the list of commands.
    let closures_list: Vec<Option<usize>> = loop_closures(&command_list);
    let mut command: char;
    while read_head < command_list.len() {
        command = command_list[read_head];
        match command {
            // '>' moves the read head/cell addressed right by one.
            // If this goes past the max memory, then it will change the file.
            '>' => {
                pointer = if pointer == MAX_MEMORY_INDEX {
                    if current_file_number == u128::MAX {
                        changing_file_number = 0;
                        0
                    } else {
                        changing_file_number += 1;
                        0
                    }
                } else { pointer + 1 };
            }
            // '<' moves the read head/cell addressed left by one.
            // If this goes under the max memory, then it will change the file.
            '<' => {
                pointer = if pointer == 0 {
                    if current_file_number == 0 {
                        changing_file_number = u128::MAX;
                        MAX_MEMORY_INDEX
                    } else {
                        changing_file_number -= 1;
                        MAX_MEMORY_INDEX
                    }
                } else { pointer - 1 };
            }
            // '+' increments the value in the cell addressed.
            '+' => { memory[pointer] += 1; }
            // '-' decrements the value in the cell addressed.
            '-' => { memory[pointer] -= 1; }
            // '.' outputs the value in the cell addressed as character.
            // As the cell value is u8, the character will be ASCII encoded.
            '.' => { print!("{}", memory[pointer] as char); }
            // Accept input from the stdin and writes the value as a u8
            // to the cell addressed. Note that only the
            // first character of the input will be taken.
            ',' => {
                let mut line = String::new();
                match std::io::stdin().read_line(&mut line) {
                    Err(_) => {panic!("[ERROR] Invalid Input.")}
                    _ => {}
                }
                let input: Vec<char> = line.chars().collect();
                let input_first_char: char = input[0];
                memory[pointer] = input_first_char as u8;
            }
            // If the cell addressed value is 0, then jump ahead to the matching bracket.
            // If not, do nothing.
            '[' => {
                if memory[pointer] == 0 {
                    let matching_closure: usize = closures_list[read_head].unwrap();
                    read_head = matching_closure;
                }
            }
            // If the cell addressed is non-zero, jump back to the matching bracket.
            // If not, do nothing.
            ']' => {
                if memory[pointer] != 0 {
                    let matching_closure: usize = closures_list[read_head].unwrap();
                    read_head = matching_closure
                }
            }
            _ => { panic!("[ERROR] Unexpected command \"{}\" in instructions.",command); }
        }
        // TRUE indicates that a memory block change is needed.
        if changing_file_number != current_file_number {
            // Serialise the current mem and write to mem/ folder as unique mem block.
            let writing_bytes: &Bytes = Bytes::new(&memory);
            let writing_file_name: String = current_file_number.to_string() + "mem.txt";
            let mut writing_file: File = File::create("mem/".to_owned() + &*writing_file_name)
                .expect("Unable to create mem file");
            writing_file.write_all(writing_bytes).expect("Unable to write to mem file");

            // Deserialise the new mem block if available. If not created create the new block.
            let reading_file_name: String = changing_file_number.to_string() + "mem.txt";
            match File::open(reading_file_name) {
                Ok(mut reading_file) => {
                    reading_file.read_to_end(&mut memory)
                        .expect("Unable to read mem block");
                }
                Err(_) => {
                    // File does not exist. Create a new one in memory.
                    memory = vec![0; MAX_MEMORY_INDEX];
                }
            }
        }
        read_head += 1;
        current_file_number = changing_file_number;
    }

    //////////////////////////////// CLEANUP ENV //////////////////////////////

    // Clean up all memory blocks in the mem directory.
    remove_dir_contents("mem/")
        .expect("Unable to clean up memory blocks");
}

// Removes all contents in a directory.
fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        fs::remove_file(entry?.path())?;
    } Ok(())
}

// Compute the positions of the matching bracket for each bracket.
// Returns a vector of matching bracket positions where relevant.
fn loop_closures(commands: &Vec<char>) -> Vec<Option<usize>> {
    let mut stack : Vec<usize> = Vec::new();
    let mut closure: Vec<Option<usize>> = vec![None; commands.len()];
    let mut current_index: usize = 0;
    for command in commands {
        if command == &'[' {
            stack.push(current_index.clone())
        }
        if command == &']' {
            assert_ne!(
                stack.len(),
                0,
                "[SYNTAX ERROR] Unmatched loop pairs at position {}.",
                current_index
            );
            let matching_index: usize = stack.pop().unwrap();
            closure[current_index] = Option::from(matching_index);
            closure[matching_index] = Option::from(current_index);
        }
        current_index += 1;
    }
    assert_eq!(
        stack.len(),
        0,
        "[SYNTAX ERROR] Unmatched loop pair(s) at {:#?}",
        stack
    );
    return closure
}