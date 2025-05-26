use ast::{Exp, Type};

use crate::ast::{self, Binop, Block, Statement};
#[derive(Debug)]
pub enum Abs<'a> {
    ASGN(&'a [u8], Exp<'a>),
    WHILE(Exp<'a>, Box<Abs<'a>>),
    CONT,
    RET(Exp<'a>),
    DECL(&'a [u8], Type, Box<Abs<'a>>),
    IF(Exp<'a>, Box<Abs<'a>>, Box<Abs<'a>>),
    FOR(Box<Abs<'a>>, Exp<'a>, Box<Abs<'a>>, Box<Abs<'a>>),
    BRK,
    SEQ(Vec<Abs<'a>>),
}

fn dedangle_else<'a>(block: &mut Block<'a>) {
    for (i, s) in stmts.into_iter().enumerate() {
        match s {
            Statement::Control(control) => { match *control {
                ast::Control::Else(statement) => {
                    match stmts[i+1] {
                        Statement::Control(control) => match *control  {
                            ast::Control::Else(else_stmt) => {
                                instructions.push(Abs::IF(exp, statement, else_stmt)); 
                                continue;},
                            _ => (),
                        },
                        _ => ()
                    }
                },
                _ => (),
            }
        },
        Statement::Block(block) => dedangle_else(&mut block),
        _ => (),
    }    }
}

fn translate_block<'a>(stmts: Vec<Statement<'a>>) -> Vec<Abs<'a>> {
    let mut instructions:  Vec<Abs<'a>> = Vec::new();
    for (i, s) in stmts.into_iter().enumerate() {
        match s {
            Statement::Simp(simp) => todo!(),
            Statement::Control(control) => { match *control {
                ast::Control::If(exp, statement) => {
                    match stmts[i+1] {
                        Statement::Control(control) => match *control  {
                            ast::Control::Else(else_stmt) => {
                                instructions.push(Abs::IF(exp, statement, else_stmt)); 
                                continue;},
                            _ => (),
                        },
                        _ => instructions.push(Abs::IF(exp, statement, Abs::SEQ(vec![])))
                    }
                },
                ast::Control::Else(statement) => continue,
                ast::Control::While(exp, statement) => instructions.push(Abs::WHILE(exp, )),
                ast::Control::For(_, statement) => todo!(),
                ast::Control::Continue => instructions.push(Abs::CONT),
                ast::Control::Break => instructions.push(Abs::BRK),
                ast::Control::Return(exp) => instructions.push(Abs::RET(exp)),
            }},
            Statement::Block(block) => instructions.push(Abs::SEQ(translate_block(block.get_statements()))),
            }
        }
        instructions
    }
    


pub fn translate_statement<'a>(s: Statement<'a>, instructions: &mut Vec<Abs<'a>>) {
    match s {
        Statement::Simp(simp) => match simp {
            ast::Simp::Simp((lv, asnop, exp)) => Abs::ASGN(lv.get_ident_lvalue(), Exp(lv.get_ident_lvalue(), )
            ast::Simp::Decl(decl) => todo!(),
        },
        Statement::Control(control) => todo!(),
        Statement::Block(block) => todo!(),
    }
}

fn map_binop(asnop: Asnop) {
    match binop {
        Binop::Plus => todo!(),
        Binop::Minus => todo!(),
        Binop::Div => todo!(),
        Binop::Mult => todo!(),
        Binop::Mod => todo!(),
        Binop::LessThan => todo!(),
        Binop::LessEqual => todo!(),
        Binop::GreaterThan => todo!(),
        Binop::GreaterEqual => todo!(),
        Binop::Equals => todo!(),
        Binop::NotEqual => todo!(),
        Binop::And => todo!(),
        Binop::Or => todo!(),
        Binop::BitAnd => todo!(),
        Binop::BitXor => todo!(),
        Binop::BitOr => todo!(),
        Binop::LShift => todo!(),
        Binop::RShift => todo!(),
    }

}
