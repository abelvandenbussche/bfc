use std::fs::{File, read_to_string};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

enum Command {
    Add(u8),
    Sub(u8),
    MoveRight(u32),
    MoveLeft(u32),
    Output,
    Input,
}

fn main() {
    // Reading the bf from a file
    let file_path = PathBuf::from(std::env::args().skip(1).next().unwrap());
    let file_name = match file_path.file_prefix() {
        Some(n) => n,
        None => {
            eprintln!("Path should contain file name");
            return;
        }
    };
    let file_contents = match read_to_string(&file_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Invalid file name/path");
            return;
        }
    };

    // Parsing the contents of the file
    let mut commands = vec![];
    for c in file_contents.chars() {
        let output = match c {
            '<' => Some(Command::MoveLeft(1)),
            '>' => Some(Command::MoveRight(1)),
            '+' => Some(Command::Add(1)),
            '-' => Some(Command::Sub(1)),
            '.' => Some(Command::Output),
            ',' => Some(Command::Input),
            _ => None,
        };
        if let Some(cmd) = output {
            commands.push(cmd);
        }
    }

    // TODO Optimizing the intermediate representation

    // Converting intermediate into assembly
    let file = match File::create(format!("bin/{}.asm", file_name.to_string_lossy())) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Failed to create file");
            return;
        }
    };
    let mut writer = BufWriter::new(file);

    // Preamble
    writer.write_all(b"BITS 64\n").unwrap();
    writer.write_all(b"extern GetStdHandle\n").unwrap();
    writer.write_all(b"extern WriteFile\n").unwrap();
    writer.write_all(b"extern ExitProcess\n").unwrap();
    writer.write_all(b"global start\n").unwrap();
    writer
        .write_all(b"section .bss\ntape: resb 3000\n")
        .unwrap();
    writer.write_all(b"section .text\n").unwrap();
    writer.write_all(b"start:\nlea r12, [rel tape]\n").unwrap();

    // Adding the bf commands
    for cmd in commands {
        let to_write = match cmd {
            Command::Add(n) => format!("add byte [r12], {n}"),
            Command::Sub(n) => format!("sub byte [r12], {n}"),
            // TODO add boundary checks or something
            Command::MoveLeft(n) => format!("sub r12, {n}"),
            Command::MoveRight(n) => format!("add r12, {n}"),
            // TODO add the other commands
            _ => String::new(),
        };
        writer
            .write_all(format!("{to_write}\n").as_bytes())
            .unwrap();
    }

    // Appendix
    writer.write_all(b"xor ecx, ecx\ncall ExitProcess").unwrap();

    writer.flush().unwrap();
}
