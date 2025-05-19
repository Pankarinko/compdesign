use std::{
    collections::HashMap,
    ffi::OsString,
    io::Write,
    process::{Command, Stdio},
};

use crate::ast::{self, Exp, Program, Simp, Statement};

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
        movq ${code}, %rax
        ret
"
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
                .write_all(file_string.as_bytes())
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
                .write_all(file_string.as_bytes())
                .expect("Failed to write to stdin");
            child.wait().expect("gcc couldn't finish execution");
        }
    }
}

fn eval_program<'a>(statements: &'a Vec<Statement<'a>>) -> Result<i32, i32> {
    let mut used_idents: HashMap<&'a [u8], i32> = HashMap::new();
    for stmt in statements.iter() {
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
        Statement::Simp(Simp::Simp((lvalue, asnop, exp))) => {
            let second_param = eval_exp(exp, idents)?;
            match asnop {
                ast::Asnop::APlus => {
                    let val = idents.get_mut(lvalue.get_ident_lvalue()).ok_or(-1)?;
                    *val = (*val).wrapping_add(second_param);
                }
                ast::Asnop::AMinus => {
                    let val = idents.get_mut(lvalue.get_ident_lvalue()).ok_or(-1)?;
                    *val = (*val).wrapping_sub(second_param);
                }
                ast::Asnop::ADiv => {
                    let val = idents.get_mut(lvalue.get_ident_lvalue()).ok_or(-1)?;
                    if let Some(res) = (*val).checked_div(second_param) {
                        *val = res;
                        return Ok(None);
                    }
                    return Err(-1);
                }
                ast::Asnop::AMult => {
                    let val = idents.get_mut(lvalue.get_ident_lvalue()).ok_or(-1)?;
                    *val = (*val).wrapping_mul(second_param);
                }
                ast::Asnop::AMod => {
                    let val = idents.get_mut(lvalue.get_ident_lvalue()).ok_or(-1)?;
                    if let Some(res) = (*val).checked_rem(second_param) {
                        *val = res;
                        return Ok(None);
                    }
                    return Err(-1);
                }
                ast::Asnop::Assign => {
                    let _mess = idents.insert(lvalue.get_ident_lvalue(), eval_exp(exp, idents)?);
                }
            }
            Ok(None)
        }
        Statement::Return(exp) => Ok(Some(eval_exp(exp, idents)?)),
    }
}

fn eval_exp<'a>(exp: &Exp<'a>, idents: &HashMap<&'a [u8], i32>) -> Result<i32, i32> {
    match exp {
        Exp::Intconst(c) => return Ok(*c),
        Exp::Ident(name) => idents.get(name).copied().ok_or(-1),
        Exp::Arithmetic(arith) => match arith.1 {
            ast::Binop::Plus => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        return Ok(e1.wrapping_add(e2));
                    }
                }
                Err(-1)
            }
            ast::Binop::Minus => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        return Ok(e1.wrapping_sub(e2));
                    }
                }
                Err(-1)
            }
            ast::Binop::Div => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        if let Some(res) = e1.checked_div(e2) {
                            return Ok(res);
                        } else {
                            return Err(15);
                        }
                    }
                }
                Err(-1)
            }
            ast::Binop::Mult => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        return Ok(e1.wrapping_mul(e2));
                    }
                }
                Err(-1)
            }
            ast::Binop::Mod => {
                if let Ok(e1) = eval_exp(&arith.0, idents) {
                    if let Ok(e2) = eval_exp(&arith.2, idents) {
                        if let Some(res) = e1.checked_rem(e2) {
                            return Ok(res);
                        } else {
                            return Err(15);
                        }
                    }
                }
                Err(-1)
            }
        },
        Exp::Negative(exp) => {
            let val = eval_exp(exp, idents)?;
            Ok(0 - val)
        }
    }
}
