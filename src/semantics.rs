use std::{collections::HashMap, process::id};

use crate::ast::{Asnop, Exp, Simp, Statement};

pub fn return_check(statements: &Vec<Statement<'_>>) -> bool {
    return statements.iter().any(|s| match s {
        Statement::Return(_) => return true,
        _ => return false,
    });
}

fn get_ident<'a>(e: &Exp<'a>, index: usize, occur: &mut HashMap<&'a [u8], usize>) {
    match e {
        Exp::Ident(ident) => {
            occur.entry(ident).or_insert(index);
            ()
        }
        Exp::Arithmetic(exps) => {
            get_ident(&(*exps).0, index, occur);
            get_ident(&(*exps).2, index, occur);
        }
        Exp::Negative(exp) => get_ident(exp, index, occur),
        Exp::Intconst(_) => (),
    }
}

fn is_contained<'a>(e: &Exp<'a>, vec: &mut Vec<&'a [u8]>) -> bool {
    match e {
        Exp::Ident(ident) => vec.contains(ident),
        Exp::Arithmetic(exps) => is_contained(&(*exps).0, vec) && is_contained(&(*exps).2, vec),
        Exp::Negative(exp) => is_contained(exp, vec),
        Exp::Intconst(_) => true,
    }
}

pub fn decl_check<'a>(
    stmt: &Statement<'a>,
    decls: &mut Vec<&'a [u8]>,
    assignments: &mut Vec<&'a [u8]>,
) -> bool {
    match stmt {
        Statement::Decl(decl) => match decl {
            crate::ast::Decl::Declare(ident) => {
                if decls.contains(&ident) || assignments.contains(&ident) {
                    return false;
                };
                decls.push(ident);
            }
            crate::ast::Decl::Assign(a) => {
                if decls.contains(&a.0) || assignments.contains(&a.0) {
                    return false;
                }
                assignments.push(a.0);
                let e = &a.1;
                if !is_contained(&e, assignments) {
                    return false;
                };
            }
        },
        Statement::Simp(simp) => match simp {
            Simp::Simp((lval, asnop, exp)) => {
                let ident = lval.get_ident_lvalue();
                match asnop {
                    Asnop::Assign => {
                        if !decls.contains(&ident) && !assignments.contains(&ident) {
                            return false;
                        } else if !is_contained(&exp, assignments) {
                            return false;
                        }
                        if !assignments.contains(&ident) {
                            assignments.push(&ident);
                        }
                    }
                    _ => {
                        if !assignments.contains(&ident) {
                            return false;
                        }
                    }
                };
                if !is_contained(&exp, assignments) {
                    return false;
                };
            }
        },
        Statement::Return(exp) => {
            if !is_contained(&exp, assignments) {
                return false;
            }
        }
    }
    return true;
}
