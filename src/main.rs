use std::{collections::HashMap, fs::File, io::Read, iter, process::exit};

use ast::Program;
//use elaboration::translate_statement;
//use elaboration::translate_statement;
//use evaluation::execute;
use lalrpop_util::lalrpop_mod;
//use semantics::{decl_check, return_check};
use tokenizer::{Token, tokenize};

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
/*pub mod code_gen;
pub mod elaboration;
pub mod instruction_selection;
pub mod ir;
pub mod semantics;*/
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
    /*
    /*Semantic analysis starts here*/
    if semantic_error {
        println!("Error: Invalid integer");
        exit(7)
    }
    let tree = translate_statement(&mut iter::once(ast.get_block()).peekable());

    //println!("{:?}", tree);
    if !return_check(&tree) {
        println!("Error: Your program does not return.");
        exit(7)
    }
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
    let loop_counter = 0;
    if !break_coninue_check(loop_counter, &tree) {
        println!("Error: Break and continue found outside of loop.");
        exit(7)
    }
    let mut program = Vec::new();
    let mut temp_count: usize = 0;
    let mut label_count: usize = 0;
    let label_cont = 0;
    let label_brk = 0;
    let mut vars: HashMap<&[u8], ir::IRExp> = HashMap::new();
    translate_to_ir(
        tree,
        &mut program,
        &mut temp_count,
        &mut label_count,
        &mut vars,
        label_cont,
        label_brk,
        None,
    );
    //println!("{:#?}", program);
    let string = args.next().unwrap().to_os_string();
    create_binary(program, temp_count, string);*/
}
