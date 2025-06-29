use std::{
    collections::{HashMap, HashSet},
    iter,
    process::exit,
};

use crate::{
    ast::{Binop, Exp, Function, Param, Program, Type},
    elaboration::{Abs, translate_statement},
};

pub fn check_semantics<'a>(program: Program<'a>) -> Vec<Abs<'a>> {
    let funcs = program.into_functions();
    if !check_function_names(funcs) {
        exit(7);
    }
    check_function_semantics(funcs)
}

fn check_function_names(funcs: &Vec<Function>) -> bool {
    let mut names = HashSet::new();
    let mut main = false;
    for f in funcs.iter() {
        if !names.insert(f.get_name()) {
            println!(
                "Error: Function \"{}\" is declared more than once.",
                str::from_utf8(f.get_name()).unwrap()
            );
            return false;
        }
        if f.get_name() == b"main" {
            main = true;
        }
    }
    if !main {
        println!("Error: missing main function.");
        return false;
    }
    true
}

fn check_function_semantics<'a>(funcs: &Vec<Function<'a>>) -> Vec<Abs<'a>> {
    let mut func_params = HashMap::new();
    let mut abs_funcs = Vec::new();
    let _ = funcs
        .iter()
        .map(|f| func_params.insert(f.get_name(), (f.get_params(), f.get_type())));
    for f in funcs.iter() {
        let mut declared: Vec<&'a [u8]> = f.get_params().iter().map(|p| p.get_name()).collect();
        let mut assigned = declared.clone();
        let stmts = translate_statement(
            &mut f
                .clone()
                .get_block()
                .into_statements()
                .into_iter()
                .peekable(),
        );
        if !return_check(&stmts) {
            println!(
                "Error: Function \"{}\" does not return.",
                str::from_utf8(f.get_name()).unwrap()
            );
            exit(7)
        }
        if !decl_check(&stmts, &mut assigned, &mut declared) {
            println!(
                "Error: Function \"{}\" has undeclared or unassigned variables.",
                str::from_utf8(f.get_name()).unwrap()
            );
            exit(7)
        }
        let mut variables = HashMap::new();
        let _ = f
            .get_params()
            .iter()
            .map(|p| variables.insert(p.get_name(), p.get_type().clone()));
        println!("{:?}", f.get_params());
        if !type_check(f.get_type(), &stmts, &func_params, &mut variables) {
            exit(7);
        }
        let loop_counter = 0;
        if !break_coninue_check(loop_counter, &stmts) {
            println!("Error: Break and continue found outside of loop.");
            exit(7)
        }
        abs_funcs.push(stmts);
    }
    abs_funcs
}

fn arg_type_check<'a>(
    f_name: &'a [u8],
    func_params: &HashMap<&'a [u8], (&Vec<Param<'a>>, &Type)>,
    args: Vec<Exp<'a>>,
    variables: &HashMap<&[u8], Type>,
) -> bool {
    if let Some(f_args) = func_params.get(f_name) {
        if f_args.0.len() == args.len() {
            let res = f_args.0.iter().enumerate().any(|(i, param)| {
                type_check_exp(&args[i], param.get_type(), func_params, variables).is_err()
            });
            if res {
                println!(
                    "Error: Function \"{}\" was called with parameters of the wrong type.",
                    str::from_utf8(f_name).unwrap(),
                );
                false
            } else {
                true
            }
        } else {
            println!(
                "Error: Function \"{}\" takes {} arguments, but {} were provided.",
                str::from_utf8(f_name).unwrap(),
                f_args.0.len(),
                args.len(),
            );
            false
        }
    } else {
        println!(
            "Error: No function with name \"{}\" found.",
            str::from_utf8(f_name).unwrap()
        );
        false
    }
}

fn return_check<'a>(s: &Abs<'a>) -> bool {
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

fn decl_check<'a>(
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
                let mut temp_assigned = assigned.clone();
                return decl_check(abs, &mut temp_assigned, declared);
            }
            false
        }
        Abs::CONT => {
            *assigned = declared.clone();
            true
        }
        Abs::RET(exp) => {
            let res = is_contained(exp, assigned);
            *assigned = declared.clone();
            res
        }
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
            let mut temp_assigned = assigned.clone();
            let return_then = decl_check(abs1, &mut temp_assigned, declared);
            let return_else = decl_check(abs2, assigned, declared);
            assigned.retain(|x| temp_assigned.contains(x));
            return_exp && return_then && return_else
        }
        Abs::FOR(abs) => {
            if let Abs::SEQ(vec) = &**abs {
                let mut temp_assigned = assigned.clone();
                if matches!(vec[0], Abs::ASGN(..)) {
                    decl_check(&vec[0], assigned, declared);
                }
                return decl_check(abs, &mut temp_assigned, declared);
            }
            decl_check(abs, assigned, declared)
        }
        Abs::BRK => {
            *assigned = declared.clone();
            true
        }
        Abs::SEQ(items) => {
            for abs in items {
                if !decl_check(abs, assigned, declared) {
                    return false;
                }
            }
            true
        }
        Abs::EXP(exp) => is_contained(exp, assigned),
        Abs::CALL(_, exps) => {
            for abs in exps {
                if !is_contained(abs, assigned) {
                    return false;
                }
            }
            true
        }
    }
}

