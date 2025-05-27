#[derive(Debug)]
pub enum Statement<'a> {
    Simp(Simp<'a>),
    Control(Box<Control<'a>>),
    Block(Block<'a>),
}

#[derive(Debug)]
pub enum Type {
    Int,
    Bool,
}

#[derive(Debug)]
pub enum Decl<'a> {
    Declare(Type, &'a [u8]),
    Assign((Type, &'a [u8], Exp<'a>)),
}
#[derive(Debug)]
pub enum Simp<'a> {
    Simp((Lvalue<'a>, Asnop, Exp<'a>)),
    Decl(Decl<'a>),
}

#[derive(Debug)]
pub enum Lvalue<'a> {
    Ident(&'a [u8]),
}

impl<'a> Lvalue<'a> {
    pub fn get_ident_lvalue(&self) -> &'a [u8] {
        match self {
            Lvalue::Ident(ident) => ident,
        }
    }
}

#[derive(Debug)]
pub enum Control<'a> {
    If(Exp<'a>, Statement<'a>, Option<Statement<'a>>),
    While(Exp<'a>, Statement<'a>),
    For((Option<Simp<'a>>, Exp<'a>, Option<Simp<'a>>), Statement<'a>),
    Continue,
    Break,
    Return(Exp<'a>),
}

#[derive(Debug)]
pub enum Exp<'a> {
    True,
    False,
    Intconst(i32),
    Ident(&'a [u8]),
    Arithmetic(Box<(Exp<'a>, Binop, Exp<'a>)>),
    Negative(Box<Exp<'a>>),
    Not(Box<Exp<'a>>),
    BitNot(Box<Exp<'a>>),
    Ternary(Box<(Exp<'a>, Exp<'a>, Exp<'a>)>),
}

#[derive(Debug)]
pub enum Binop {
    Plus,
    Minus,
    Div,
    Mult,
    Mod,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Equals,
    NotEqual,
    And,
    Or,
    BitAnd,
    BitXor,
    BitOr,
    LShift,
    RShift,
}
#[derive(Debug)]
pub enum Asnop {
    APlus,
    AMinus,
    ADiv,
    AMult,
    AMod,
    Assign,
    ABitOr,
    ABitAnd,
    ABitXor,
    ALShift,
    ARShift,
}
#[derive(Debug)]
pub enum Program<'a> {
    Block(Block<'a>),
}

impl<'a> Program<'a> {
    pub fn get_statements(&'a self) -> &'a Vec<Statement<'a>> {
        match self {
            Program::Block(block) => block.get_statements(),
        }
    }
}

#[derive(Debug)]
pub enum Block<'a> {
    Block(Vec<Statement<'a>>),
}

impl<'a> Block<'a> {
    pub fn get_statements(&'a self) -> &'a Vec<Statement<'a>> {
        match self {
            Block::Block(statements) => &statements,
        }
    }
}
