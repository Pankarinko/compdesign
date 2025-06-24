use std::{
    ffi::OsString,
    io::Write,
    process::{Command, Stdio},
};

use crate::{
    instruction_selection::{init_stack_counter, translate_instruction},
    ir::IRCmd,
};

pub fn create_binary(program: Vec<IRCmd>, num_temps: usize, string: OsString) {
    let mut program_code = ".intel_syntax noprefix
        .global main
        .global _main
        .text
        main:
        call _main
        mov rdi, rax
        mov rax, 0x3C
        syscall
        _main:
"
    .to_string();
    let mut stack_counter = init_stack_counter(num_temps);
    for cmd in program.into_iter() {
        translate_instruction(num_temps, &mut stack_counter, cmd, &mut program_code);
    }
    //println!("{:#?}", program_code);
    let output_file = string.to_str().unwrap();
    /*let output_file = "this_file";*/
    let mut child = Command::new("gcc")
        .args(["-xassembler", "-o", output_file, "-"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin
        .write_all(program_code.as_bytes())
        .expect("Failed to write to stdin");
    child.wait().expect("gcc couldn't finish execution");
}
