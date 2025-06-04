use std::{collections::HashMap, fs::File, io::Read, iter, process::exit};

use ast::Program;
use elaboration::translate_statement;
//use elaboration::translate_statement;
//use evaluation::execute;
use lalrpop_util::lalrpop_mod;
use semantics::{decl_check, return_check};
use tokenizer::{Token, tokenize};

use crate::semantics::type_check;

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    parser
);

pub mod ast;
pub mod elaboration;
//pub mod evaluation;
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
    if let Err(e) = tokenize(&input, &mut tokens) {
        println!("Error: Your program contains unknown tokens.");

        exit(e)
    }
    //println!("{:?}", &tokens);
    let lexer = tokens.into_iter();

    let ast: Program<'_>;

    if let Ok(result) = parser::ProgramParser::new().parse(&input, lexer) {
        ast = result;
        //println!("{:#?}", ast);
    } else {
        println!("Error: Your program cannot be parsed.");
        exit(42)
    }
    let tree = translate_statement(&mut iter::once(ast.get_block()).peekable());
    if !return_check(&tree) {
        println!("Error: Your program does not return.");
        exit(7)
    }
    //println!("{:#?}", tree);
    let mut declared = Vec::new();
    let mut assigned = Vec::new();
    if !decl_check(&tree, &mut assigned, &mut declared) {
        println!("Error: Your code has undeclared or unassigned variables.");
        exit(7)
    }
    let mut types = HashMap::new();
    if !type_check(&ast::Type::Int, &tree, &mut types) {
        exit(7);
    }
    /*
    if !return_check(ast.get_statements()) {
        exit(7)
    }
    if !decl_check(ast.get_statements()) {
        exit(7)
    }
    let string = args.next().unwrap().to_os_string();
    execute(ast, string);*/
}
