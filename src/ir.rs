use std::collections::HashMap;

use crate::{ast::Exp, elaboration::Abs};

#[derive(Clone, Debug)]
pub enum IRExp {
    Temp(usize),
    ConstInt(i32),
    ConstBool(bool),
    Neg(Box<IRExp>),
    NotBool(Box<IRExp>),
    NotInt(Box<IRExp>),
    Exp(Box<(IRExp, Op, IRExp)>),
}

//todo hashset

#[derive(Debug)]
pub enum IRCmd {
    Load(IRExp, IRExp),
    JumpIf(IRExp, usize),
    Jump(usize),
    Label(usize),
    Return(IRExp),
}

#[derive(Clone, Debug)]
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

pub fn translate_to_ir<'a>(
    abs: Abs<'a>,
    program: &mut Vec<IRCmd>,
    temp_count: &mut usize,
    label_count: &mut usize,
    vars: &mut HashMap<&'a [u8], IRExp>,
    label_cont: usize,
    label_brk: usize,
    step: Option<&Abs<'a>>,
) {
    match abs {
        Abs::ASGN(ident, mut exp) => {
            let mut e = exp_to_irexp(&mut exp, temp_count, label_count, vars);
            program.append(&mut e.0);
            if let Some(temp) = vars.insert(ident, IRExp::Temp(*temp_count)) {
                program.push(IRCmd::Load(temp, e.1));
            } else {
                program.push(IRCmd::Load(IRExp::Temp(*temp_count), e.1));
                *temp_count += 1;
            }
        }
        Abs::WHILE(mut exp, abs) => {
            let mut e = { exp_to_irexp(&mut exp, temp_count, label_count, vars) };
            let label_start = *label_count;
            let label_end = *label_count + 1;
            *label_count += 2;
            program.push(IRCmd::Label(label_start));
            program.append(&mut e.0);
            program.push(IRCmd::JumpIf(IRExp::NotBool(Box::new(e.1)), label_end));
            translate_to_ir(
                *abs,
                program,
                temp_count,
                label_count,
                vars,
                label_start,
                label_end,
                None,
            );
            program.push(IRCmd::Jump(label_start));
            program.push(IRCmd::Label(label_end));
        }
        Abs::CONT => {
            if let Some(abs) = step {
                translate_to_ir(
                    abs.clone(),
                    program,
                    temp_count,
                    label_count,
                    vars,
                    label_cont,
                    label_brk,
                    step,
                );
            }
            program.push(IRCmd::Jump(label_cont));
        }
        Abs::RET(mut exp) => {
            let mut e = exp_to_irexp(&mut exp, temp_count, label_count, vars);
            program.append(&mut e.0);
            program.push(IRCmd::Return(e.1));
        }
        Abs::DECL(_, _, abs) => {
            translate_to_ir(
                *abs,
                program,
                temp_count,
                label_count,
                vars,
                label_cont,
                label_brk,
                step,
            );
        }
        Abs::IF(mut exp, abs1, abs2) => {
            let mut e1 = exp_to_irexp(&mut exp, temp_count, label_count, vars);
            program.append(&mut e1.0);
            let then_label = *label_count;
            let end_label = *label_count + 1;
            *label_count += 2;
            program.push(IRCmd::JumpIf(e1.1, then_label));
            translate_to_ir(
                *abs2,
                program,
                temp_count,
                label_count,
                vars,
                label_cont,
                label_brk,
                step,
            );
            program.push(IRCmd::Jump(end_label));
            program.push(IRCmd::Label(then_label));
            translate_to_ir(
                *abs1,
                program,
                temp_count,
                label_count,
                vars,
                label_cont,
                label_brk,
                step,
            );
            program.push(IRCmd::Label(end_label));
        }
        Abs::FOR(b) => {
            let mut seq = Vec::new();
            match *b {
                Abs::DECL(_, _, abs) => {
                    if let Abs::SEQ(vec) = *abs {
                        seq = vec
                    }
                }
                Abs::SEQ(vec) => seq = vec,
                _ => (),
            }
            if matches!(seq[0], Abs::ASGN(..)) {
                translate_to_ir(
                    seq.remove(0),
                    program,
                    temp_count,
                    label_count,
                    vars,
                    label_cont,
                    label_brk,
                    step,
                );
            }
            let label_start = *label_count;
            let label_end = *label_count + 1;
            *label_count += 2;
            program.push(IRCmd::Label(label_start));
            if let Abs::EXP(mut exp) = seq.remove(0) {
                let mut e = { exp_to_irexp(&mut exp, temp_count, label_count, vars) };
                program.append(&mut e.0);
                program.push(IRCmd::JumpIf(IRExp::NotBool(Box::new(e.1)), label_end));
            }
            translate_to_ir(
                seq[0].clone(),
                program,
                temp_count,
                label_count,
                vars,
                label_cont,
                label_brk,
                seq.last(),
            );
            translate_to_ir(
                seq[1].clone(),
                program,
                temp_count,
                label_count,
                vars,
                label_cont,
                label_brk,
                step,
            );
            program.push(IRCmd::Jump(label_start));
            program.push(IRCmd::Label(label_end));
        }
        Abs::BRK => program.push(IRCmd::Jump(label_brk)),
        Abs::SEQ(items) => {
            for abs in items {
                translate_to_ir(
                    abs,
                    program,
                    temp_count,
                    label_count,
                    vars,
                    label_cont,
                    label_brk,
                    step,
                );
            }
        }
        Abs::EXP(_) => (),
    }
}

