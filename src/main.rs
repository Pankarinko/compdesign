use std::{collections::HashMap, fs::File, io::Read, iter, process::exit};

use ast::Program;
//use elaboration::translate_statement;
//use elaboration::translate_statement;
//use evaluation::execute;
use lalrpop_util::lalrpop_mod;
//use semantics::{decl_check, return_check};
use tokenizer::{Token, tokenize};

use crate::{code_gen::create_binary, ir::translate_to_ir, semantics::check_semantics};

/*use crate::{
    code_gen::create_binary,
    ir::{IRCmd, translate_to_ir},
    semantics::{break_coninue_check, type_check},
};*/

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    parser
);

pub mod ast;
pub mod code_gen;
pub mod elaboration;
pub mod instruction_selection;
pub mod ir;
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
                println!("Error: Path cannot be found.");
                exit(42);
            }
        },
        None => {
            println!("Error: Path cannot be found.");
            exit(42);
        }
    }
    let mut input = Vec::new();
    if file.read_to_end(&mut input).is_err() {
        println!("Unable to read file!");
        exit(42);
    };
    let mut semantic_error = false;
    if let Err(e) = tokenize(&input, &mut semantic_error, &mut tokens) {
        if e == 42 {
            println!("Error: Your program contains unknown tokens.");
            exit(e)
        }
    }
    //println!("{:#?}", &tokens);
    let lexer = tokens.into_iter();

    let ast: Program<'_>;
    if let Ok(result) = parser::ProgramParser::new().parse(&input, lexer) {
        ast = result;
        //println!("{:#?}", ast);
    } else {
        println!("Error: Your program cannot be parsed.");
        exit(42)
    }

    /*Semantic analysis starts here*/
    if semantic_error {
        println!("Error: Invalid integer");
        exit(7)
    }
    let funcs = check_semantics(ast);
    //println!("{:#?}", funcs);

    let program_in_ir = translate_to_ir(funcs);

    //println!("{:#?}", program_in_ir);
    let string = args.next().unwrap().to_os_string();
    create_binary(program_in_ir, string);
}
