use std::collections::HashMap;

use crate::ir::{IRCmd, IRExp, Op};

#[derive(Clone)]
pub enum Operand {
    Reg(usize),
    Imm(i32),
}

fn map_temp_to_register(temps_count: usize, temps: HashMap<usize, String>) {}

fn translate_instruction(cmd: IRCmd, assembly: &mut String, temps: &mut HashMap<usize, String>) {
    match cmd {
        IRCmd::Load(irexp, irexp1) => {
            let operand = expr_to_assembly(irexp1, assembly, temps);
            if let IRExp::Temp(temp) = irexp {
                match operand {
                    Operand::Imm(i) => {
                        assembly.push_str(&format!("mov ${} %{}\n", temps.get(reg).unwrap();, i))
                    }
                    Operand::Reg(reg) => {
                        assembly.push_str(&format!("mov %{} %{}\n", temps.get(reg).unwrap();, reg))
                    }
                }
            }
        }
        IRCmd::JumpIf(irexp, _) => todo!(),
        IRCmd::Jump(_) => todo!(),
        IRCmd::Label(label) => assembly.push_str(&format!("{label}:")),
        IRCmd::Return(irexp) => todo!(),
    }
}

fn constant_folding(e1: IRExp, op: Op, e2: IRExp, assembly: &mut String) -> Option<Operand> {
    match op {
        Op::Plus => {
            if let IRExp::ConstInt(val1) = e1 {
                if let IRExp::ConstInt(val2) = e2 {
                    assembly.push_str(&format!("mov ${} %eax\n", val1 + val2));
                    return Some(0);
                }
            }
            return None;
        }
        Op::Minus => {
            if let IRExp::ConstInt(val1) = e1 {
                if let IRExp::ConstInt(val2) = e2 {
                    assembly.push_str(&format!("mov ${} %eax\n", val1 + val2));
                    return Some(Operand::Reg(0));
                }
            }
            return None;
        }
        Op::Mult => {
            if let IRExp::ConstInt(val1) = e1 {
                if let IRExp::ConstInt(val2) = e2 {
                    assembly.push_str(&format!("mov ${} %eax\n", val1 - val2));
                    return Some(Operand::Reg(0));
                }
            }
            return None;
        }
        _ => None,
    }
}

