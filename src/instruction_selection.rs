use crate::ir::{IRCmd, IRExp};

pub fn init_stack_counter(num_temps: usize) -> usize {
    (num_temps + 1).saturating_sub(7)
}

pub fn translate_functions(funcs: Vec<(&[u8], usize, usize, Vec<IRCmd>)>, assembly: &mut String) {
    let main = funcs
        .iter()
        .find(|(name, _, _, _)| name == b"main")
        .unwrap();
    let temp_count = main.1;
    let mut stack_counter = init_stack_counter(main.1);
    for cmd in (main.3).iter().cloned() {
        translate_instruction(temp_count, &mut stack_counter, cmd, assembly);
    }
    for f in funcs.iter().filter(|(name, _, _, _)| name != b"main") {
        assembly.push_str(&format!("\n{}:\n", str::from_utf8(f.0).unwrap().to_owned()));
        let temp_count = f.1;
        let mut stack_counter = init_stack_counter(f.1);
        for cmd in (f.3).iter().cloned() {
            move_params(f.2, assembly);
            translate_instruction(temp_count, &mut stack_counter, cmd, assembly);
        }
    }
}

fn move_params(num_args: usize, assembly: &mut String) {
    let params_regs = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
    let local_regs = ["ebx", "edi", "esi", "r8d", "r9d", "r10d"];
    let mut i = 0;
    while i < num_args && i < params_regs.len() {
        assembly.push_str(&format!("mov {}, {}\n", local_regs[i], params_regs[i]));
        i += 1;
    }
}

fn move_args(vals: Vec<IRExp>, assembly: &mut String, num_temps: usize, stack_counter: &mut usize) {
    let args_regs = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
    let mut i = 0;
    let mut new_stack_counter = 7;
    for val in vals.iter() {
        if i < args_regs.len() {
            let operand = expr_to_assembly(num_temps, stack_counter, val.clone(), assembly);
            assembly.push_str(&format!("mov {}, {}\n", args_regs[i], operand));
            i += 1;
        } else {
            let operand = expr_to_assembly(num_temps, stack_counter, val.clone(), assembly);
            assembly.push_str(&format!(
                "mov DWORD PTR [rsp-{}], {}\n",
                new_stack_counter, operand
            ));
            new_stack_counter += 1;
        }
    }
}

fn save_register_onto_stack(assembly: &mut String) {
    assembly.push_str("push rbx\n");
    assembly.push_str("push rsi\n");
    assembly.push_str("push rdi\n");
    assembly.push_str("push r8\n");
    assembly.push_str("push r9\n");
    assembly.push_str("push r10\n");
    assembly.push_str("push r11\n");
}

fn get_register_from_stack(assembly: &mut String) {
    assembly.push_str("pop r11\n");
    assembly.push_str("pop r10\n");
    assembly.push_str("pop r9\n");
    assembly.push_str("pop r8\n");
    assembly.push_str("pop rdi\n");
    assembly.push_str("pop rsi\n");
    assembly.push_str("pop rbx\n");
}