fn break_coninue_check(counter: usize, abs: &Abs) -> bool {
    match abs {
        Abs::ASGN(..) => true,
        Abs::WHILE(_, abs) => break_coninue_check(counter + 1, abs),
        Abs::CONT => counter > 0,
        Abs::RET(_) => true,
        Abs::DECL(_, _, abs) => break_coninue_check(counter, abs),
        Abs::IF(_, abs1, abs2) => {
            break_coninue_check(counter, abs1) && break_coninue_check(counter, abs2)
        }
        Abs::FOR(abs) => break_coninue_check(counter + 1, abs),
        Abs::BRK => counter > 0,
        Abs::SEQ(items) => {
            for abs in items.iter() {
                if matches!(abs, Abs::BRK | Abs::CONT | Abs::RET(_)) {
                    return break_coninue_check(counter, abs);
                } else if !break_coninue_check(counter, abs) {
                    return false;
                }
            }
            true
        }
        Abs::EXP(..) => true,
        Abs::CALL(_items, _exps) => true,
    }
}
fn type_check_exp<'a>(
    exp: &Exp,
    t: &Type,
    func_params: &HashMap<&'a [u8], (&Vec<Param<'a>>, &Type)>,
    variables: &HashMap<&[u8], Type>,
) -> Result<Type, Type> {
    match exp {
        Exp::True => {
            if *t == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(Type::Bool)
            }
        }
        Exp::False => {
            if *t == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(Type::Bool)
            }
        }
        Exp::Intconst(_) => {
            if *t == Type::Int {
                Ok(Type::Int)
            } else {
                Err(Type::Int)
            }
        }
        Exp::Ident(name) => {
            println!("{:?}", name);
            let ident_type = variables.get(name).unwrap().clone();
            if ident_type == *t {
                Ok(ident_type)
            } else {
                Err(ident_type)
            }
        }
        Exp::Arithmetic(b) => {
            let (e1, binop, e2) = &**b;

            if let Some(binop_type) = type_check_arithmetic(binop) {
                type_check_exp(e1, &binop_type, func_params, variables)?;
                type_check_exp(e2, &binop_type, func_params, variables)?;
            } else if type_check_exp(e1, &Type::Bool, func_params, variables).is_ok() {
                if type_check_exp(e2, &Type::Bool, func_params, variables).is_err() {
                    return Err(Type::Int);
                }
            } else if type_check_exp(e2, &Type::Int, func_params, variables).is_err() {
                return Err(Type::Bool);
            }
            if binop_return_type(binop) == *t {
                Ok(t.clone())
            } else {
                Err(binop_return_type(binop))
            }
        }
        Exp::Negative(exp) => type_check_exp(exp, &Type::Int, func_params, variables),
        Exp::Not(exp) => type_check_exp(exp, &Type::Bool, func_params, variables),
        Exp::BitNot(exp) => type_check_exp(exp, &Type::Int, func_params, variables),
        Exp::Ternary(b) => {
            let (e1, e2, e3) = &**b;
            type_check_exp(e1, &Type::Bool, func_params, variables)?;
            type_check_exp(e2, t, func_params, variables)?;
            if let Err(err) = type_check_exp(e3, t, func_params, variables) {
                Err(err)
            } else {
                Ok(Type::Bool)
            }
        }
        Exp::Call(call) => match call {
            crate::ast::Call::Print(arg_list) => todo!(),
            crate::ast::Call::Read(arg_list) => todo!(),
            crate::ast::Call::Flush(arg_list) => todo!(),
            crate::ast::Call::Func(name, arg_list) => {
                if let Some(data) = func_params.get(name) {
                    if data.1 == t {
                        return Ok(data.1.clone());
                    } else {
                        return Err(data.1.clone());
                    }
                }
                println!(
                    "Error: No function with name \"{}\" found.",
                    str::from_utf8(name).unwrap()
                );
                exit(7);
            }
        },
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

fn type_check<'a>(
    return_type: &Type,
    abs: &Abs<'a>,
    func_params: &HashMap<&'a [u8], (&Vec<Param<'a>>, &Type)>,
    variables: &mut HashMap<&'a [u8], Type>,
) -> bool {
    match abs {
        Abs::ASGN(name, exp) => {
            if let Err(t) =
                type_check_exp(exp, variables.get(name).unwrap(), func_params, variables)
            {
                println!("Type Error: Wrong use of type {t:?} in expression {exp:?}");
                false
            } else {
                true
            }
        }
        Abs::WHILE(exp, statements) => {
            if type_check_exp(exp, &Type::Bool, func_params, variables).is_err() {
                println!("Type Error: While condition {exp:?} should evaluate to bool");
                false
            } else {
                type_check(return_type, statements, func_params, variables)
            }
        }
        Abs::CONT => true,
        Abs::RET(exp) => {
            if let Err(err_type) = type_check_exp(exp, return_type, func_params, variables) {
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
            type_check(return_type, abs, func_params, variables)
        }
        Abs::IF(exp, abs1, abs2) => {
            if type_check_exp(exp, &Type::Bool, func_params, variables).is_err() {
                println!("Type Error: If condition need to evaluate to bool");
                false
            } else {
                type_check(return_type, abs1, func_params, variables)
                    && type_check(return_type, abs2, func_params, variables)
            }
        }
        Abs::FOR(abs) => type_check(return_type, abs, func_params, variables),
        Abs::BRK => true,
        Abs::SEQ(items) => {
            for abs in items {
                if !type_check(return_type, abs, func_params, variables) {
                    return false;
                }
            }
            true
        }
        Abs::EXP(exp) => {
            if type_check_exp(exp, &Type::Bool, func_params, variables).is_err() {
                println!("Type Error: The for loops break condition should evaluate to bool");
                false
            } else {
                true
            }
        }
        Abs::CALL(name, args) => arg_type_check(name, func_params, args.clone(), variables),
    }
}
