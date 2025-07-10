use crate::ir::{IRCmd, IRExp, IRFunction};

pub struct Phi {
    temps: Vec<usize>,
}

pub fn into_ssa() {}

fn rename_temps_function(f: &mut IRFunction) {
    let mut vers: Vec<usize> = Vec::new();
    vers.resize_with(f.num_temps, || 0);
    for c in f.instructions.iter_mut() {
        match c {
            IRCmd::Load(temp, irexp) => {
                rename_temps_expr(irexp, &vers);
                if let IRExp::Temp(t) = temp {
                    vers[t.name] += 1;
                    t.ver += 1;
                }
            }
            IRCmd::JumpIf(irexp, _) => rename_temps_expr(irexp, &vers),
            IRCmd::Jump(_) => (),
            IRCmd::Label(_) => (),
            IRCmd::Return(irexp) => rename_temps_expr(irexp, &vers),
            IRCmd::Call(call) => match call {
                crate::ir::Call::Print(irexp) => rename_temps_expr(irexp, &vers),
                crate::ir::Call::Read => (),
                crate::ir::Call::Flush => (),
                crate::ir::Call::Func(_, irexps) => {
                    irexps.iter_mut().for_each(|e| rename_temps_expr(e, &vers));
                }
            },
        }
    }
}

fn rename_temps_expr(e: &mut IRExp, vers: &[usize]) {
    match e {
        IRExp::Temp(temp) => temp.ver = vers[temp.name],
        IRExp::ConstInt(_) | IRExp::ConstBool(_) => (),
        IRExp::Neg(irexp) | IRExp::NotBool(irexp) | IRExp::NotInt(irexp) => {
            rename_temps_expr(irexp, vers)
        }
        IRExp::Exp(b) => {
            rename_temps_expr(&mut b.0, vers);
            rename_temps_expr(&mut b.2, vers);
        }
        IRExp::Call(call) => match &mut **call {
            crate::ir::Call::Print(irexp) => rename_temps_expr(irexp, vers),
            crate::ir::Call::Read => (),
            crate::ir::Call::Flush => (),
            crate::ir::Call::Func(_, irexps) => {
                irexps.iter_mut().for_each(|e| rename_temps_expr(e, vers));
            }
        },
    }
}
