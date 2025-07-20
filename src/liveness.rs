use crate::ir::{IRCmd, IRExp, Temp};

#[derive(Debug)]
enum Rules {
    Use(usize),
    Def(usize),
    Succ(usize),
    Nec(usize),
}

/* Creates a vector of currently live temps for every line. Repeats until saturated. */
pub fn analyze_func(cmds: &mut Vec<IRCmd>) -> Vec<Vec<usize>> {
    let rules = break_func_into_rules(cmds);
    let mut live_temps = Vec::new();
    let mut needed_temps = Vec::new();
    for _ in 0..rules.len() {
        needed_temps.push(vec![]);
    }
    loop {
        println!("{:?}", needed_temps);
        if collect_needed_temps(&rules, &mut needed_temps) {
            break;
        }
    }
    let len = cmds.len();
    for cmd in cmds.iter_mut() {
        if let IRCmd::Load(IRExp::Temp(t), _) = cmd {
            if !needed_temps.iter().any(|x| x.contains(&t.name)) {
                *cmd = IRCmd::Label(len);
            }
        }
    }
    for _ in 0..rules.len() {
        live_temps.push(vec![]);
    }
    loop {
        if collect_live_temps(&rules, &mut live_temps) {
            break;
        }
    }
    live_temps
}

fn collect_live_temps(rules: &[Vec<Rules>], live_temps: &mut [Vec<usize>]) -> bool {
    let mut saturated = true;
    for (index, line) in rules.iter().rev().enumerate() {
        let i = live_temps.len() - index - 1;
        for rule in line.iter() {
            if let Rules::Use(temp) = rule {
                if !live_temps[i].contains(temp) {
                    saturated = false;
                    live_temps[i].push(*temp);
                }
            }
        }
        for l in line.iter().filter_map(|r| {
            if let Rules::Succ(line_i) = *r
                && line_i < rules.len()
            {
                Some(line_i)
            } else {
                None
            }
        }) {
            for temp in live_temps[l].clone().iter() {
                if !line.iter().any(|r| {
                    if let Rules::Def(t) = r {
                        if *t == *temp {
                            return true;
                        }
                    }
                    false
                }) && !live_temps[i].contains(temp)
                {
                    saturated = false;
                    live_temps[i].push(*temp);
                }
            }
        }
    }
    saturated
}

fn collect_needed_temps(rules: &[Vec<Rules>], needed_temps: &mut [Vec<usize>]) -> bool {
    let mut saturated = true;
    for (index, line) in rules.iter().rev().enumerate() {
        let i = needed_temps.len() - index - 1;
        for rule in line.iter() {
            if let Rules::Nec(temp) = rule {
                if !needed_temps[i].contains(temp) {
                    saturated = false;
                    needed_temps[i].push(*temp);
                }
            }
        }
        for l in line.iter().filter_map(|r| {
            if let Rules::Succ(line_i) = *r
                && line_i < rules.len()
            {
                Some(line_i)
            } else {
                None
            }
        }) {
            for temp in needed_temps[l].clone().iter() {
                if !line.iter().any(|r| {
                    if let Rules::Def(t) = r {
                        if *t == *temp {
                            return true;
                        }
                    }
                    false
                }) {
                    if !needed_temps[i].contains(temp) {
                        saturated = false;
                        needed_temps[i].push(*temp);
                    }
                } else {
                    for x in line
                        .iter()
                        .filter_map(|r| if let Rules::Use(x) = r { Some(x) } else { None })
                    {
                        if !needed_temps[i].contains(x) {
                            saturated = false;
                            needed_temps[i].push(*x);
                        }
                    }
                }
            }
        }
    }
    saturated
}

