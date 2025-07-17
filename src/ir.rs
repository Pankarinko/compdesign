use std::collections::HashMap;

use crate::{ast::Exp, elaboration::Abs, semantics::AbsFunction};

#[derive(Debug)]
pub struct IRFunction<'a> {
    pub name: &'a [u8],
    pub num_temps: usize,
    pub num_params: usize,
    pub instructions: Vec<IRCmd>,
}

#[derive(Clone, Debug)]
pub enum IRExp {
    Temp(Temp),
    ConstInt(i32),
    ConstBool(bool),
    Neg(Box<IRExp>),
    NotBool(Box<IRExp>),
    NotInt(Box<IRExp>),
    Exp(Box<(IRExp, Op, IRExp)>),
    Call(Box<Call>),
}

#[derive(Debug, Clone)]
pub struct Temp {
    pub name: usize,
    pub ver: usize,
}

#[derive(Clone, Debug)]
pub enum Call {
    Print(IRExp),
    Read,
    Flush,
    Func(String, Vec<IRExp>),
}

#[derive(Clone, Debug)]
pub enum IRCmd {
    Load(IRExp, IRExp),
    JumpIf(IRExp, usize),
    Jump(usize),
    Label(usize),
    Return(IRExp),
    Call(Call),
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

pub fn translate_to_ir<'a>(funcs: Vec<AbsFunction<'a>>) -> Vec<IRFunction<'a>> {
    let mut label_count = 0;
    let mut funcs_in_ir = Vec::new();
    for f in funcs {
        let mut num_temps = 0;
        let label_cont = 0;
        let label_brk = 0;
        let mut vars: HashMap<&[u8], IRExp> = HashMap::new();
        f.param_names.iter().for_each(|name| {
            vars.insert(
                name,
                IRExp::Temp(Temp {
                    name: num_temps,
                    ver: 0,
                }),
            );
            num_temps += 1;
        });
        let mut instructions = Vec::new();
        translate_command(
            f.body,
            &mut instructions,
            &mut num_temps,
            &mut label_count,
            &mut vars,
            label_cont,
            label_brk,
            None,
        );
        funcs_in_ir.push(IRFunction {
            name: f.name,
            num_temps,
            num_params: f.param_names.len(),
            instructions,
        });
    }
    funcs_in_ir
}

