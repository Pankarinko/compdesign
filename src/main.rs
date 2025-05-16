use ast::Program;
use lalrpop_util::lalrpop_mod;
use semantics::return_check;
use tokenizer::Token;

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
    let input = b"int main () {int a = 5; return 0;}";
    if tokenizer::tokenize(input, &mut tokens).is_err() {
        todo!()
    }
    println!("{:?}", tokens);
    let lexer = tokens.into_iter();
    let ast: Program<'_>;
    if let Ok(result) = parser::ProgramParser::new().parse(input, lexer) {
        ast = result
    } else {
        todo!()
    }
    if !return_check(ast.get_statements()) {
        println!("stop");
        todo!()
    }
    println!("{:?}", ast);
}