fn map_temp_to_register(
    num_temps: usize,
    stack_counter: &mut usize,
    temp_index: Option<usize>,
) -> String {
    if let Some(index) = temp_index {
        if index < 11 {
            match index {
                0 => return "ebx".to_owned(),
                1 => return "edi".to_owned(),
                2 => return "esi".to_owned(),
                3 => return "r8d".to_owned(),
                4 => return "r9d".to_owned(),
                5 => return "r10d".to_owned(),
                6 => return "r11d".to_owned(),
                7 => return "r12d".to_owned(),
                8 => return "r13d".to_owned(),
                9 => return "r14d".to_owned(),
                _ => return "r15d".to_owned(),
            }
        } else {
            let stack_i = (index - 10 + 7) * 4;
            return format!("DWORD PTR [rsp-{}]", stack_i).to_string();
        }
    }
    if num_temps < 11 && *stack_counter < (11 - num_temps) {
        let stack_i = *stack_counter;
        *stack_counter += 1;
        match num_temps + stack_i {
            0 => return "ebx".to_owned(),
            1 => return "edi".to_owned(),
            2 => return "esi".to_owned(),
            3 => return "r8d".to_owned(),
            4 => return "r9d".to_owned(),
            5 => return "r10d".to_owned(),
            6 => return "r11d".to_owned(),
            7 => return "r12d".to_owned(),
            8 => return "r13d".to_owned(),
            9 => return "r14d".to_owned(),
            _ => return "r15d".to_owned(),
        }
    }
    let s_i = *stack_counter + 7;
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
        IRCmd::Call(call) => match call {
            crate::ir::Call::Print(irexp) => {
                save_register_onto_stack(assembly);
                let old_stack_counter = *stack_counter;
                let operand = expr_to_assembly(num_temps, stack_counter, irexp, assembly);
                *stack_counter = old_stack_counter;
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str(&format!("mov edi, {}\n", operand));
                assembly.push_str("call putchar\n");
                assembly.push_str("mov rdi, QWORD PTR stdout[rip]\n");
                assembly.push_str("call fflush\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
            }
            crate::ir::Call::Read => {
                assembly.push_str(&format!("mov eax, {}\n", *stack_counter * 4));
                save_register_onto_stack(assembly);
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str("call getchar\n");
                assembly.push_str("call fflush\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
            }
            crate::ir::Call::Flush => {
                assembly.push_str(&format!("mov eax, {}\n", *stack_counter * 4));
                save_register_onto_stack(assembly);
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str("mov rdi, QWORD PTR stdout[rip]\n");
                assembly.push_str("call fflush\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
            }
            crate::ir::Call::Func(name, args) => {
                save_register_onto_stack(assembly);
                let old_stack_counter = *stack_counter;
                move_args(args, assembly, num_temps, stack_counter);
                *stack_counter = old_stack_counter;
                assembly.push_str(&format!("call {}\n", name));
                get_register_from_stack(assembly);
            }
        },
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
                assembly.push_str(&format!("mov {}, 1\n", r));
            } else {
                assembly.push_str(&format!("mov {}, 0\n", r));
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
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("cmp eax, {}\n", second_op));
                    assembly.push_str("sete al\n");
                    assembly.push_str("movzx eax, al\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::NotEqual => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("cmp eax, {}\n", second_op));
                    assembly.push_str("setne al\n");
                    assembly.push_str("movzx eax, al\n");
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
                    assembly.push_str(&format!("or eax, {}\n", second_op));
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::LShift => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("mov ecx, {}\n", second_op));
                    assembly.push_str("sal eax, cl\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
                crate::ir::Op::RShift => {
                    assembly.push_str(&format!("mov eax, {}\n", first_op));
                    assembly.push_str(&format!("mov ecx, {}\n", second_op));
                    assembly.push_str("sar eax, cl\n");
                    assembly.push_str(&format!("mov {}, eax\n", new_r));
                }
            }
            new_r
        }
        IRExp::Call(call) => match *call {
            crate::ir::Call::Print(irexp) => {
                save_register_onto_stack(assembly);
                let old_stack_counter = *stack_counter;
                let operand = expr_to_assembly(num_temps, stack_counter, irexp, assembly);
                *stack_counter = old_stack_counter;
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str(&format!("mov edi, {}\n", operand));
                assembly.push_str("call putchar\n");
                assembly.push_str("mov rdi, QWORD PTR stdout[rip]\n");
                assembly.push_str("call fflush\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
                let r = map_temp_to_register(num_temps, stack_counter, None);
                assembly.push_str(&format!("mov {}, eax\n", r));
                r
            }
            crate::ir::Call::Read => {
                assembly.push_str(&format!("mov eax, {}\n", *stack_counter * 4));
                save_register_onto_stack(assembly);
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str("call getchar\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
                let r = map_temp_to_register(num_temps, stack_counter, None);
                assembly.push_str(&format!("mov {}, eax\n", r));
                r
            }
            crate::ir::Call::Flush => {
                assembly.push_str(&format!("mov eax, {}\n", *stack_counter * 4));
                save_register_onto_stack(assembly);
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str("mov rdi, QWORD PTR stdout[rip]\n");
                assembly.push_str("call fflush\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
                let r = map_temp_to_register(num_temps, stack_counter, None);
                assembly.push_str(&format!("mov {}, eax\n", r));
                r
            }
            crate::ir::Call::Func(name, args) => {
                save_register_onto_stack(assembly);
                let old_stack_counter = *stack_counter;
                move_args(args, assembly, num_temps, stack_counter);
                *stack_counter = old_stack_counter;
                assembly.push_str(&format!("call {}\n", name));
                get_register_from_stack(assembly);
                let r = map_temp_to_register(num_temps, stack_counter, None);
                assembly.push_str(&format!("mov {}, eax\n", r));
                r
            }
        },
    }
}
