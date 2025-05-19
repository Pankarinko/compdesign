use crate::ast::{Asnop, Exp, Simp, Statement};

pub fn return_check(statements: &Vec<Statement<'_>>) -> bool {
    statements.iter().any(|s| matches!(s, Statement::Return(_)))
}

fn is_contained<'a>(e: &Exp<'a>, vec: &mut Vec<&'a [u8]>) -> bool {
    match e {
        Exp::Ident(ident) => vec.contains(ident),
        Exp::Arithmetic(exps) => is_contained(&exps.0, vec) && is_contained(&exps.2, vec),
        Exp::Negative(exp) => is_contained(exp, vec),
        Exp::Intconst(_) => true,
    }
}

pub fn decl_check<'a>(statements: &'a Vec<Statement<'a>>) -> bool {
    let mut decls: Vec<&'a [u8]> = Vec::new();
    let mut assignments: Vec<&'a [u8]> = Vec::new();
    for stmt in statements.iter() {
        match stmt {
            Statement::Decl(decl) => match decl {
                crate::ast::Decl::Declare(ident) => {
                    if decls.contains(ident) || assignments.contains(ident) {
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
                    if !is_contained(e, &mut assignments) {
                        return false;
                    };
                }
            },
            Statement::Simp(simp) => match simp {
                Simp::Simp((lval, asnop, exp)) => {
                    let ident = lval.get_ident_lvalue();
                    match asnop {
                        Asnop::Assign => {
                            if (!decls.contains(&ident) && !assignments.contains(&ident))
                                || !is_contained(exp, &mut assignments)
                            {
                                return false;
                            }
                            if !assignments.contains(&ident) {
                                assignments.push(ident);
                            }
                        }
                        _ => {
                            if !assignments.contains(&ident) {
                                return false;
                            }
                        }
                    };
                    if !is_contained(exp, &mut assignments) {
                        return false;
                    };
                }
            },
            Statement::Return(exp) => {
                if !is_contained(exp, &mut assignments) {
                    return false;
                }
            }
        }
    }
    true
}
