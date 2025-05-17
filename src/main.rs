use std::{fs::File, io::Read, process::exit};

use ast::Program;
use lalrpop_util::lalrpop_mod;
use semantics::{decl_check, return_check};
use tokenizer::{Token, tokenize};

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    parser
);

pub mod ast;
pub mod semantics;
pub mod tokenizer;

fn main() {
    let mut tokens = Vec::new();
    let args = std::env::args_os();
    let path = args.skip(1).next();
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
    /*let v = b"int main() { }";
    let input = &v[..];*/
    if let Err(e) = tokenize(&input, &mut tokens) {
        if e == 7 {
            exit(42);
        } else {
            exit(42);
        }
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
}
