use crate::ir::{IRCmd, IRExp, IRFunction};

enum Rules {
    Use(usize),
    Def(usize),
    Succ(usize),
}

fn liveness_analysis() -> Vec<Vec<usize>> {
    todo!()
}

fn break_program_into_rules(funcs: Vec<IRFunction<'_>>) {}

fn break_func_into_rules(cmds: Vec<IRCmd>) -> Vec<Vec<Rules>> {
    let mut rules = Vec::new();
    for (i, c) in cmds.iter().enumerate() {
        let mut rules_line = Vec::new();
        match c {
            IRCmd::Load(t, exp) => {
                rules_line.push(Rules::Def(get_temps(t)[0]));
                let temps = get_temps(exp);
                temps.iter().for_each(|t| rules_line.push(Rules::Use(*t)));
                rules_line.push(Rules::Succ(i + 1));
            }
            IRCmd::JumpIf(exp, l) => {
                let temps = get_temps(exp);
                temps.iter().for_each(|t| rules_line.push(Rules::Use(*t)));
                let line_i = cmds
                    .iter()
                    .position(|x| {
                        if let IRCmd::Label(line) = x {
                            if line == l {
                                return true;
                            }
                        }
                        false
                    })
                    .unwrap();
                rules_line.push(Rules::Succ(line_i + 1));
                rules_line.push(Rules::Succ(i + 1));
            }
            IRCmd::Jump(l) => {
                let line_i = cmds
                    .iter()
                    .position(|x| {
                        if let IRCmd::Label(line) = x {
                            if line == l {
                                return true;
                            }
                        }
                        false
                    })
                    .unwrap();
                rules_line.push(Rules::Succ(line_i + 1));
            }
            IRCmd::Label(_) => rules_line.push(Rules::Succ(i + 1)),
            IRCmd::Return(exp) => {
                let temps = get_temps(exp);
                temps.iter().for_each(|t| rules_line.push(Rules::Use(*t)));
            }
            IRCmd::Call(call) => match call {
                crate::ir::Call::Print(irexp) => {
                    let temps = get_temps(irexp);
                    temps.iter().for_each(|t| rules_line.push(Rules::Use(*t)));
                    rules_line.push(Rules::Succ(i + 1));
                }
                crate::ir::Call::Read => rules_line.push(Rules::Succ(i + 1)),
                crate::ir::Call::Flush => rules_line.push(Rules::Succ(i + 1)),
                crate::ir::Call::Func(_, irexps) => {
                    let mut temps = vec![];
                    irexps.iter().for_each(|x| temps.append(&mut get_temps(x)));
                    temps.iter().for_each(|t| rules_line.push(Rules::Use(*t)));
                    rules_line.push(Rules::Succ(i + 1));
                }
            },
        }
        rules.push(rules_line);
    }
    rules
}

fn get_temps(exp: &IRExp) -> Vec<usize> {
    match exp {
        IRExp::Temp(t) => vec![*t],
        IRExp::ConstInt(_) => vec![],
        IRExp::ConstBool(_) => vec![],
        IRExp::Neg(irexp) => get_temps(irexp),
        IRExp::NotBool(irexp) => get_temps(irexp),
        IRExp::NotInt(irexp) => get_temps(irexp),
        IRExp::Exp(b) => {
            let mut temps = get_temps(&b.0);
            temps.append(&mut get_temps(&b.2));
            temps
        }
        IRExp::Call(call) => match &**call {
            crate::ir::Call::Print(irexp) => get_temps(irexp),
            crate::ir::Call::Read => vec![],
            crate::ir::Call::Flush => vec![],
            crate::ir::Call::Func(_, args) => {
                let mut temps = vec![];
                args.iter().for_each(|x| temps.append(&mut get_temps(x)));
                temps
            }
        },
    }
}
