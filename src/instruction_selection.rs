use std::collections::HashMap;

use crate::ir::{IRCmd, IRExp, Op};

#[derive(Clone)]
pub enum Operand {
    Reg(usize),
    Imm(i32),
}

impl Operand {
    pub fn get_op_string(self, temps: &HashMap<usize, String>) -> String {
        match self {
            Self::Imm(i) => String::from(format!("{}", i)),
            Self::Reg(r) => temps.get(&r).unwrap().clone(),
        }
    }
}

fn map_temp_to_register(
    num_temps: usize,
    stack_counter: usize,
    temp_index: Option<usize>,
) -> String {
    if let Some(index) = temp_index {
        if index <= 12 {
            match index {
                1 => return "ebx".to_string(),
                2 => return "edx".to_string(),
                3 => return "3si".to_string(),
                4 => return "3di".to_string(),
                5 => return "r8d".to_string(),
                6 => return "r9d".to_string(),
                7 => return "r10d".to_string(),
                8 => return "r11d".to_string(),
                9 => return "r12d".to_string(),
                10 => return "r13d".to_string(),
                11 => return "r14d".to_string(),
                12 => return "r15d".to_string(),
            }
        } else {
            let stack_i = (index - 12) * 4;
            return format!("DWORD PTR [rbp-{}]", stack_i).to_string();
        }
    }
    if num_temps <= 12 && stack_counter <= (12 - num_temps) {
        match num_temps + stack_counter {
            1 => return "ebx".to_string(),
            2 => return "edx".to_string(),
            3 => return "3si".to_string(),
            4 => return "3di".to_string(),
            5 => return "r8d".to_string(),
            6 => return "r9d".to_string(),
            7 => return "r10d".to_string(),
            8 => return "r11d".to_string(),
            9 => return "r12d".to_string(),
            10 => return "r13d".to_string(),
            11 => return "r14d".to_string(),
            _ => return "r15d".to_string(),
        }
    }
    return format!("DWORD PTR [rbp-{}]", stack_counter * 4).to_string();
}

fn translate_instruction(cmd: IRCmd, assembly: &mut String, temps: &mut HashMap<usize, String>) {
    match cmd {
        IRCmd::Load(irexp, irexp1) => {
            let operand = expr_to_assembly(irexp1, assembly, temps);
            if let IRExp::Temp(i) = irexp {
                assembly.push_str(&format!(
                    "mov {} {}\n",
                    temps.get(&(i + 1)).unwrap(),
                    operand.get_op_string(&temps)
                ));
            }
        }

        IRCmd::JumpIf(irexp, label) => {
            let operand = expr_to_assembly(irexp, assembly, temps);
            assembly.push_str(&format!("cmp {}, 0\n", operand.get_op_string(&temps)));
            assembly.push_str(&format!("jne _LABEL_{label}\n"));
        }
        IRCmd::Jump(label) => assembly.push_str(&format!("jmp {label}",)),
        IRCmd::Label(label) => assembly.push_str(&format!("_LABEL_{label}:")),
        IRCmd::Return(irexp) => {
            let operand = expr_to_assembly(irexp, assembly, temps);
            assembly.push_str(&format!("mov eax, {}\n", operand.get_op_string(&temps)));
        }
    }
}

