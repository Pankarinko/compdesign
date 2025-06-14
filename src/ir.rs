use std::{collections::HashMap};

use crate::{ast::Exp, elaboration::Abs};

pub enum IRExp {
    Val(usize),
    ConstInt(i32),
    ConstBool(bool),
    Neg(Box<IRExp>),
    NotBool(Box<IRExp>),
    NotInt(Box<IRExp>),
    Exp(Box<(IRExp, Op, IRExp)>),
}

//todo hashset


pub enum IRCmd {
    Expr(usize, IRExp),
    Arithmetic(usize, IRExp, Op, IRExp),
    JUmpIf(IRExp, usize),
    Jump(usize),
    Label(usize),
    Return(IRExp),
}

pub enum Op {
    Plus,
    Minus,
    Mult,
    Div,
    Mod,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Equals,
    NotEqual,
    BitAnd,
    BitXor,
    BitOr,
    LShift,
    RShift,
}


pub fn translate_to_ir(abs: Abs, program: &mut Vec<IRCmd>) {
    match abs {
        Abs::ASGN(ident, exp) => ,
        Abs::WHILE(exp, abs) => todo!(),
        Abs::CONT => todo!(),
        Abs::RET(exp) => todo!(),
        Abs::DECL(items, _, abs) => todo!(),
        Abs::IF(exp, abs, abs1) => todo!(),
        Abs::FOR(abs) => todo!(),
        Abs::BRK => todo!(),
        Abs::SEQ(items) => todo!(),
        Abs::EXP(exp) => todo!(),
    }
}

pub fn exp_to_irexp<'a>(exp: &mut Exp<'a>, temp_count:  &mut usize, label_count: &mut usize, temps: &mut HashMap<usize, i32>) -> (Vec<IRCmd>, IRExp) {
    match exp {
        Exp::True => (vec![], IRExp::ConstBool(true)),
        Exp::False => (vec![], IRExp::ConstBool(false)),
        Exp::Intconst(num) => (vec![], IRExp::ConstInt(*num)),
        Exp::Ident(name) => {
            // TODO: insert value
            *temp_count += 1;
            (vec![], IRExp::Val(*temp_count - 1))},
        Exp::Arithmetic(b) => {
            let e1 = exp_to_irexp(&mut b.0, temp_count, label_count, temps);
            let e2 = exp_to_irexp(&mut b.2, temp_count, label_count, temps);
            (e1.0).append(&mut e2.0);
    match b.1 {
            crate::ast::Binop::Plus => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::Plus, e2.1))))
            },
            crate::ast::Binop::Minus => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::Minus, e2.1))))
            },
            crate::ast::Binop::Div => {(e1.0).push(IRCmd::Expr(*temp_count, IRExp::Exp(Box::new((e1.1, Op::Div, e2.1)))));
            *temp_count += 1;
            (e1.0, IRExp::Val(*temp_count - 1))},
            crate::ast::Binop::Mult => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::Mult, e2.1))))
            },
            crate::ast::Binop::Mod => {(e1.0).push(IRCmd::Expr(*temp_count, IRExp::Exp(Box::new((e1.1, Op::Mod, e2.1)))));
            *temp_count += 1;
            (e1.0, IRExp::Val(*temp_count - 1))},
            crate::ast::Binop::LessThan => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::LessThan, e2.1))))
            },
            crate::ast::Binop::LessEqual => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::LessEqual, e2.1))))
            },
            crate::ast::Binop::GreaterThan => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::GreaterThan, e2.1))))
            },
            crate::ast::Binop::GreaterEqual => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::GreaterEqual, e2.1))))
            },
            crate::ast::Binop::Equals => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::Equals, e2.1))))
            },
            crate::ast::Binop::NotEqual => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::NotEqual, e2.1))))
            },
            crate::ast::Binop::And => return None,
            crate::ast::Binop::Or => return None,
            crate::ast::Binop::BitAnd => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::BitAnd, e2.1))))
            },
            crate::ast::Binop::BitXor => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::BitXor, e2.1))))
            },
            crate::ast::Binop::BitOr => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::BitOr, e2.1))))
            },
            crate::ast::Binop::LShift => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::LShift, e2.1))))
            },
            crate::ast::Binop::RShift => {
            
            (e1.0, IRExp::Exp(Box::new((e1.1, Op::RShift, e2.1))))
            },
        }},
        Exp::Negative(exp) => {
            let e = exp_to_irexp(exp, temp_count, label_count, temps);
            (e.0, IRExp::Neg(Box::new(e.1)))},
        Exp::Not(exp) => {
            let e = exp_to_irexp(exp, temp_count, label_count, temps);
            (e.0, IRExp::NotBool(Box::new(e.1)))},
        Exp::BitNot(exp) => {
            let e = exp_to_irexp(exp, temp_count, label_count, temps);
            (e.0, IRExp::NotInt(Box::new(e.1)))},
        Exp::Ternary(b) =>            { let e1 = exp_to_irexp(&mut b.0, temp_count, label_count, temps);
            let e2 = exp_to_irexp(&mut b.1, temp_count, label_count, temps);
            let e3 = exp_to_irexp(&mut b.2, temp_count, label_count, temps);
            let vec = Vec::new();
            vec.append(&mut e1.0);
            vec.push(IRCmd::JUmpIf(e1.1, *label_count));
            vec.append(&mut e2.0);
            vec.push(IRCmd::Expr(*temp_count, e2.1));
            vec.push(IRCmd::Jump(*label_count + 1));
            vec.push(IRCmd::Label(*label_count));
            *label_count += label_count;
            vec.append(&mut e2.0);
            vec.push(IRCmd::Expr(*temp_count, e2.1));
            vec.push(IRCmd::Label(*label_count));
            *label_count += 1;
            *temp_count += 1;
            (vec, IRExp::Val(*temp_count - 1))
    },
    }
}

/*
instructions:
Add,
And,
cmp,
Div/idiv
Jmp,
Mov,
Neg,
Nop,
Not, logical
Or, logical
Sal, shift arith left
Sar shift arith right,
sub,
xor,
Push
Pop


 */
