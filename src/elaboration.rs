use std::{iter, process::exit};

use ast::{Exp, Type};

use crate::ast::{self, Asnop, Binop, Call, Lvalue, Simp, Statement};
#[derive(Debug, Clone)]
pub enum Abs<'a> {
    ASGN(&'a [u8], Exp<'a>),
    WHILE(Exp<'a>, Box<Abs<'a>>),
    CONT,
    RET(Exp<'a>),
    DECL(&'a [u8], Type, Box<Abs<'a>>),
    IF(Exp<'a>, Box<Abs<'a>>, Box<Abs<'a>>),
    FOR(Box<Abs<'a>>),
    BRK,
    SEQ(Vec<Abs<'a>>),
    EXP(Exp<'a>),
    CALL(&'a [u8], Vec<Exp<'a>>),
}
fn translate_simpopt<'a>(simpopt: Option<Simp<'a>>) -> Abs<'a> {
    match simpopt {
        None => Abs::SEQ(vec![]),
        Some(simp) => match simp {
            ast::Simp::Simp((l, a, e)) => Abs::ASGN(l.get_ident_lvalue(), map_asnop(l, a, e)),
            ast::Simp::Decl(decl) => match decl {
                ast::Decl::Declare(typ, name) => Abs::DECL(name, typ, Box::new(Abs::SEQ(vec![]))),
                ast::Decl::Assign((typ, name, exp)) => {
                    Abs::DECL(name, typ, Box::new(Abs::SEQ(vec![Abs::ASGN(name, exp)])))
                }
            },
            ast::Simp::Call(call) => match call {
                Call::Print(arg_list) => Abs::CALL(b"print", arg_list.into_args()),
                Call::Read(_) => Abs::CALL(b"read", vec![]),
                Call::Flush(_) => Abs::CALL(b"flush", vec![]),
                Call::Func(name, arg_list) => Abs::CALL(name, arg_list.into_args()),
            },
        },
    }
}

pub fn translate_statement<'a>(
    stmts: &mut std::iter::Peekable<impl Iterator<Item = Statement<'a>>>,
) -> Abs<'a> {
    match stmts.next() {
        None => Abs::SEQ(vec![]),
        Some(s) => match s {
            Statement::Simp(simp) => match simp {
                ast::Simp::Simp((l, a, e)) => Abs::ASGN(l.get_ident_lvalue(), map_asnop(l, a, e)),
                ast::Simp::Decl(decl) => match decl {
                    ast::Decl::Declare(typ, name) => {
                        let mut vec = Vec::new();
                        while stmts.peek().is_some() {
                            vec.push(translate_statement(stmts));
                        }
                        Abs::DECL(name, typ, Box::new(Abs::SEQ(vec)))
                    }
                    ast::Decl::Assign((typ, name, exp)) => {
                        let mut vec = vec![Abs::ASGN(name, exp)];
                        while stmts.peek().is_some() {
                            vec.push(translate_statement(stmts));
                        }
                        Abs::DECL(name, typ, Box::new(Abs::SEQ(vec)))
                    }
                },
                ast::Simp::Call(call) => match call {
                    Call::Print(arg_list) => Abs::CALL(b"print", arg_list.into_args()),
                    Call::Read(_) => Abs::CALL(b"read", vec![]),
                    Call::Flush(_) => Abs::CALL(b"flush", vec![]),
                    Call::Func(name, arg_list) => Abs::CALL(name, arg_list.into_args()),
                },
            },
            Statement::Control(control) => match *control {
                ast::Control::If(exp, statement, statement2) => match statement2 {
                    Some(s) => Abs::IF(
                        exp,
                        Box::new(translate_statement(&mut iter::once(statement).peekable())),
                        Box::new(translate_statement(&mut iter::once(s).peekable())),
                    ),
                    None => Abs::IF(
                        exp,
                        Box::new(translate_statement(&mut iter::once(statement).peekable())),
                        Box::new(Abs::SEQ(vec![])),
                    ),
                },
                ast::Control::While(exp, statement) => Abs::WHILE(
                    exp,
                    Box::new(translate_statement(&mut iter::once(statement).peekable())),
                ),
                ast::Control::For((simp1, exp, simp2), statement) => {
                    let step = translate_simpopt(simp2);
                    if matches!(step, Abs::DECL(..)) {
                        println!(
                            "Error: The step statememt in a for loop cannot be a declaration."
                        );
                        exit(7);
                    }
                    let exp_asb = Abs::EXP(exp);
                    let mut for_loop = Abs::SEQ(vec![]);
                    let initializer = translate_simpopt(simp1);
                    match initializer {
                        Abs::DECL(items, typ, scope) => {
                            if let Abs::SEQ(vec) = *scope {
                                let mut new_vec = vec.clone();
                                new_vec.push(exp_asb);
                                let body =
                                    translate_statement(&mut iter::once(statement).peekable());
                                if let Abs::SEQ(mut statements) = body {
                                    new_vec.append(&mut statements);
                                } else {
                                    new_vec.push(body);
                                }
                                new_vec.push(step);
                                for_loop = Abs::DECL(items, typ, Box::new(Abs::SEQ(new_vec)));
                            }
                        }
                        _ => {
                            let mut new_vec = vec![initializer, exp_asb];
                            let body = translate_statement(&mut iter::once(statement).peekable());
                            if let Abs::SEQ(mut statements) = body {
                                new_vec.append(&mut statements);
                            } else {
                                new_vec.push(body);
                            }

                            new_vec.push(step);
                            for_loop = Abs::SEQ(new_vec);
                        }
                    };
                    Abs::FOR(Box::new(for_loop))
                }
                ast::Control::Continue => Abs::CONT,
                ast::Control::Break => Abs::BRK,
                ast::Control::Return(exp) => Abs::RET(exp),
            },
            Statement::Block(block) => {
                let mut statements = block.into_statements().into_iter().peekable();
                let mut instructions = Vec::new();
                while statements.peek().is_some() {
                    instructions.push(translate_statement(&mut statements));
                }
                Abs::SEQ(instructions)
            }
        },
    }
}

fn map_asnop<'a>(lvalue: Lvalue<'a>, asnop: Asnop, exp: Exp<'a>) -> Exp<'a> {
    match asnop {
        Asnop::APlus => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::Plus,
            exp,
        ))),
        Asnop::AMinus => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::Minus,
            exp,
        ))),
        Asnop::ADiv => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::Div,
            exp,
        ))),
        Asnop::AMult => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::Mult,
            exp,
        ))),
        Asnop::AMod => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::Mod,
            exp,
        ))),
        Asnop::Assign => exp,
        Asnop::ABitOr => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::BitOr,
            exp,
        ))),
        Asnop::ABitAnd => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::BitAnd,
            exp,
        ))),
        Asnop::ABitXor => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::BitXor,
            exp,
        ))),
        Asnop::ALShift => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::LShift,
            exp,
        ))),
        Asnop::ARShift => Exp::Arithmetic(Box::new((
            Exp::Ident(lvalue.get_ident_lvalue()),
            Binop::RShift,
            exp,
        ))),
    }
}