fn constant_folding(e1: IRExp, op: Op, e2: IRExp, assembly: &mut String) -> Option<Operand> {
    if let IRExp::ConstInt(val1) = e1 {
        if let IRExp::ConstInt(val2) = e2 {
            match op {
                Op::Plus => {
                    assembly.push_str(&format!("mov eax, {}\n", val1 + val2));
                    return Some(Operand::Reg(0));
                }
                Op::Minus => {
                    assembly.push_str(&format!("mov eax, {}\n", val1 - val2));
                    return Some(Operand::Reg(0));
                }
                Op::Mult => {
                    assembly.push_str(&format!("mov eax {}\n", val1 * val2));
                    return Some(Operand::Reg(0));
                }
                Op::Div => None,
                Op::Mod => None,
                Op::LessThan => {
                    let mut val;
                    if val1 < val2 {
                        val = 1;
                    } else {
                        val = 0;
                    }
                    assembly.push_str(&format!("mov eax, {}\n", val));
                    return Some(Operand::Reg(0));
                }
                Op::LessEqual => {
                    let mut val;
                    if val1 <= val2 {
                        val = 1;
                    } else {
                        val = 0;
                    }
                    assembly.push_str(&format!("mov eax, {}\n", val));
                    return Some(Operand::Reg(0));
                }

                Op::GreaterThan => {
                    let mut val;
                    if val1 > val2 {
                        val = 1;
                    } else {
                        val = 0;
                    }
                    assembly.push_str(&format!("mov eax {}\n", val));
                    return Some(Operand::Reg(0));
                }
                Op::GreaterEqual => {
                    let mut val;
                    if val1 >= val2 {
                        val = 1;
                    } else {
                        val = 0;
                    }
                    assembly.push_str(&format!("mov eax {}\n", val));
                    return Some(Operand::Reg(0));
                }
                Op::Equals => {
                    let mut val;
                    if val1 == val2 {
                        val = 1;
                    } else {
                        val = 0;
                    }
                    assembly.push_str(&format!("mov eax {}\n", val));
                    return Some(Operand::Reg(0));
                }

                Op::NotEqual => {
                    let mut val;
                    if val1 != val2 {
                        val = 1;
                    } else {
                        val = 0;
                    }
                    assembly.push_str(&format!("mov eax {}\n", val));
                    return Some(Operand::Reg(0));
                }
                Op::BitAnd => {
                    assembly.push_str(&format!("mov eax {}\n", val1 & val2));
                    return Some(Operand::Reg(0));
                }

                Op::BitXor => {
                    assembly.push_str(&format!("mov eax {}\n", val1 ^ val2));
                    return Some(Operand::Reg(0));
                }

                Op::BitOr => {
                    assembly.push_str(&format!("mov eax {}\n", val1 | val2));
                    return Some(Operand::Reg(0));
                }
                Op::LShift => {
                    assembly.push_str(&format!("mov eax {}\n", val1 | val2));
                    return Some(Operand::Reg(0));
                }
                Op::RShift => {
                    assembly.push_str(&format!("mov eax {}\n", val1 | val2));
                    return Some(Operand::Reg(0));
                }
            };
            return None;
        }
    } else if let IRExp::ConstBool(val1) = e1 {
        if let IRExp::ConstBool(val2) = e2 {
            match op {
                Op::Equals => {
                    let mut val;
                    if val1 == val2 {
                        val = 1;
                    } else {
                        val = 0;
                    }
                    assembly.push_str(&format!("mov eax {}\n", val));
                    return Some(Operand::Reg(0));
                }

                Op::NotEqual => {
                    let mut val;
                    if val1 != val2 {
                        val = 1;
                    } else {
                        val = 0;
                    }
                    assembly.push_str(&format!("mov eax {}\n", val));
                    return Some(Operand::Reg(0));
                }
                _ => (),
            }
        }
    }
    return None;
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
            if val {
                return Operand::Imm(1);
            } else {
                return Operand::Imm(0);
            }
        }
        IRExp::Neg(irexp) => {
            let operand = expr_to_assembly(*irexp, assembly, temps);
            if let Operand::Reg(r) = operand.clone() {
                assembly.push_str(&format!("neg {}\n", temps.get(&r).unwrap()));
                operand
            } else {
                if let Operand::Imm(val) = operand {
                    assembly.push_str(&format!("mov eax, {}\n", val));
                    assembly.push_str(&format!("neg eax\n"));
                }
                Operand::Reg(0)
            }
        }
        IRExp::NotBool(irexp) => todo!(),
        IRExp::NotInt(irexp) => {
            let operand = expr_to_assembly(*irexp, assembly, temps);
            if let Operand::Reg(r) = operand.clone() {
                assembly.push_str(&format!("neg {}\n", temps.get(&r).unwrap()));
                operand
            } else {
                if let Operand::Imm(val) = operand {
                    assembly.push_str(&format!("mov eax {} \n", val));
                    assembly.push_str(&format!("not eax\n"));
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
                Operand::Reg(r) => first_op = format!("{}", temps.get(&r).unwrap()),
                Operand::Imm(i) => first_op = format!("{i}"),
            }
            match second {
                Operand::Reg(r) => second_op = format!("%}", temps.get(&r).unwrap()),
                Operand::Imm(i) => second_op = format!("{i}"),
            }
            match op {
                crate::ir::Op::Plus => {
                    assembly.push_str(&format!("mov eax {}\n", first_op));
                    assembly.push_str(&format!("add eax {}\n", second_op));
                    return Operand::Reg(0);
                }
                crate::ir::Op::Minus => {
                    assembly.push_str(&format!("mov eax  {}\n", first_op));
                    assembly.push_str(&format!("sub eax  {}\n", second_op));
                    return Operand::Reg(0);
                }
                crate::ir::Op::Mult => {
                    assembly.push_str(&format!("mov eax {}\n", first_op));
                    assembly.push_str(&format!("imul eax {}\n", second_op));
                    return Operand::Reg(0);
                }
                crate::ir::Op::Div => {
                    assembly.push_str(&format!("mov eax {}\n", first_op));
                    assembly.push_str(&format!("cdq\n"));
                    assembly.push_str(&format!("idiv {}\n", second_op));
                    return Operand::Reg(0);
                }
                crate::ir::Op::Mod => {
                    assembly.push_str(&format!("movl eax {}\n", first_op));
                    assembly.push_str(&format!("cdq\n"));
                    assembly.push_str(&format!("idiv {}\n", second_op));
                    return Operand::Reg(1);
                }
                crate::ir::Op::LessThan => {
                    if let Operand::Imm(_) = first {
                        let expr = IRExp::Exp(Box::new((e2, Op::GreaterEqual, e1)));
                        return expr_to_assembly(expr, assembly, temps);
                    }
                    assembly.push_str(&format!("compl {} {}\n", second_op, first_op));
                    assembly.push_str(&format!("setl al\n"));
                    assembly.push_str(&format!("movl eax al\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::LessEqual => {
                    if let Operand::Imm(_) = first {
                        let expr = IRExp::Exp(Box::new((e2, Op::GreaterThan, e1)));
                        return expr_to_assembly(expr, assembly, temps);
                    }
                    assembly.push_str(&format!("compl {} {}\n", second_op, first_op));
                    assembly.push_str(&format!("setle al\n"));
                    assembly.push_str(&format!("movl eax al\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::GreaterThan => {
                    if let Operand::Imm(_) = first {
                        let expr = IRExp::Exp(Box::new((e2, Op::LessEqual, e1)));
                        return expr_to_assembly(expr, assembly, temps);
                    }
                    assembly.push_str(&format!("compl {} {}\n", second_op, first_op));
                    assembly.push_str(&format!("setg al\n"));
                    assembly.push_str(&format!("movl eax al\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::GreaterEqual => {
                    if let Operand::Imm(_) = first {
                        let expr = IRExp::Exp(Box::new((e2, Op::LessThan, e1)));
                        return expr_to_assembly(expr, assembly, temps);
                    }
                    assembly.push_str(&format!("compl {} {}\n", second_op, first_op));
                    assembly.push_str(&format!("setge al\n"));
                    assembly.push_str(&format!("movl eax al\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::Equals => {
                    if let Operand::Imm(_) = first {
                        assembly.push_str(&format!("compl {} {}\n", second_op, first_op));
                    } else {
                        assembly.push_str(&format!("compl {} {}\n", first_op, second_op));
                    }
                    assembly.push_str(&format!("sete al\n"));
                    assembly.push_str(&format!("movl eax al\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::NotEqual => {
                    if let Operand::Imm(_) = first {
                        assembly.push_str(&format!("compl {} {}\n", second_op, first_op));
                    } else {
                        assembly.push_str(&format!("compl {} {}\n", first_op, second_op));
                    }
                    assembly.push_str(&format!("setne al\n"));
                    assembly.push_str(&format!("movl eax al\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::BitAnd => {
                    assembly.push_str(&format!("movl {} %eax\n", first_op));
                    assembly.push_str(&format!("andl {} %eax\n", second_op));
                    Operand::Reg(0)
                }
                crate::ir::Op::BitXor => {
                    assembly.push_str(&format!("movl eax {}\n", first_op));
                    assembly.push_str(&format!("xorl eax {}\n", second_op));
                    Operand::Reg(0)
                }
                crate::ir::Op::BitOr => {
                    assembly.push_str(&format!("movl eax {}\n", first_op));
                    assembly.push_str(&format!("andl eax {}\n", second_op));
                    Operand::Reg(0)
                }
                crate::ir::Op::LShift => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("mov ecx, {}\n", second_op));
                    assembly.push_str(&format!("sall eax cl\n"));
                    Operand::Reg(0)
                }
                crate::ir::Op::RShift => {
                    assembly.push_str(&format!("movl eax {}\n", first_op));
                    assembly.push_str(&format!("movl ecx {}\n", second_op));
                    assembly.push_str(&format!("sall eax cl\n"));
                    Operand::Reg(0)
                }
            }
        }
    }
}

fn push_to_stack(stack_counter: usize) {}
