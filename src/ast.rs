#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Simp(Simp<'a>),
    Control(Box<Control<'a>>),
    Block(Block<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Int,
    Bool,
}

#[derive(Debug, Clone)]
pub enum Decl<'a> {
    Declare(Type, &'a [u8]),
    Assign((Type, &'a [u8], Exp<'a>)),
}
#[derive(Debug, Clone)]
pub enum Simp<'a> {
    Simp((Lvalue<'a>, Asnop, Exp<'a>)),
    Decl(Decl<'a>),
    Call(Call<'a>),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Control<'a> {
    If(Exp<'a>, Statement<'a>, Option<Statement<'a>>),
    While(Exp<'a>, Statement<'a>),
    For((Option<Simp<'a>>, Exp<'a>, Option<Simp<'a>>), Statement<'a>),
    Continue,
    Break,
    Return(Exp<'a>),
}

#[derive(Debug, Clone)]
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
    Call(Call<'a>),
}

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum Program<'a> {
    Program(Vec<Function<'a>>),
}

impl<'a> Program<'a> {
    pub fn into_functions(&self) -> &Vec<Function<'a>> {
        match self {
            Program::Program(func) => func,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Function<'a> {
    Function(Type, &'a [u8], ParamList<'a>, Block<'a>),
}

impl<'a> Function<'a> {
    pub fn get_params(&self) -> &Vec<Param<'a>> {
        match self {
            Function::Function(_, _, ParamList::ParamList(params), _) => params,
        }
    }

    pub fn get_block(self) -> Block<'a> {
        match self {
            Function::Function(_, _, _, block) => block,
        }
    }

    pub fn get_name(&self) -> &'a [u8] {
        match self {
            Function::Function(_, name, _, _) => name,
        }
    }

    pub fn get_type(&self) -> &Type {
        match self {
            Function::Function(t, _, _, _) => t,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParamList<'a> {
    ParamList(Vec<Param<'a>>),
}

#[derive(Debug, Clone)]
pub enum Param<'a> {
    Param(Type, &'a [u8]),
}

impl<'a> Param<'a> {
    pub fn get_name(&self) -> &'a [u8] {
        match self {
            Param::Param(_, name) => name,
        }
    }

    pub fn get_type(&self) -> &Type {
        match self {
            Param::Param(t, _) => t,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Block<'a> {
    Block(Vec<Statement<'a>>),
}

impl<'a> Block<'a> {
    pub fn into_statements(self) -> Vec<Statement<'a>> {
        match self {
            Block::Block(statements) => statements,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Call<'a> {
    Print(ArgList<'a>),
    Read(ArgList<'a>),
    Flush(ArgList<'a>),
    Func(&'a [u8], ArgList<'a>),
}

#[derive(Debug, Clone)]
pub enum ArgList<'a> {
    Args(Vec<Exp<'a>>),
}

impl<'a> ArgList<'a> {
    pub fn get_args(&self) -> &Vec<Exp<'a>> {
        match self {
            ArgList::Args(args) => args,
        }
    }

    pub fn into_args(self) -> Vec<Exp<'a>> {
        match self {
            ArgList::Args(args) => args,
        }
    }
}
