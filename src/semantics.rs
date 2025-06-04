use std::collections::HashMap;

use crate::{
    ast::{Binop, Exp, Type},
    elaboration::Abs,
};

pub fn return_check<'a>(s: &Abs<'a>) -> bool {
    match s {
        Abs::RET(_) => true,
        Abs::DECL(_, _, seq) => return_check(seq),
        Abs::IF(_, abs1, abs2) => return_check(abs1) && return_check(abs2),
        Abs::SEQ(items) => {
            for s in items {
                if return_check(s) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

fn is_contained<'a>(e: &Exp<'a>, vec: &mut Vec<&'a [u8]>) -> bool {
    match e {
        Exp::Ident(ident) => vec.contains(ident),
        Exp::Arithmetic(exps) => is_contained(&exps.0, vec) && is_contained(&exps.2, vec),
        Exp::Negative(exp) => is_contained(exp, vec),
        Exp::Not(exp) => is_contained(exp, vec),
        Exp::BitNot(exp) => is_contained(exp, vec),
        Exp::Ternary(exps) => {
            is_contained(&exps.0, vec) && is_contained(&exps.1, vec) && is_contained(&exps.2, vec)
        }
        _ => true,
    }
}

pub fn decl_check<'a>(
    abs: &Abs<'a>,
    assigned: &mut Vec<&'a [u8]>,
    declared: &mut Vec<&'a [u8]>,
) -> bool {
    match abs {
        Abs::ASGN(name, exp) => {
            if declared.contains(name) && is_contained(exp, assigned) {
                if !assigned.contains(name) {
                    assigned.push(name);
                }
                return true;
            }
            false
        }
        Abs::WHILE(exp, abs) => {
            if is_contained(exp, assigned) {
                return decl_check(abs, assigned, declared);
            }
            false
        }
        Abs::CONT => true,
        Abs::RET(exp) => is_contained(exp, assigned),
        Abs::DECL(name, _, abs) => {
            if declared.contains(name) {
                return false;
            }
            declared.push(name);
            if decl_check(abs, assigned, declared) {
                declared.remove(declared.iter().position(|n| n == name).unwrap());
                if let Some(index) = assigned.iter().position(|x| x == name) {
                    assigned.remove(index);
                }
                return true;
            }
            false
        }
        Abs::IF(exp, abs1, abs2) => {
            let return_exp = is_contained(exp, assigned);
            let return_then = decl_check(abs1, assigned, declared);
            let after_then = assigned.clone();
            let return_else = decl_check(abs2, assigned, declared);
            assigned.retain(|x| after_then.contains(x));
            return_exp && return_then && return_else
        }
        Abs::FOR(abs) => decl_check(abs, assigned, declared),
        Abs::BRK => true,
        Abs::SEQ(items) => {
            if let Some(pos) = items
                .iter()
                .rposition(|x| matches!(x, Abs::RET(..) | Abs::CONT | Abs::BRK))
            {
                let mut new_items = items.clone();
                let new_scope = new_items.split_off(pos + 1);
                for abs in new_items {
                    if !decl_check(&abs, assigned, declared) {
                        return false;
                    }
                }
                let mut new_assigned = declared.clone();
                decl_check(&Abs::SEQ(new_scope), &mut new_assigned, declared)
            } else {
                for abs in items {
                    if !decl_check(abs, assigned, declared) {
                        return false;
                    }
                }
                true
            }
        }
        Abs::EXP(exp) => is_contained(exp, assigned),
    }
}

fn type_check_exp(exp: &Exp, t: &Type, variables: &HashMap<&[u8], Type>) -> Result<Type, Type> {
    match exp {
        Exp::True => {
            if *t == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(Type::Int)
            }
        }
        Exp::False => {
            if *t == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(Type::Int)
            }
        }
        Exp::Intconst(_) => {
            if *t == Type::Int {
                Ok(Type::Int)
            } else {
                Err(Type::Bool)
            }
        }
        Exp::Ident(name) => {
            let ident_type = variables.get(name).unwrap().clone();
            if ident_type == *t {
                return Ok(ident_type);
            } else {
                return Err(ident_type);
            }
        }
        Exp::Arithmetic(b) => {
            let (e1, binop, e2) = &**b;
            if let Some(binop_type) = type_check_arithmetic(binop) {
                if let Err(e1_type) = type_check_exp(e1, &binop_type, variables) {
                    return Err(e1_type);
                }
                if let Err(e2_type) = type_check_exp(e2, &binop_type, variables) {
                    return Err(e2_type);
                }
                if binop_return_type(binop) == *t {
                    Ok(t.clone())
                } else {
                    Err(binop_return_type(binop))
                }
            } else {
                if let Ok(_) = type_check_exp(e1, &Type::Bool, variables) {
                    if let Ok(_) = type_check_exp(e2, &Type::Bool, variables) {
                        return Ok(Type::Bool);
                    } else {
                        return Err(Type::Int);
                    }
                } else if let Ok(_) = type_check_exp(e1, &Type::Int, variables) {
                    return Ok(Type::Bool);
                } else {
                    return Err(Type::Int);
                }
            }
        }
        Exp::Negative(exp) => type_check_exp(exp, &Type::Int, variables),
        Exp::Not(exp) => type_check_exp(exp, &Type::Bool, variables),
        Exp::BitNot(exp) => type_check_exp(exp, &Type::Int, variables),
        Exp::Ternary(b) => {
            let (e1, e2, e3) = &**b;
            let cond = type_check_exp(e1, &Type::Bool, variables);
            if cond.is_ok() {
                if type_check_exp(e2, &Type::Bool, variables).is_ok() {
                    if let Err(err) = type_check_exp(e3, &Type::Bool, variables) {
                        return Err(err);
                    } else {
                        return Ok(Type::Bool);
                    }
                } else if type_check_exp(e2, &Type::Int, variables).is_ok() {
                    if let Err(err) = type_check_exp(e3, &Type::Int, variables) {
                        return Err(err);
                    } else {
                        return Ok(Type::Int);
                    }
                }
            }
            cond
        }
    }
}

