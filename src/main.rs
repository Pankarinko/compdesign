use std::{
    fmt::format,
    fs::File,
    io::{Read, Write},
    process::{Command, Stdio, exit},
};

use ast::Program;
use evaluation::create_binary;
use lalrpop_util::lalrpop_mod;
use semantics::{decl_check, return_check};
use tokenizer::{Token, tokenize};

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    parser
);

pub mod ast;
pub mod evaluation;
pub mod semantics;
pub mod tokenizer;

fn main() {
    let mut tokens = Vec::new();
    let mut args = std::env::args_os().skip(1);
    let path = args.next();
    let mut file;
    match path {
        Some(input_path) => match File::open(input_path) {
            Ok(f) => file = f,
            Err(_) => {
                println!("path not found!");
                exit(42);
            }
        },
        None => {
            println!("path not found!");
            exit(42);
        }
    }
    let mut input = Vec::new();
    if let Err(_) = file.read_to_end(&mut input) {
        println!("Unable to read file!");
        exit(42);
    };
    /*let v = b"int main() {
int _Th1S_1S_th3_B3stt_V4r_N4ME_I_c0uld_TH1NK_of_ = 2;
return _Th1S_1S_th3_B3stt_V4r_N4ME_I_c0uld_TH1NK_of_*_Th1S_1S_th3_B3stt_V4r_N4ME_I_c0uld_TH1NK_of_;}
";
    let input = &v[..];*/
    if let Err(e) = tokenize(&input, &mut tokens) {
        exit(e)
    }
    let lexer = tokens.into_iter();
    let ast: Program<'_>;
    if let Ok(result) = parser::ProgramParser::new().parse(&input, lexer) {
        ast = result
    } else {
        exit(42)
    }
    if !return_check(ast.get_statements()) {
        println!("stop");
        exit(7)
    }
    if !decl_check(ast.get_statements()) {
        println!("stop 2");
        exit(7)
    }
    let string = args.next().unwrap().to_os_string();
    create_binary(ast, string);
}