fn translate_command<'a>(
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
            let temp = vars.get(ident).unwrap();
            program.push(IRCmd::Load(temp.clone(), e.1));
        }
        Abs::WHILE(mut exp, abs) => {
            let mut e = { exp_to_irexp(&mut exp, temp_count, label_count, vars) };
            let label_start = *label_count;
            let label_end = *label_count + 1;
            *label_count += 2;
            program.push(IRCmd::Label(label_start));
            program.append(&mut e.0);
            program.push(IRCmd::JumpIf(IRExp::NotBool(Box::new(e.1)), label_end));
            translate_command(
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
                translate_command(
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
        Abs::DECL(ident, _, abs) => {
            vars.insert(
                ident,
                IRExp::Temp(Temp {
                    name: *temp_count,
                    ver: 0,
                }),
            );
            *temp_count += 1;
            translate_command(
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
            translate_command(
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
            translate_command(
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
                Abs::DECL(ident, _, abs) => {
                    if let Abs::SEQ(vec) = *abs {
                        seq = vec
                    }
                    vars.insert(
                        ident,
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                    );
                    *temp_count += 1;
                    if matches!(seq[0], Abs::ASGN(..)) {
                        translate_command(
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
                }
                Abs::SEQ(vec) => {
                    seq = vec;
                    if matches!(seq[0], Abs::ASGN(..)) {
                        translate_command(
                            seq.remove(0),
                            program,
                            temp_count,
                            label_count,
                            vars,
                            label_cont,
                            label_brk,
                            step,
                        );
                    } else {
                        seq.remove(0);
                    }
                }
                _ => (),
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
            for i in 0..seq.len() - 1 {
                translate_command(
                    seq[i].clone(),
                    program,
                    temp_count,
                    label_count,
                    vars,
                    label_start,
                    label_end,
                    seq.last(),
                );
            }
            if let Some(last) = seq.last() {
                translate_command(
                    last.clone(),
                    program,
                    temp_count,
                    label_count,
                    vars,
                    label_cont,
                    label_brk,
                    step,
                );
            }
            program.push(IRCmd::Jump(label_start));
            program.push(IRCmd::Label(label_end));
        }
        Abs::BRK => program.push(IRCmd::Jump(label_brk)),
        Abs::SEQ(items) => {
            for abs in items {
                translate_command(
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
        Abs::CALL(name, mut args) => match name {
            b"print" => {
                let mut exp = args.pop().unwrap();
                let mut res = exp_to_irexp(&mut exp, temp_count, label_count, vars);
                program.append(&mut res.0);
                program.push(IRCmd::Call(Call::Print(res.1)));
            }
            b"read" => {
                program.push(IRCmd::Call(Call::Read));
            }
            b"flush" => {
                program.push(IRCmd::Call(Call::Flush));
            }
            _ => {
                let mut cmds = Vec::new();
                let mut func_args = Vec::new();
                for mut exp in args {
                    let mut res = exp_to_irexp(&mut exp, temp_count, label_count, vars);
                    cmds.append(&mut res.0);
                    func_args.push(res.1);
                }
                program.append(&mut cmds);
                program.push(IRCmd::Call(Call::Func(
                    format!("_{}", str::from_utf8(name).unwrap()),
                    func_args,
                )));
            }
        },
    }
}

fn exp_to_irexp<'a>(
    exp: &mut Exp<'a>,
    temp_count: &mut usize,
    label_count: &mut usize,
    vars: &mut HashMap<&'a [u8], IRExp>,
) -> (Vec<IRCmd>, IRExp) {
    match exp {
        Exp::True => {
            let vec = vec![IRCmd::Load(
                IRExp::Temp(Temp {
                    name: *temp_count,
                    ver: 0,
                }),
                IRExp::ConstBool(true),
            )];
            *temp_count += 1;
            (
                vec,
                IRExp::Temp(Temp {
                    name: *temp_count - 1,
                    ver: 0,
                }),
            )
        }
        Exp::False => {
            let vec = vec![IRCmd::Load(
                IRExp::Temp(Temp {
                    name: *temp_count,
                    ver: 0,
                }),
                IRExp::ConstBool(false),
            )];
            *temp_count += 1;
            (
                vec,
                IRExp::Temp(Temp {
                    name: *temp_count - 1,
                    ver: 0,
                }),
            )
        }
        Exp::Intconst(num) => {
            let vec = vec![IRCmd::Load(
                IRExp::Temp(Temp {
                    name: *temp_count,
                    ver: 0,
                }),
                IRExp::ConstInt(*num),
            )];
            *temp_count += 1;
            (
                vec,
                IRExp::Temp(Temp {
                    name: *temp_count - 1,
                    ver: 0,
                }),
            )
        }
        Exp::Ident(name) => {
            let vec = vec![IRCmd::Load(
                IRExp::Temp(Temp {
                    name: *temp_count,
                    ver: 0,
                }),
                vars.get(name).unwrap().clone(),
            )];
            *temp_count += 1;
            (
                vec,
                IRExp::Temp(Temp {
                    name: *temp_count - 1,
                    ver: 0,
                }),
            )
        }
        Exp::Arithmetic(b) => {
            let mut e1 = exp_to_irexp(&mut b.0, temp_count, label_count, vars);
            match b.1 {
                crate::ast::Binop::Plus => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::Plus, e2.1))),
                    ));
                }
                crate::ast::Binop::Minus => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::Minus, e2.1))),
                    ));
                }
                crate::ast::Binop::Div => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::Div, e2.1))),
                    ));
                }
                crate::ast::Binop::Mult => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::Mult, e2.1))),
                    ));
                }
                crate::ast::Binop::Mod => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::Mod, e2.1))),
                    ));
                }
                crate::ast::Binop::LessThan => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::LessThan, e2.1))),
                    ));
                }
                crate::ast::Binop::LessEqual => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::LessEqual, e2.1))),
                    ));
                }
                crate::ast::Binop::GreaterThan => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::GreaterThan, e2.1))),
                    ));
                }
                crate::ast::Binop::GreaterEqual => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::GreaterEqual, e2.1))),
                    ));
                }
                crate::ast::Binop::Equals => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::Equals, e2.1))),
                    ));
                }
                crate::ast::Binop::NotEqual => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::NotEqual, e2.1))),
                    ));
                }
                crate::ast::Binop::And => {
                    let mut vec = Vec::new();
                    vec.append(&mut e1.0);
                    let false_label = *label_count;
                    let done_label = *label_count + 1;
                    *label_count += 2;
                    vec.push(IRCmd::JumpIf(IRExp::NotBool(Box::new(e1.1)), false_label));
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    vec.append(&mut e2.0);
                    vec.push(IRCmd::JumpIf(IRExp::NotBool(Box::new(e2.1)), false_label));
                    vec.push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::ConstBool(true),
                    ));
                    vec.push(IRCmd::Jump(done_label));
                    vec.push(IRCmd::Label(false_label));
                    vec.push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::ConstBool(false),
                    ));
                    vec.push(IRCmd::Label(done_label));
                    *temp_count += 1;
                    return (
                        vec,
                        IRExp::Temp(Temp {
                            name: *temp_count - 1,
                            ver: 0,
                        }),
                    );
                }
                crate::ast::Binop::Or => {
                    let mut vec = Vec::new();
                    vec.append(&mut e1.0);
                    let true_label = *label_count;
                    let done_label = *label_count + 1;
                    *label_count += 2;
                    vec.push(IRCmd::JumpIf(e1.1, true_label));
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    vec.append(&mut e2.0);
                    vec.push(IRCmd::JumpIf(e2.1, true_label));
                    vec.push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::ConstBool(false),
                    ));
                    vec.push(IRCmd::Jump(done_label));
                    vec.push(IRCmd::Label(true_label));
                    vec.push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::ConstBool(true),
                    ));
                    vec.push(IRCmd::Label(done_label));
                    *temp_count += 1;
                    return (
                        vec,
                        IRExp::Temp(Temp {
                            name: *temp_count - 1,
                            ver: 0,
                        }),
                    );
                }
                crate::ast::Binop::BitAnd => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::BitAnd, e2.1))),
                    ));
                }
                crate::ast::Binop::BitXor => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::BitXor, e2.1))),
                    ));
                }
                crate::ast::Binop::BitOr => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::BitOr, e2.1))),
                    ));
                }
                crate::ast::Binop::LShift => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::LShift, e2.1))),
                    ));
                }
                crate::ast::Binop::RShift => {
                    let mut e2 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
                    (e1.0).append(&mut e2.0);
                    (e1.0).push(IRCmd::Load(
                        IRExp::Temp(Temp {
                            name: *temp_count,
                            ver: 0,
                        }),
                        IRExp::Exp(Box::new((e1.1, Op::RShift, e2.1))),
                    ));
                }
            }
            *temp_count += 1;
            (
                e1.0,
                IRExp::Temp(Temp {
                    name: *temp_count - 1,
                    ver: 0,
                }),
            )
        }
        Exp::Negative(exp) => {
            let mut e = exp_to_irexp(exp, temp_count, label_count, vars);
            {
                e.0.push(IRCmd::Load(
                    IRExp::Temp(Temp {
                        name: *temp_count,
                        ver: 0,
                    }),
                    IRExp::Neg(Box::new(e.1)),
                ));
                *temp_count += 1;
                (
                    e.0,
                    IRExp::Temp(Temp {
                        name: *temp_count - 1,
                        ver: 0,
                    }),
                )
            }
        }
        Exp::Not(exp) => {
            let mut e = exp_to_irexp(exp, temp_count, label_count, vars);
            {
                e.0.push(IRCmd::Load(
                    IRExp::Temp(Temp {
                        name: *temp_count,
                        ver: 0,
                    }),
                    IRExp::NotBool(Box::new(e.1)),
                ));
                *temp_count += 1;
                (
                    e.0,
                    IRExp::Temp(Temp {
                        name: *temp_count - 1,
                        ver: 0,
                    }),
                )
            }
        }
        Exp::BitNot(exp) => {
            let mut e = exp_to_irexp(exp, temp_count, label_count, vars);
            {
                e.0.push(IRCmd::Load(
                    IRExp::Temp(Temp {
                        name: *temp_count,
                        ver: 0,
                    }),
                    IRExp::NotInt(Box::new(e.1)),
                ));
                *temp_count += 1;
                (
                    e.0,
                    IRExp::Temp(Temp {
                        name: *temp_count - 1,
                        ver: 0,
                    }),
                )
            }
        }
        Exp::Ternary(b) => {
            let mut e1 = exp_to_irexp(&mut b.0, temp_count, label_count, vars);
            let mut e2 = exp_to_irexp(&mut b.1, temp_count, label_count, vars);
            let mut e3 = exp_to_irexp(&mut b.2, temp_count, label_count, vars);
            let mut vec = Vec::new();
            vec.append(&mut e1.0);
            vec.push(IRCmd::JumpIf(e1.1, *label_count));
            vec.append(&mut e3.0);
            vec.push(IRCmd::Load(
                IRExp::Temp(Temp {
                    name: *temp_count,
                    ver: 0,
                }),
                e3.1,
            ));
            vec.push(IRCmd::Jump(*label_count + 1));
            vec.push(IRCmd::Label(*label_count));
            *label_count += 1;
            vec.append(&mut e2.0);
            vec.push(IRCmd::Load(
                IRExp::Temp(Temp {
                    name: *temp_count,
                    ver: 0,
                }),
                e2.1,
            ));
            vec.push(IRCmd::Label(*label_count));
            *label_count += 1;
            *temp_count += 1;
            (
                vec,
                IRExp::Temp(Temp {
                    name: *temp_count - 1,
                    ver: 0,
                }),
            )
        }
        Exp::Call(call) => match call {
            crate::ast::Call::Print(arg_list) => {
                let mut cmds = Vec::new();
                let mut exp = arg_list.clone().into_args().pop().unwrap();
                let mut res = exp_to_irexp(&mut exp, temp_count, label_count, vars);
                cmds.append(&mut res.0);
                cmds.push(IRCmd::Load(
                    IRExp::Temp(Temp {
                        name: *temp_count,
                        ver: 0,
                    }),
                    IRExp::Call(Box::new(Call::Print(res.1))),
                ));
                *temp_count += 1;
                (
                    cmds,
                    IRExp::Temp(Temp {
                        name: *temp_count - 1,
                        ver: 0,
                    }),
                )
            }
            crate::ast::Call::Read(..) => {
                let cmds = vec![IRCmd::Load(
                    IRExp::Temp(Temp {
                        name: *temp_count,
                        ver: 0,
                    }),
                    IRExp::Call(Box::new(Call::Read)),
                )];
                *temp_count += 1;
                (
                    cmds,
                    IRExp::Temp(Temp {
                        name: *temp_count - 1,
                        ver: 0,
                    }),
                )
            }
            crate::ast::Call::Flush(..) => {
                let cmds = vec![IRCmd::Load(
                    IRExp::Temp(Temp {
                        name: *temp_count,
                        ver: 0,
                    }),
                    IRExp::Call(Box::new(Call::Flush)),
                )];
                *temp_count += 1;
                (
                    cmds,
                    IRExp::Temp(Temp {
                        name: *temp_count - 1,
                        ver: 0,
                    }),
                )
            }
            crate::ast::Call::Func(name, arg_list) => {
                let mut cmds = Vec::new();
                let mut args = Vec::new();
                for mut exp in arg_list.clone().into_args() {
                    let mut res = exp_to_irexp(&mut exp, temp_count, label_count, vars);
                    cmds.append(&mut res.0);
                    args.push(res.1);
                }
                cmds.push(IRCmd::Load(
                    IRExp::Temp(Temp {
                        name: *temp_count,
                        ver: 0,
                    }),
                    IRExp::Call(Box::new(Call::Func(
                        format!("_{}", str::from_utf8(name).unwrap()),
                        args,
                    ))),
                ));
                *temp_count += 1;
                (
                    cmds,
                    IRExp::Temp(Temp {
                        name: *temp_count - 1,
                        ver: 0,
                    }),
                )
            }
        },
    }
}
