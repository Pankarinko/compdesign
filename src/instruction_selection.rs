use std::fmt::format;

use crate::{
    coloring::color_func,
    ir::{IRCmd, IRExp, IRFunction},
};

pub fn init_stack_counter(num_temps: usize) -> usize {
    (num_temps + 1).saturating_sub(7)
}

pub fn translate_functions(funcs: &mut Vec<IRFunction<'_>>, assembly: &mut String) {
    let mut main = funcs.iter_mut().find(|f| f.name == b"main").unwrap();
    let coloring = color_func(&mut main);
    let temp_count = main.num_temps;
    let mut stack_counter = init_stack_counter(main.num_temps);
    for cmd in (main.instructions).iter().cloned() {
        translate_instruction(
            temp_count,
            &mut stack_counter,
            cmd,
            assembly,
            &coloring,
            "eax".to_owned(),
        );
    }
    for f in funcs.iter_mut().filter(|f| f.name != b"main") {
        assembly.push_str(&format!(
            "\n_{}:\n",
            str::from_utf8(f.name).unwrap().to_owned()
        ));
        let coloring = color_func(f);
        let temp_count = f.num_temps;
        let mut stack_counter = init_stack_counter(f.num_temps);
        move_params(f.num_params, assembly);
        for cmd in (f.instructions).iter().cloned() {
            translate_instruction(
                temp_count,
                &mut stack_counter,
                cmd,
                assembly,
                &coloring,
                "eax".to_owned(),
            );
        }
    }
}

fn move_params(num_params: usize, assembly: &mut String) {
    let params_regs = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
    let local_regs = ["ebx", "edi", "esi", "r8d", "r9d", "r10d"];
    let mut i = 0;
    while i < num_params && i < params_regs.len() {
        assembly.push_str(&format!("mov {}, {}\n", local_regs[i], params_regs[i]));
        i += 1;
    }
}

