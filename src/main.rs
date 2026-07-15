use std::fs::{File, read_to_string};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::{self};

enum Command {
    Add(u8),
    Sub(u8),
    MoveRight(u32),
    MoveLeft(u32),
    Output,
    Input,
    LoopStart,
    LoopEnd,
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
    }
    .to_string_lossy();

    let output_path = "bin/";

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
            '[' => Some(Command::LoopStart),
            ']' => Some(Command::LoopEnd),
            _ => None,
        };
        if let Some(cmd) = output {
            commands.push(cmd);
        }
    }

    // TODO Check for syntax errors
    let mut depth = 0;
    for i in &commands {
        match i {
            Command::LoopStart => depth += 1,
            Command::LoopEnd => depth -= 1,
            _ => {}
        }
    }
    if depth != 0 {
        eprintln!("Syntax error");
        return;
    }
    // TODO Optimizing the intermediate representation

    // Converting intermediate into assembly
    let file = match File::create(format!("bin/{}.asm", file_name)) {
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
    writer.write_all(b"extern ReadFile\n").unwrap();
    writer.write_all(b"extern ExitProcess\n").unwrap();
    writer.write_all(b"global start\n").unwrap();
    writer
        .write_all(b"section .bss\ntape: resb 3000\nwritten: resb 1\nread: resd 1\n")
        .unwrap();
    writer.write_all(b"section .text\n").unwrap();
    writer
        .write_all(b"start:\nsub rsp, 40\nlea r12, [rel tape]\n")
        .unwrap();
    writer
        .write_all(b"mov ecx, -11\ncall GetStdHandle\nmov r13, rax\n")
        .unwrap();
    writer
        .write_all(b"mov ecx, -10\ncall GetStdHandle\nmov r14, rax\n")
        .unwrap();

    // Adding the bf commands
    let mut label_numbers = vec![];
    let mut next_number = 0;
    for cmd in commands {
        let to_write = match cmd {
            Command::Add(n) => format!("add byte [r12], {n}"),
            Command::Sub(n) => format!("sub byte [r12], {n}"),
            // TODO add boundary checks or something
            Command::MoveLeft(n) => format!("sub r12, {n}"),
            Command::MoveRight(n) => format!("add r12, {n}"),
            // TODO remove assembly comments here
            Command::Output => String::from(
                "
mov rcx, r13 ; give handle to function
mov rdx, r12 ; address of what to write
mov r8d, 1 ; how many bytes to write
lea r9, [rel written] ; give windows address to write bytes written to
mov qword [rsp+32], 0 ; set the fifth arg to null
call WriteFile
                    ",
            ),
            Command::Input => String::from(
                "
mov rcx, r14 ; give handle to function
mov rdx, r12 ; address to write to
mov r8d, 1 ; how many bytes to read
lea r9, [rel read] ; give windows address to write bytes read to
mov qword [rsp+32], 0 ; set the fifth arg to null
call ReadFile
                    ",
            ),
            Command::LoopStart => {
                label_numbers.push(next_number);
                next_number += 1;
                format!(
                    "cmp byte [r12], 0\nje loop_end{}\nloop{}:",
                    next_number - 1,
                    next_number - 1
                )
            }
            Command::LoopEnd => {
                let label_number = label_numbers.pop().unwrap();
                format!("cmp byte [r12], 0\njne loop{label_number}\nloop_end{label_number}:")
            }
        };
        writer
            .write_all(format!("{to_write}\n").as_bytes())
            .unwrap();
    }

    // Appendix
    writer
        .write_all(b"add rsp, 40\nxor ecx, ecx\ncall ExitProcess")
        .unwrap();

    writer.flush().unwrap();

    // Compiling into executable
    process::Command::new("nasm")
        .args([
            "-f",
            "win64",
            &format!("{output_path}{file_name}.asm"),
            "-o",
            &format!("{output_path}{file_name}.obj"),
        ])
        .status()
        .expect("Failed to run nasm");

    process::Command::new("lld-link")
        .args([
            &format!("{output_path}{file_name}.obj"),
            "kernel32.lib",
            "/ENTRY:start",
            "/SUBSYSTEM:CONSOLE",
            &format!("/OUT:{output_path}{file_name}.exe"),
        ])
        .status()
        .expect("Failed to run lld-link");
}
