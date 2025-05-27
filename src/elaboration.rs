use ast::{Exp, Type};

use crate::ast::{self, Asnop, Binop, Block, Lvalue, Simp, SimpOpt, Statement};
#[derive(Debug)]
pub enum Abs<'a> {
    ASGN(&'a [u8], Exp<'a>),
    WHILE(Exp<'a>, Box<Abs<'a>>),
    CONT,
    RET(Exp<'a>),
    DECL(&'a [u8], Type, Abs<'a>),
    IF(Exp<'a>, Box<Abs<'a>>, Box<Abs<'a>>),
    FOR(Box<Abs<'a>>, Exp<'a>, Box<Abs<'a>>, Box<Abs<'a>>),
    BRK,
    SEQ(Vec<Abs<'a>>),
}
fn translate_simpopt<'a>(simpopt: SimpOpt<'a>) -> Abs<'a> {
    match simpopt {
        SimpOpt::NoSimp => Abs::SEQ(vec![]),
        SimpOpt::Simp(simp) => match simp {
            ast::Simp::Simp((l, a, e)) => Abs::ASGN(l, map_asnop(lvalue, asnop, exp)),
            ast::Simp::Decl(decl) => match decl {
                ast::Decl::Declare(typ, name) => Abs::DECL(name, typ, Abs::SEQ(vec![])),
                ast::Decl::Assign((typ, name, exp)) => {
                    Abs::DECL(typ, name, Abs::SEQ(vec![Abs::ASGN(name, exp)]))
                }
            },
        },
    }
}

pub fn translate_statement<'a>(stmts: Iterator<Statement<'a>>) -> Abs<'a> {
    let mut instructions = Vec::new();
    match stmts.next() {
        Statement::Simp(simp) => match simp {
            ast::Simp::Simp((l, a, e)) => Abs::ASGN(l, map_asnop(lvalue, asnop, exp)),
            ast::Simp::Decl(decl) => match decl {
                ast::Decl::Declare(typ, name) => {
                    Abs::DECL(name, typ, Abs::SEQ(translate_statement(stmts)))
                }
                ast::Decl::Assign((typ, name, exp)) => instructions.push(Abs::DECL(
                    typ,
                    name,
                    Abs::SEQ(vec![Abs::ASGN(name, exp)].append(&mut translate_statement(stmts))),
                )),
            },
            Statement::Control(control) => match control {
                ast::Control::If(exp, statement, statement2) => match statement2 {
                    Some(s) => Abs::IF(exp, statement, s),
                    None => Abs::IF(exp, statement, Abs::SEQ(vec![])),
                },
                ast::Control::While(exp, statement) => Abs::WHILE(exp, vec![statement].into_iter()),
                ast::Control::For((simp1, exp, simp2), statement) => Abs::FOR(
                    translate_simp(simp1),
                    exp,
                    translate_simpopt(simp2),
                    translate_statements(vec![statement].into_iter()),
                ),
                ast::Control::Continue => Abs::CONT,
                ast::Control::Break => Abs::BRK,
                ast::Control::Return(exp) => Abs::RET(exp),
            },
            Statement::Block(block) => instructions.push(Abs::SEQ(translate_statements(
                block.get_statements().into_iter(),
            ))),
        },
    }
}

fn map_asnop(lvalue: Lvalue<'_>, asnop: Asnop, exp: Exp<'_>) -> Exp<'_> {
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