fn move_args(
    args: Vec<IRExp>,
    assembly: &mut String,
    num_temps: usize,
    stack_counter: &mut usize,
    coloring: &Vec<usize>,
    current_temp: &mut str,
) {
    let args_regs = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
    let mut i = 0;
    let mut new_stack_counter = 8;
    while i < args.len() && i < args_regs.len() {
        let operand = expr_to_assembly(
            num_temps,
            stack_counter,
            args[i].clone(),
            assembly,
            coloring,
            current_temp,
        );
        assembly.push_str(&format!(
            "mov DWORD PTR [rsp-{}], {operand}\n",
            new_stack_counter * 4,
        ));
        i += 1;
        new_stack_counter += 1;
    }
    let mut j = 0;
    new_stack_counter = 8;
    while j < i {
        assembly.push_str(&format!(
            "mov {}, DWORD PTR [rsp-{}]\n",
            args_regs[j],
            new_stack_counter * 4
        ));
        j += 1;
        new_stack_counter += 1;
    }
    new_stack_counter = 8;
    while i < args.len() {
        let operand = expr_to_assembly(
            num_temps,
            stack_counter,
            args[i].clone(),
            assembly,
            coloring,
            current_temp,
        );
        assembly.push_str(&format!("mov eax, {operand}\n"));
        assembly.push_str(&format!(
            "mov DWORD PTR [rsp-{}], eax\n",
            new_stack_counter * 4
        ));
        new_stack_counter += 1;
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

fn map_temp_to_register(color: usize, load: bool, assembly: &mut String) -> String {
    if color <= 11 {
        match color {
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
        let stack_i = (color - 10 + 7) * 4;
        if load {
            return format!("DWORD PTR [rsp-{}]", stack_i).to_string();
        } else {
            assembly.push_str(&format!("mov eax, DWORD PTR [rsp-{}]\n", stack_i));
            return "eax".to_owned();
        }
    }
}

pub fn translate_instruction(
    num_temps: usize,
    stack_counter: &mut usize,
    cmd: IRCmd,
    assembly: &mut String,
    coloring: &Vec<usize>,
    mut current_temp: String,
) {
    match cmd {
        IRCmd::Load(irexp, irexp1) => {
            let operand = expr_to_assembly(
                num_temps,
                stack_counter,
                irexp1,
                assembly,
                coloring,
                &mut current_temp,
            );
            if let IRExp::Temp(i) = irexp {
                let r = map_temp_to_register(coloring[i.name], true, assembly);
                println!("{:?}", r);
                println!("{:?}", operand);
                assembly.push_str(&format!("mov {r}, {operand}\n"));
                current_temp = r;
            }
        }
        IRCmd::JumpIf(irexp, label) => {
            let operand = expr_to_assembly(
                num_temps,
                stack_counter,
                irexp,
                assembly,
                coloring,
                &mut current_temp,
            );
            assembly.push_str(&format!("cmp {operand}, 1\n"));

            assembly.push_str(&format!("je _LABEL_{label}\n"));
        }
        IRCmd::Jump(label) => assembly.push_str(&format!("jmp _LABEL_{label}\n",)),
        IRCmd::Label(label) => assembly.push_str(&format!("_LABEL_{label}:\n")),
        IRCmd::Return(irexp) => {
            let operand = expr_to_assembly(
                num_temps,
                stack_counter,
                irexp,
                assembly,
                coloring,
                &mut current_temp,
            );

            assembly.push_str(&format!("mov ebx, {operand}\n"));

            assembly.push_str("mov rdi, QWORD PTR stdout[rip]\n");
            assembly.push_str("call fflush\n");
            assembly.push_str("mov eax, ebx\n");
            assembly.push_str("ret\n");
        }
        IRCmd::Call(call) => match call {
            crate::ir::Call::Print(irexp) => {
                save_register_onto_stack(assembly);
                let old_stack_counter = *stack_counter;
                let operand = expr_to_assembly(
                    num_temps,
                    stack_counter,
                    irexp,
                    assembly,
                    coloring,
                    &mut current_temp,
                );
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str(&format!("mov edi, {operand}\n"));

                *stack_counter = old_stack_counter;
                assembly.push_str("call putchar\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
            }
            crate::ir::Call::Read => {
                assembly.push_str(&format!("mov eax, {}\n", *stack_counter * 4));
                save_register_onto_stack(assembly);
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str("call getchar\n");
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
                move_args(
                    args,
                    assembly,
                    num_temps,
                    stack_counter,
                    coloring,
                    &mut current_temp,
                );
                *stack_counter = old_stack_counter;
                assembly.push_str(&format!("call {name}\n"));
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
    coloring: &Vec<usize>,
    current_temp: &mut str,
) -> String {
    match expr {
        IRExp::Temp(t) => map_temp_to_register(coloring[t.name], false, assembly),
        IRExp::ConstInt(val) => format!("{val}"),
        IRExp::ConstBool(val) => {
            if val {
                assembly.push_str(&format!("mov eax, 1\n"));
            } else {
                assembly.push_str(&format!("mov eax, 0\n"));
            }
            format!("eax")
        }
        IRExp::Neg(irexp) => {
            assembly.push_str(&format!("neg {current_temp}\n"));
            format!("{current_temp}")
        }
        IRExp::NotBool(irexp) => {
            assembly.push_str(&format!("xor {current_temp}, 1\n"));
            format!("{current_temp}")
        }
        IRExp::NotInt(irexp) => {
            assembly.push_str(&format!("not {current_temp}\n"));
            format!("{current_temp}")
        }
        IRExp::Exp(b) => {
            let (e1, op, e2) = *b;
            let first_op = expr_to_assembly(
                num_temps,
                stack_counter,
                e1,
                assembly,
                coloring,
                current_temp,
            );
            let second_op = expr_to_assembly(
                num_temps,
                stack_counter,
                e2,
                assembly,
                coloring,
                current_temp,
            );
            match op {
                crate::ir::Op::Plus => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("add eax, {second_op}\n"));
                }
                crate::ir::Op::Minus => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("sub eax, {second_op}\n"));
                }
                crate::ir::Op::Mult => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("imul eax, {second_op}\n"));
                }
                crate::ir::Op::Div => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str("cdq\n");
                    assembly.push_str(&format!("idiv {second_op}\n"));
                }
                crate::ir::Op::Mod => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str("cdq\n");
                    assembly.push_str(&format!("idiv {second_op}\n"));
                    return "edx".to_owned();
                }
                crate::ir::Op::LessThan => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("cmp eax, {second_op}\n"));
                    assembly.push_str("setl al\n");
                    assembly.push_str("movzx eax, al\n");
                }
                crate::ir::Op::LessEqual => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("cmp eax, {second_op}\n"));
                    assembly.push_str("setle al\n");
                    assembly.push_str("movzx eax, al\n");
                }
                crate::ir::Op::GreaterThan => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("cmp eax, {second_op}\n"));
                    assembly.push_str("setg al\n");
                    assembly.push_str("movzx eax, al\n");
                }
                crate::ir::Op::GreaterEqual => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("cmp eax, {second_op}\n"));
                    assembly.push_str("setge al\n");
                    assembly.push_str("movzx eax, al\n");
                }
                crate::ir::Op::Equals => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("cmp eax, {second_op}\n"));
                    assembly.push_str("sete al\n");
                    assembly.push_str("movzx eax, al\n");
                }
                crate::ir::Op::NotEqual => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("cmp eax, {second_op}\n"));
                    assembly.push_str("setne al\n");
                    assembly.push_str("movzx eax, al\n");
                }
                crate::ir::Op::BitAnd => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("and eax, {second_op}\n"));
                }
                crate::ir::Op::BitXor => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("xor eax, {second_op}\n"));
                }
                crate::ir::Op::BitOr => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("or eax, {second_op}\n"));
                }
                crate::ir::Op::LShift => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("mov ecx, {second_op}\n"));
                    assembly.push_str("sal eax, cl\n");
                }
                crate::ir::Op::RShift => {
                    assembly.push_str(&format!("mov eax, {first_op}\n"));
                    assembly.push_str(&format!("mov ecx, {second_op}\n"));
                    assembly.push_str("sar eax, cl\n");
                }
            }
            "eax".to_owned()
        }
        IRExp::Call(call) => match *call {
            crate::ir::Call::Print(irexp) => {
                save_register_onto_stack(assembly);
                let old_stack_counter = *stack_counter;
                let operand = expr_to_assembly(
                    num_temps,
                    stack_counter,
                    irexp,
                    assembly,
                    coloring,
                    current_temp,
                );
                *stack_counter = old_stack_counter;
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str(&format!("mov edi, {operand}\n"));
                assembly.push_str("call putchar\n");
                assembly.push_str("mov rdi, QWORD PTR stdout[rip]\n");
                assembly.push_str("call fflush\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
                "eax".to_owned()
            }
            crate::ir::Call::Read => {
                assembly.push_str(&format!("mov eax, {}\n", *stack_counter * 4));
                save_register_onto_stack(assembly);
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str("call getchar\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
                "eax".to_owned()
            }
            crate::ir::Call::Flush => {
                assembly.push_str(&format!("mov eax, {}\n", *stack_counter * 4));
                save_register_onto_stack(assembly);
                assembly.push_str("sub rsp, 8\n");
                assembly.push_str("mov rdi, QWORD PTR stdout[rip]\n");
                assembly.push_str("call fflush\n");
                assembly.push_str("add rsp, 8\n");
                get_register_from_stack(assembly);
                "eax".to_owned()
            }
            crate::ir::Call::Func(name, args) => {
                save_register_onto_stack(assembly);
                let old_stack_counter = *stack_counter;
                move_args(
                    args,
                    assembly,
                    num_temps,
                    stack_counter,
                    coloring,
                    current_temp,
                );
                *stack_counter = old_stack_counter;
                assembly.push_str(&format!("call {name}\n"));
                get_register_from_stack(assembly);
                "eax".to_owned()
            }
        },
    }
}
