use std::{fs::File, io::Read, iter, process::exit};

use ast::Program;
use elaboration::translate_statement;
//use elaboration::translate_statement;
//use evaluation::execute;
use lalrpop_util::lalrpop_mod;
use semantics::return_check;
//use semantics::{decl_check, return_check};
use tokenizer::{Token, tokenize};

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
    if file.read_to_end(&mut input).is_err() {
        println!("Unable to read file!");
        exit(42);
    };
    if let Err(e) = tokenize(&input, &mut tokens) {
        println!("Lexer failed {e}");

        exit(e)
    }
    //println!("{:?}", &tokens);
    let lexer = tokens.into_iter();

    let ast: Program<'_>;

    if let Ok(result) = parser::ProgramParser::new().parse(&input, lexer) {
        ast = result;
        //println!("{:#?}", ast);
    } else {
        println!("parser failed");
        exit(42)
    }
    let tree = translate_statement(&mut iter::once(ast.get_block()).peekable());
    println!("{:?}", tree);
    println!("{:?}", return_check(tree));
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
