#[derive(Debug)]
pub enum Statement<'a> {
    Decl(Decl<'a>),
    Simp(Simp<'a>),
    Return(Exp<'a>),
}

#[derive(Debug)]
pub enum Decl<'a> {
    Declare(&'a [u8]),
    Assign((&'a [u8], Exp<'a>)),
}
#[derive(Debug)]
pub enum Simp<'a> {
    Simp((Lvalue<'a>, Asnop, Exp<'a>)),
}

#[derive(Debug)]
pub enum Lvalue<'a> {
    Ident(&'a [u8]),
    Parenth(Box<Lvalue<'a>>),
}

impl<'a> Lvalue<'a> {
    pub fn get_ident_lvalue(&'a self) -> &'a [u8] {
        match self {
            Lvalue::Ident(ident) => return ident,
            Lvalue::Parenth(lvalue) => return (*lvalue).get_ident_lvalue(),
        }
    }
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

#[derive(Debug)]
pub enum Program<'a> {
    Prog(Vec<Statement<'a>>),
}

impl<'a> Program<'a> {
    pub fn get_statements(&'a self) -> &'a Vec<Statement<'a>> {
        match self {
            Program::Prog(statements) => return &statements,
        }
    }
}