fn break_func_into_rules(cmds: &[IRCmd]) -> Vec<Vec<Rules>> {
    let mut rules = Vec::new();
    for (i, c) in cmds.iter().enumerate() {
        let mut rules_line = Vec::new();
        match c {
            IRCmd::Load(t, exp) => {
                rules_line.push(Rules::Def(get_temps(t)[0]));
                let temps = get_temps(exp);
                get_exp_with_effect(exp, &mut rules_line);
                temps.iter().for_each(|t| rules_line.push(Rules::Use(*t)));
                rules_line.push(Rules::Succ(i + 1));
            }
            IRCmd::JumpIf(exp, l) => {
                let temps = get_temps(exp);
                temps.iter().for_each(|t| {
                    rules_line.push(Rules::Use(*t));
                    rules_line.push(Rules::Nec(*t));
                });
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
                temps.iter().for_each(|t| {
                    rules_line.push(Rules::Use(*t));
                    rules_line.push(Rules::Nec(*t));
                });
            }
            IRCmd::Call(call) => match call {
                crate::ir::Call::Print(irexp) => {
                    let temps = get_temps(irexp);
                    temps.iter().for_each(|t| rules_line.push(Rules::Use(*t)));
                    rules_line.push(Rules::Succ(i + 1));
                }
                crate::ir::Call::Read | crate::ir::Call::Flush => {
                    rules_line.push(Rules::Succ(i + 1))
                }
                crate::ir::Call::Func(_, irexps) => {
                    let mut temps = vec![];
                    irexps.iter().for_each(|x| temps.append(&mut get_temps(x)));
                    temps.iter().for_each(|t| {
                        rules_line.push(Rules::Use(*t));
                        rules_line.push(Rules::Nec(*t));
                    });
                    rules_line.push(Rules::Succ(i + 1));
                }
            },
        }
        rules.push(rules_line);
    }
    rules
}

fn get_exp_with_effect(exp: &IRExp, rules: &mut Vec<Rules>) {
    match exp {
        IRExp::Temp(_) => (),
        IRExp::ConstInt(_) => (),
        IRExp::ConstBool(_) => (),
        IRExp::Neg(irexp) | IRExp::NotBool(irexp) | IRExp::NotInt(irexp) => {
            get_exp_with_effect(irexp, rules)
        }

        IRExp::Exp(b) => {
            let ar = &**b;
            match ar.1 {
                crate::ir::Op::Div | crate::ir::Op::Mod => {
                    if let IRExp::Temp(t) = &ar.0 {
                        rules.push(Rules::Nec(t.name));
                    }
                    if let IRExp::Temp(t) = &ar.2 {
                        rules.push(Rules::Nec(t.name));
                    }
                }
                _ => (),
            }
        }
        IRExp::Call(call) => match &**call {
            crate::ir::Call::Print(IRExp::Temp(t)) => {
                rules.push(Rules::Nec(t.name));
            }
            crate::ir::Call::Func(_, irexps) => {
                irexps.iter().for_each(|e| {
                    if let IRExp::Temp(t) = e {
                        rules.push(Rules::Nec(t.name));
                    }
                });
            }
            _ => (),
        },
    }
    println!("{:?}", exp);
}

/* Collects all the temps in an expression */
fn get_temps(exp: &IRExp) -> Vec<usize> {
    match exp {
        IRExp::Temp(t) => vec![t.name],
        IRExp::ConstInt(_) | IRExp::ConstBool(_) => vec![],
        IRExp::Neg(irexp) | IRExp::NotBool(irexp) | IRExp::NotInt(irexp) => get_temps(irexp),
        IRExp::Exp(b) => {
            let mut temps = get_temps(&b.0);
            temps.append(&mut get_temps(&b.2));
            temps
        }
        IRExp::Call(call) => match &**call {
            crate::ir::Call::Print(irexp) => get_temps(irexp),
            crate::ir::Call::Read | crate::ir::Call::Flush => vec![],
            crate::ir::Call::Func(_, args) => {
                let mut temps = vec![];
                args.iter().for_each(|x| temps.append(&mut get_temps(x)));
                temps
            }
        },
    }
}
