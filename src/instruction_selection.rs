use std::num;

use crate::ir::{IRCmd, IRExp};

pub fn init_stack_counter(num_temps: usize) -> usize {
    num_temps.saturating_sub(12)
}
fn map_temp_to_register(
    num_temps: usize,
    stack_counter: &mut usize,
    temp_index: Option<usize>,
) -> String {
    if let Some(index) = temp_index {
        if index <= 12 {
            match index {
                0 => return "ebx".to_string(),
                1 => return "ecx".to_string(),
                2 => return "esi".to_string(),
                3 => return "edi".to_string(),
                4 => return "r8d".to_string(),
                5 => return "r9d".to_string(),
                6 => return "r10d".to_string(),
                7 => return "r11d".to_string(),
                8 => return "r12d".to_string(),
                9 => return "r13d".to_string(),
                10 => return "r14d".to_string(),
                11 => return "r15d".to_string(),
                _ => {
                    *stack_counter += 1;
                    return "DWORD PTR [rsp-4]".to_string();
                }
            }
        } else {
            let stack_i = (index - 11) * 4;
            return format!("DWORD PTR [rsp-{}]", stack_i).to_string();
        }
    }
    if num_temps <= 12 && *stack_counter <= (12 - num_temps) {
        let stack_i = *stack_counter;
        *stack_counter += 1;
        match num_temps + stack_i {
            0 => return "ebx".to_string(),
            1 => return "ecx".to_string(),
            2 => return "esi".to_string(),
            3 => return "edi".to_string(),
            4 => return "r8d".to_string(),
            5 => return "r9d".to_string(),
            6 => return "r10d".to_string(),
            7 => return "r11d".to_string(),
            8 => return "r12d".to_string(),
            9 => return "r13d".to_string(),
            10 => return "r14d".to_string(),
            11 => return "r15d".to_string(),
            _ => {
                *stack_counter += 1;
                return "DWORD PTR [rsp-4]".to_string();
            }
        }
    }
    let s_i = *stack_counter;
    *stack_counter += 1;
    format!("DWORD PTR [rsp-{}]", s_i * 4).to_string()
}

pub fn translate_instruction(
    num_temps: usize,
    stack_counter: &mut usize,
    cmd: IRCmd,
    assembly: &mut String,
) {
    match cmd {
        IRCmd::Load(irexp, irexp1) => {
            let operand = expr_to_assembly(num_temps, stack_counter, irexp1, assembly);
            if let IRExp::Temp(i) = irexp {
                let r = map_temp_to_register(num_temps, stack_counter, Some(i));
                assembly.push_str(&format!("mov eax, {}\n", operand));
                assembly.push_str(&format!("mov {}, eax\n", r));
            }
        }

        IRCmd::JumpIf(irexp, label) => {
            let operand = expr_to_assembly(num_temps, stack_counter, irexp, assembly);
            assembly.push_str(&format!("cmp {}, 1\n", operand));
            assembly.push_str(&format!("je _LABEL_{label}\n"));
        }
        IRCmd::Jump(label) => assembly.push_str(&format!("jmp _LABEL_{label}\n",)),
        IRCmd::Label(label) => assembly.push_str(&format!("_LABEL_{label}:\n")),
        IRCmd::Return(irexp) => {
            let operand = expr_to_assembly(num_temps, stack_counter, irexp, assembly);
            assembly.push_str(&format!("mov eax, {}\n", operand));
            assembly.push_str("ret\n");
        }
    }
}

fn expr_to_assembly(
    num_temps: usize,
    stack_counter: &mut usize,
    expr: IRExp,
    assembly: &mut String,
) -> String {
    match expr {
        IRExp::Temp(t) => map_temp_to_register(num_temps, stack_counter, Some(t)),
        IRExp::ConstInt(val) => {
            let r = map_temp_to_register(num_temps, stack_counter, None);
            assembly.push_str(&format!("mov {}, {}\n", r, val));
            r
        }
        IRExp::ConstBool(val) => {
            let r = map_temp_to_register(num_temps, stack_counter, None);
            if val {
                assembly.push_str(&format!("mov {}, {}\n", r, 1));
            } else {
                assembly.push_str(&format!("mov {}, {}\n", r, 0));
            }
            r
        }
        IRExp::Neg(irexp) => {
            let r = expr_to_assembly(num_temps, stack_counter, *irexp, assembly);
            let new_r = map_temp_to_register(num_temps, stack_counter, None);
            assembly.push_str(&format!("mov eax, {}\n", r));
            assembly.push_str(&format!("mov {}, eax\n", new_r));
            assembly.push_str(&format!("neg {}\n", new_r));
            new_r
        }
        IRExp::NotBool(irexp) => {
            let r = expr_to_assembly(num_temps, stack_counter, *irexp, assembly);
            let new_r = map_temp_to_register(num_temps, stack_counter, None);
            assembly.push_str(&format!("mov eax, {}\n", r));
            assembly.push_str(&format!("mov {}, eax\n", new_r));
            assembly.push_str(&format!("xor {}, 1\n", new_r));
            new_r
        }
        IRExp::NotInt(irexp) => {
            let r = expr_to_assembly(num_temps, stack_counter, *irexp, assembly);
            let new_r = map_temp_to_register(num_temps, stack_counter, None);
            assembly.push_str(&format!("mov eax, {}\n", r));
            assembly.push_str(&format!("mov {}, eax\n", new_r));
            assembly.push_str(&format!("not {}\n", new_r));
            new_r
        }
        IRExp::Exp(b) => {
            let (e1, op, e2) = *b;
            let first_op = expr_to_assembly(num_temps, stack_counter, e1, assembly);
            let second_op = expr_to_assembly(num_temps, stack_counter, e2, assembly);
            let new_r = map_temp_to_register(num_temps, stack_counter, None);
            match op {
                crate::ir::Op::Plus => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("add eax, {}\n", second_op));
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::Minus => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("sub eax, {}\n", second_op));
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::Mult => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("imul eax, {}\n", second_op));
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::Div => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str("cdq\n");
                    assembly.push_str(&format!("idiv {}\n", second_op));
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::Mod => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str("cdq\n");
                    assembly.push_str(&format!("idiv {}\n", second_op));
                    assembly.push_str(&format!("mov {}, edx\n", new_r));
                }
                crate::ir::Op::LessThan => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("cmp eax, {}\n", second_op));
                    assembly.push_str("setl al\n");
                    assembly.push_str("movzx eax, al\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::LessEqual => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("cmp eax, {}\n", second_op));
                    assembly.push_str("setle al\n");
                    assembly.push_str("movzx eax, al\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::GreaterThan => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("cmp eax, {}\n", second_op));
                    assembly.push_str("setg al\n");
                    assembly.push_str("movzx eax, al\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::GreaterEqual => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("cmp eax, {}\n", second_op));
                    assembly.push_str("setge al\n");
                    assembly.push_str("movzx eax, al\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::Equals => {
                    assembly.push_str("sete al\n");
                    assembly.push_str("mov eax, al\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::NotEqual => {
                    assembly.push_str("setne al\n");
                    assembly.push_str("mov eax, al\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::BitAnd => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("and eax, {}\n", second_op));
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::BitXor => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("xor eax, {}\n", second_op));
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::BitOr => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("and eax, {}\n", second_op));
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::LShift => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("mov ecx, {}\n", second_op));
                    assembly.push_str("sall eax, cl\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::RShift => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("mov ecx, {}\n", second_op));
                    assembly.push_str("sall eax, cl\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
            }
            new_r
        }
    }
}
