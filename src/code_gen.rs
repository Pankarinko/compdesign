use std::{
    ffi::OsString,
    io::Write,
    process::{Command, Stdio},
};

use crate::{instruction_selection::translate_functions, ir::IRFunction};

pub fn create_binary(program_in_ir: &mut Vec<IRFunction<'_>>, string: OsString) {
    let mut assembly = ".intel_syntax noprefix
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
    translate_functions(program_in_ir, &mut assembly);
    //println!("{}", assembly);
    let output_file = string.to_str().unwrap();
    /*let output_file = "this_file";*/
    let mut child = Command::new("gcc")
        .args(["-xassembler", "-o", output_file, "-"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin
        .write_all(assembly.as_bytes())
        .expect("Failed to write to stdin");
    child.wait().expect("gcc couldn't finish execution");
}
