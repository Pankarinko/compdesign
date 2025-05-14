use crate::tokenizer::{self, ArithmeticSymbol, Keyword, Token};
#[derive(Debug)]
pub enum Statement<'a> {
    Decl(Decl<'a>),
    Simp(Simp<'a>),
    Exp(Exp<'a>),
}
#[derive(Debug)]
pub enum Decl<'a> {
    Declare,
    Assign(Exp<'a>),
}
#[derive(Debug)]
pub enum Simp<'a> {
    Simp((Lvalue<'a>, Asnop, Exp<'a>)),
}
#[derive(Debug)]
pub enum Lvalue<'a> {
    Ident(&'a [u8]),
    Paranth(Box<Lvalue<'a>>),
}

#[derive(Debug)]
pub enum StatementList<'a> {
    Empty,
    Statements((Statement<'a>, Box<StatementList<'a>>)),
}

#[derive(Debug)]
pub enum Exp<'a> {
    Intconst(i32),
    Ident(&'a [u8]),
    Arithmetic(Box<(Exp<'a>, Binop, Exp<'a>)>),
    Negative(Box<Exp<'a>>),
}
#[derive(Debug)]
pub enum Binop {
    Plus,
    Minus,
    Div,
    Mult,
    Mod,
}
#[derive(Debug)]
pub enum Asnop {
    APlus,
    AMinus,
    ADiv,
    AMult,
    AMod,
    Assign,
}
/*pub Program: Program<'a> {
    <stmts:Sta => Program::Prog(StatementList<'a>),
#[precedence(level="1")]  #[assoc(side="right")]
    "-" <Exp>, => Exp::Negative(Box::new()),
}*/
#[derive(Debug)]
pub enum Program<'a> {
    Prog(StatementList<'a>),
    Empty,
}
