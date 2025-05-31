use std::collections::{HashMap, HashSet};

use crate::{ast::Exp, elaboration::Abs};

pub fn return_check<'a>(s: Abs<'a>) -> bool {
    match s {
        Abs::RET(_) => true,
        Abs::DECL(_, _, seq) => return_check(*seq),
        Abs::IF(_, abs1, abs2) => return_check(*abs1) && return_check(*abs2),
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
    abs: Abs<'a>,
    assigned: &mut Vec<&'a [u8]>,
    declared: &mut Vec<&'a [u8]>,
) -> bool {
    match abs {
        Abs::ASGN(name, exp) => {
            if declared.contains(&name) && is_contained(&exp, assigned) {
                if !assigned.contains(&name) {
                    assigned.push(name);
                }
                return true;
            }
            false
        }
        Abs::WHILE(exp, abs) => {
            if is_contained(&exp, assigned) {
                return decl_check(*abs, assigned, declared);
            }
            false
        }
        Abs::CONT => true,
        Abs::RET(exp) => is_contained(&exp, assigned),
        Abs::DECL(name, _, abs) => {
            if declared.contains(&name) {
                return false;
            }
            declared.push(name);
            if decl_check(*abs, assigned, declared) {
                declared.remove(declared.iter().position(|n| n == &name).unwrap());
                if let Some(index) = assigned.iter().position(|x| x == &name) {
                    assigned.remove(index);
                }
                return true;
            }
            false
        }
        Abs::IF(exp, abs1, abs2) => {
            let return_exp = is_contained(&exp, assigned);
            let return_then = decl_check(*abs1, assigned, declared);
            let after_then = assigned.clone();
            let return_else = decl_check(*abs2, assigned, declared);
            assigned.retain(|x| after_then.contains(x));
            return_exp && return_then && return_else
        }
        Abs::FOR(abs, exp, abs1, abs2) => {
            is_contained(&exp, assigned)
                && decl_check(*abs, assigned, declared)
                && decl_check(*abs1, assigned, declared)
                && decl_check(*abs2, assigned, declared)
        }
        Abs::BRK => true,
        Abs::SEQ(items) => {
            for abs in items {
                if !decl_check(abs, assigned, declared) {
                    return false;
                }
            }
            true
        }
    }
}
