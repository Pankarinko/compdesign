use std::{
    collections::HashMap,
    ffi::OsString,
    io::Write,
    process::{Command, Stdio, id},
};

use crate::{
    ast::{self, Exp, Program, Simp, Statement},
    semantics::decl_check,
};

pub fn execute(ast: Program<'_>, string: OsString) {
    let res = eval_program(ast.get_statements());
    create_binary(res, string);
}

pub fn create_binary(res: Result<i32, i32>, string: OsString) {
    match res {
        Ok(code) => {
            let file_string = format!(
                ".global main
        .global _main
        .text
        main:
        call _main
        movq %rax, %rdi
        movq $0x3C, %rax
        syscall
        _main:
        movq ${}, %rax
        ret
",
                code
            );
            let output_file = string.to_str().unwrap();
            /*let output_file = "this_file";*/
            let mut child = Command::new("gcc")
                .args(["-xassembler", "-o", output_file, "-"])
                .stdin(Stdio::piped())
                .spawn()
                .expect("Failed to spawn child process");
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write(file_string.as_bytes())
                .expect("Failed to write to stdin");
            child.wait().expect("gcc couldn't finish execution");
        }
        Err(_) => {
            let file_string = ".global main
    .global _main
    .text
    main:
    call _main
    movq %rax, %rdi
    movq $0x3C, %rax
    syscall
    _main:
    xor %eax, %eax
    div %eax
    ret
";
            let output_file = string.to_str().unwrap();
            /*let output_file = "this_file";*/
            let mut child = Command::new("gcc")
                .args(["-xassembler", "-o", output_file, "-"])
                .stdin(Stdio::piped())
                .spawn()
                .expect("Failed to spawn child process");
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin
                .write(file_string.as_bytes())
                .expect("Failed to write to stdin");
            child.wait().expect("gcc couldn't finish execution");
        }
    }
}

fn eval_program<'a>(statements: &'a Vec<Statement<'a>>) -> Result<i32, i32> {
    let mut used_idents: HashMap<&'a [u8], i32> = HashMap::new();
    let mut idents: HashMap<&'a [u8], usize> = HashMap::new();
    let mut decls: Vec<&'a [u8]> = Vec::new();
    let mut assignments: Vec<&'a [u8]> = Vec::new();
    for (i, stmt) in statements.iter().enumerate() {
        if !decl_check(stmt, i, &mut idents, &mut decls, &mut assignments) {
            return Err(7);
        }
        if let Some(res) = eval_stmt(stmt, &mut used_idents)? {
            return Ok(res);
        }
    }
    Ok(0)
}

fn eval_stmt<'a>(
    stmt: &Statement<'a>,
    idents: &mut HashMap<&'a [u8], i32>,
) -> Result<Option<i32>, i32> {
    match stmt {
        Statement::Decl(decl) => match decl {
            ast::Decl::Declare(_) => Ok(None),
            ast::Decl::Assign((ident, exp)) => {
                let _mess = idents.insert(*ident, eval_exp(exp, idents)?);
                Ok(None)
            }
        },
        Statement::Simp(Simp::Simp((lvalue, _, exp))) => {
            let _mess = idents.insert(lvalue.get_ident_lvalue(), eval_exp(exp, idents)?);
            Ok(None)
        }
        Statement::Return(exp) => Ok(Some(eval_exp(exp, idents)?)),
    }
}

fn eval_exp<'a>(exp: &Exp<'a>, idents: &HashMap<&'a [u8], i32>) -> Result<i32, i32> {
    match exp {
        Exp::Intconst(c) => return Ok(*c),
        Exp::Ident(name) => {
            return idents.get(name).copied().ok_or(-1);
        }
        Exp::Arithmetic(arith) => match arith.1 {
            ast::Binop::Plus => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        return Ok(e1.wrapping_add(e2));
                    }
                }
                return Err(-1);
            }
            ast::Binop::Minus => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        return Ok(e1.wrapping_sub(e2));
                    }
                }
                return Err(-1);
            }
            ast::Binop::Div => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        if e2 == 0 {
                            /*return floating point error if divided by zero*/
                            return Err(15);
                        }
                        return Ok(e1.wrapping_div(e2));
                    }
                };
                return Err(-1);
            }
            ast::Binop::Mult => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        return Ok(e1.wrapping_mul(e2));
                    }
                }
                return Err(-1);
            }
            ast::Binop::Mod => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        if let Some(res) = e1.checked_rem(e2) {
                        } else {
                            return Err(15);
                        }
                    }
                }
                return Err(-1);
            }
        },
        Exp::Negative(exp) => {
            let val = eval_exp(exp, idents)?;
            return Ok(0 - val);
        }
    }
}