fn type_check_arithmetic(binop: &Binop) -> Option<Type> {
    match binop {
        Binop::Equals => None,
        Binop::NotEqual => None,
        Binop::And => Some(Type::Bool),
        Binop::Or => Some(Type::Bool),
        _ => Some(Type::Int),
    }
}

fn binop_return_type(binop: &Binop) -> Type {
    match binop {
        Binop::Equals => Type::Bool,
        Binop::NotEqual => Type::Bool,
        Binop::And => Type::Bool,
        Binop::Or => Type::Bool,
        Binop::LessThan => Type::Bool,
        Binop::LessEqual => Type::Bool,
        Binop::GreaterThan => Type::Bool,
        Binop::GreaterEqual => Type::Bool,
        _ => Type::Int,
    }
}

pub fn type_check<'a>(
    return_type: &Type,
    abs: &Abs<'a>,
    variables: &mut HashMap<&'a [u8], Type>,
) -> bool {
    match abs {
        Abs::ASGN(name, exp) => {
            if let Err(t) = type_check_exp(exp, variables.get(name).unwrap(), &variables) {
                println!("Type Error: Wrong use of type {t:?} in expression {exp:?}");
                false
            } else {
                true
            }
        }
        Abs::WHILE(exp, statements) => {
            if type_check_exp(exp, &Type::Bool, &variables).is_err() {
                println!("Type Error: While condition {exp:?} should evaluate to bool");
                false
            } else {
                type_check(return_type, statements, variables)
            }
        }
        Abs::CONT => true,
        Abs::RET(exp) => {
            if let Err(err_type) = type_check_exp(exp, return_type, &variables) {
                println!(
                    "Type Error: Function should return {return_type:?} but it currently returns {err_type:?}"
                );
                false
            } else {
                true
            }
        }
        Abs::DECL(name, t, abs) => {
            variables.insert(name, t.clone());
            type_check(return_type, abs, variables)
        }
        Abs::IF(exp, abs1, abs2) => {
            if let Err(_) = type_check_exp(exp, &Type::Bool, &variables) {
                println!("Type Error: If condition need to evaluate to bool");
                false
            } else {
                type_check(return_type, abs1, variables) && type_check(return_type, abs2, variables)
            }
        }
        Abs::FOR(abs) => type_check(return_type, abs, variables),
        Abs::BRK => true,
        Abs::SEQ(items) => {
            for abs in items {
                if !type_check(return_type, abs, variables) {
                    return false;
                }
            }
            true
        }
        Abs::EXP(exp) => {
            if type_check_exp(exp, &Type::Bool, variables).is_err() {
                println!("Type Error: The for loops break condition should evaluate to bool");
                false
            } else {
                true
            }
        }
    }
}