pub fn exp_to_irexp<'a>(
    exp: &mut Exp<'a>,
    temp_count: &mut usize,
    label_count: &mut usize,
    vars: &mut HashMap<&'a [u8], IRExp>,
) -> (Vec<IRCmd>, IRExp) {
    match exp {
        Exp::True => (vec![], IRExp::ConstBool(true)),
        Exp::False => (vec![], IRExp::ConstBool(false)),
        Exp::Intconst(num) => (vec![], IRExp::ConstInt(*num)),
        Exp::Ident(name) => (vec![], vars.get(name).unwrap().clone()),
        Exp::Arithmetic(b) => {
            let mut e1 = exp_to_irexp(&mut b.0, temp_count, label_count, vars);
            let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
            (e1.0).append(&mut e2.0);
            match b.1 {
                crate::ast::Binop::Plus => (e1.0, IRExp::Exp(Box::new((e1.1, Op::Plus, e2.1)))),
                crate::ast::Binop::Minus => (e1.0, IRExp::Exp(Box::new((e1.1, Op::Minus, e2.1)))),
                crate::ast::Binop::Div => {
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(*temp_count),
                        IRExp::Exp(Box::new((e1.1, Op::Div, e2.1))),
                    ));
                    *temp_count += 1;
                    (e1.0, IRExp::Temp(*temp_count - 1))
                }
                crate::ast::Binop::Mult => (e1.0, IRExp::Exp(Box::new((e1.1, Op::Mult, e2.1)))),
                crate::ast::Binop::Mod => {
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(*temp_count),
                        IRExp::Exp(Box::new((e1.1, Op::Mod, e2.1))),
                    ));
                    *temp_count += 1;
                    (e1.0, IRExp::Temp(*temp_count - 1))
                }
                crate::ast::Binop::LessThan => {
                    (e1.0, IRExp::Exp(Box::new((e1.1, Op::LessThan, e2.1))))
                }
                crate::ast::Binop::LessEqual => {
                    (e1.0, IRExp::Exp(Box::new((e1.1, Op::LessEqual, e2.1))))
                }
                crate::ast::Binop::GreaterThan => {
                    (e1.0, IRExp::Exp(Box::new((e1.1, Op::GreaterThan, e2.1))))
                }
                crate::ast::Binop::GreaterEqual => {
                    (e1.0, IRExp::Exp(Box::new((e1.1, Op::GreaterEqual, e2.1))))
                }
                crate::ast::Binop::Equals => (e1.0, IRExp::Exp(Box::new((e1.1, Op::Equals, e2.1)))),
                crate::ast::Binop::NotEqual => {
                    (e1.0, IRExp::Exp(Box::new((e1.1, Op::NotEqual, e2.1))))
                }
                crate::ast::Binop::And => {
                    let mut vec = Vec::new();
                    vec.append(&mut e1.0);
                    vec.push(IRCmd::JumpIf(IRExp::NotBool(Box::new(e1.1)), *label_count));
                    vec.append(&mut e2.0);
                    vec.push(IRCmd::JumpIf(IRExp::NotBool(Box::new(e2.1)), *label_count));
                    vec.push(IRCmd::Load(
                        IRExp::Temp(*temp_count),
                        IRExp::ConstBool(true),
                    ));
                    vec.push(IRCmd::Jump(*label_count + 1));
                    vec.push(IRCmd::Label(*label_count));
                    vec.push(IRCmd::Label(*label_count));
                    vec.push(IRCmd::Load(
                        IRExp::Temp(*temp_count),
                        IRExp::ConstBool(false),
                    ));
                    vec.push(IRCmd::Label(*label_count + 1));
                    *temp_count += 1;
                    *label_count += 2;
                    (vec, IRExp::Temp(*temp_count - 1))
                }
                crate::ast::Binop::Or => {
                    let mut vec = Vec::new();
                    vec.append(&mut e1.0);
                    vec.push(IRCmd::JumpIf(e1.1, *label_count));
                    vec.append(&mut e2.0);
                    vec.push(IRCmd::JumpIf(IRExp::NotBool(Box::new(e2.1)), *label_count));
                    vec.push(IRCmd::Load(
                        IRExp::Temp(*temp_count),
                        IRExp::ConstBool(false),
                    ));
                    vec.push(IRCmd::Jump(*label_count + 1));
                    vec.push(IRCmd::Label(*label_count));
                    vec.push(IRCmd::Load(
                        IRExp::Temp(*temp_count),
                        IRExp::ConstBool(true),
                    ));
                    vec.push(IRCmd::Label(*label_count + 1));
                    *temp_count += 1;
                    *label_count += 2;
                    (vec, IRExp::Temp(*temp_count - 1))
                }
                crate::ast::Binop::BitAnd => (e1.0, IRExp::Exp(Box::new((e1.1, Op::BitAnd, e2.1)))),
                crate::ast::Binop::BitXor => (e1.0, IRExp::Exp(Box::new((e1.1, Op::BitXor, e2.1)))),
                crate::ast::Binop::BitOr => (e1.0, IRExp::Exp(Box::new((e1.1, Op::BitOr, e2.1)))),
                crate::ast::Binop::LShift => (e1.0, IRExp::Exp(Box::new((e1.1, Op::LShift, e2.1)))),
                crate::ast::Binop::RShift => (e1.0, IRExp::Exp(Box::new((e1.1, Op::RShift, e2.1)))),
            }
        }
        Exp::Negative(exp) => {
            let e = exp_to_irexp(exp, temp_count, label_count, vars);
            (e.0, IRExp::Neg(Box::new(e.1)))
        }
        Exp::Not(exp) => {
            let e = exp_to_irexp(exp, temp_count, label_count, vars);
            (e.0, IRExp::NotBool(Box::new(e.1)))
        }
        Exp::BitNot(exp) => {
            let e = exp_to_irexp(exp, temp_count, label_count, vars);
            (e.0, IRExp::NotInt(Box::new(e.1)))
        }
        Exp::Ternary(b) => {
            let mut e1 = exp_to_irexp(&mut b.0, temp_count, label_count, vars);
            let mut e2 = exp_to_irexp(&mut b.1, temp_count, label_count, vars);
            let e3 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
            let mut vec = Vec::new();
            vec.append(&mut e1.0);
            vec.push(IRCmd::JumpIf(e1.1, *label_count));
            vec.append(&mut e2.0);
            vec.push(IRCmd::Load(IRExp::Temp(*temp_count), e2.1));
            vec.push(IRCmd::Jump(*label_count + 1));
            vec.push(IRCmd::Label(*label_count));
            *label_count += 1;
            vec.append(&mut e2.0);
            vec.push(IRCmd::Load(IRExp::Temp(*temp_count), e3.1));
            vec.push(IRCmd::Label(*label_count));
            *label_count += 1;
            *temp_count += 1;
            (vec, IRExp::Temp(*temp_count - 1))
        }
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
