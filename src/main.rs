use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// Declare fixed memory size for brainfuck memory array.
const MEMORY_SIZE: usize = 30_000;

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

    let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    let mut pointer: usize = 0;
    let mut read_head: usize = 0;
    let command_list: Vec<char> = commands.chars().collect();

    //////////////////////////////// INTERPRET //////////////////////////////

    // Compute the matching brackets positions for each bracket in the list of commands.
    let closures_list: Vec<Option<usize>> = loop_closures(&command_list);
    let mut command: char;
    while read_head < command_list.len() {
        command = command_list[read_head];
        match command {
            // '>' moves the read head/cell addressed right by one.
            '>' => { pointer = if pointer + 1 == MEMORY_SIZE { 0 } else { pointer + 1 }; }
            // '<' moves the read head/cell addressed left by one.
            '<' => { pointer = if pointer - 1 == usize::MAX { MEMORY_SIZE } else { pointer - 1 }; }
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
        read_head += 1;
    }

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