fn expr_to_assembly(
    expr: IRExp,
    assembly: &mut String,
    temps: &mut HashMap<usize, String>,
) -> Operand {
    match expr {
        IRExp::Temp(t) => return Operand::Reg(t + 1),
        IRExp::ConstInt(val) => return Operand::Imm(val),
        IRExp::ConstBool(val) => {
            if val == true {
                return Operand::Imm(1);
            } else {
                return Operand::Imm(0);
            }
        }
        IRExp::Neg(irexp) => {
            let operand = expr_to_assembly(*irexp, assembly, temps);
            if let Operand::Reg(r) = operand.clone() {
                assembly.push_str(&format!("neg %{}\n", temps.get(&r).unwrap()));
                operand
            } else {
                if let Operand::Imm(val) = operand {
                    assembly.push_str(&format!("mov ${} %eax\n", val));
                    assembly.push_str(&format!("neg %eax\n"));
                }
                Operand::Reg(0)
            }
        }
        IRExp::NotBool(irexp) => todo!(),
        IRExp::NotInt(irexp) => {
            let operand = expr_to_assembly(*irexp, assembly, temps);
            if let Operand::Reg(r) = operand.clone() {
                assembly.push_str(&format!("neg %{}\n", temps.get(&r).unwrap()));
                operand
            } else {
                if let Operand::Imm(val) = operand {
                    assembly.push_str(&format!("mov ${} %eax\n", val));
                    assembly.push_str(&format!("not %eax\n"));
                }
                Operand::Reg(0)
            }
        }
        IRExp::Exp(b) => {
            let (e1, op, e2) = *b;
            if let Some(index) = constant_folding(e1.clone(), op.clone(), e2.clone(), assembly) {
                return index;
            }
            let first = expr_to_assembly(e1.clone(), assembly, temps);
            let second = expr_to_assembly(e2.clone(), assembly, temps);
            let first_op: String;
            let second_op: String;
            match first {
                Operand::Reg(r) => first_op = format!("%{}", temps.get(&r).unwrap()),
                Operand::Imm(i) => first_op = format!("${i}"),
            }
            match second {
                Operand::Reg(r) => second_op = format!("%{}", temps.get(&r).unwrap()),
                Operand::Imm(i) => second_op = format!("${i}"),
            }
            match op {
                crate::ir::Op::Plus => {
                    assembly.push_str(&format!("mov {} %eax\n", first_op));
                    assembly.push_str(&format!("add {} %eax\n", second_op));
                    return Operand::Reg(0);
                }
                crate::ir::Op::Minus => {
                    assembly.push_str(&format!("mov {} %eax\n", first_op));
                    assembly.push_str(&format!("sub {} %eax\n", second_op));
                    return Operand::Reg(0);
                }
                crate::ir::Op::Mult => {
                    assembly.push_str(&format!("mov {} %eax\n", first_op));
                    assembly.push_str(&format!("imul {} %eax\n", second_op));
                    return Operand::Reg(0);
                }
                crate::ir::Op::Div => {
                    assembly.push_str(&format!("mov {} %eax\n", first_op));
                    assembly.push_str(&format!("cdq\n"));
                    assembly.push_str(&format!("idiv {}\n", second_op));
                    return Operand::Reg(0);
                }
                crate::ir::Op::Mod => {
                    assembly.push_str(&format!("movl {} %eax\n", first_op));
                    assembly.push_str(&format!("cdq\n"));
                    assembly.push_str(&format!("idiv {}\n", second_op));
                    return Operand::Reg(1);
                }
                crate::ir::Op::LessThan => {
                    if let Operand::Imm(_) = first {
                        let expr = IRExp::Exp(Box::new((e2, Op::GreaterEqual, e1)));
                        return expr_to_assembly(expr, assembly, temps);
                    }
                    assembly.push_str(&format!("compl {} {}\n", first_op, second_op));
                    assembly.push_str(&format!("setl %al\n"));
                    assembly.push_str(&format!("movl %al %eax\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::LessEqual => {
                    if let Operand::Imm(_) = first {
                        let expr = IRExp::Exp(Box::new((e2, Op::GreaterThan, e1)));
                        return expr_to_assembly(expr, assembly, temps);
                    }
                    assembly.push_str(&format!("compl {} {}\n", first_op, second_op));
                    assembly.push_str(&format!("setle %al\n"));
                    assembly.push_str(&format!("movl %al %eax\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::GreaterThan => {
                    if let Operand::Imm(_) = first {
                        let expr = IRExp::Exp(Box::new((e2, Op::LessEqual, e1)));
                        return expr_to_assembly(expr, assembly, temps);
                    }
                    assembly.push_str(&format!("compl {} {}\n", first_op, second_op));
                    assembly.push_str(&format!("setg %al\n"));
                    assembly.push_str(&format!("movl %al %eax\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::GreaterEqual => {
                    if let Operand::Imm(_) = first {
                        let expr = IRExp::Exp(Box::new((e2, Op::LessThan, e1)));
                        return expr_to_assembly(expr, assembly, temps);
                    }
                    assembly.push_str(&format!("compl {} {}\n", first_op, second_op));
                    assembly.push_str(&format!("setge %al\n"));
                    assembly.push_str(&format!("movl %al %eax\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::Equals => {
                    if let Operand::Imm(_) = first {
                        assembly.push_str(&format!("compl {} {}\n", first_op, second_op));
                    } else {
                        assembly.push_str(&format!("compl {} {}\n", second_op, first_op));
                    }
                    assembly.push_str(&format!("sete %al\n"));
                    assembly.push_str(&format!("movl %al %eax\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::NotEqual => {
                    if let Operand::Imm(_) = first {
                        assembly.push_str(&format!("compl {} {}\n", first_op, second_op));
                    } else {
                        assembly.push_str(&format!("compl {} {}\n", second_op, first_op));
                    }
                    assembly.push_str(&format!("setne %al\n"));
                    assembly.push_str(&format!("movl %al %eax\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::BitAnd => {
                    assembly.push_str(&format!("movl {} %eax\n", first_op));
                    assembly.push_str(&format!("andl {} %eax\n", second_op));
                    Operand::Reg(0)
                }
                crate::ir::Op::BitXor => {
                    assembly.push_str(&format!("movl {} %eax\n", first_op));
                    assembly.push_str(&format!("xorl {} %eax\n", second_op));
                    Operand::Reg(0)
                }
                crate::ir::Op::BitOr => {
                    assembly.push_str(&format!("movl {} %eax\n", first_op));
                    assembly.push_str(&format!("andl {} %eax\n", second_op));
                    Operand::Reg(0)
                }
                /*movl    %edi, %eax
                movl    %esi, %ecx
                sall    %cl, %eax*/
                crate::ir::Op::LShift => {
                    assembly.push_str(&format!("movl {} %eax\n", first_op));
                    assembly.push_str(&format!("movl {} %ecx\n", second_op));
                    assembly.push_str(&format!("sall %cl %edx\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::RShift => {
                    assembly.push_str(&format!("movl {} %eax\n", first_op));
                    assembly.push_str(&format!("movl {} %ecx\n", second_op));
                    assembly.push_str(&format!("sall %cl %edx\n"));
                    Operand::Reg(0)
                }
            }
        }
    }
}

fn push_to_stack(stack_counter: usize) {}
