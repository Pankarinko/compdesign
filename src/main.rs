use lalrpop_util::lalrpop_mod;
use tokenizer::Token;

lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    parser
);

pub mod ast;
pub mod tokenizer;

fn main() {
    let mut tokens = Vec::new();
    let input = b"((32 + (1 / -6)))";
    tokenizer::tokenize(input, &mut tokens).unwrap();
    println!("{:?}", tokens);
    let lexer = tokens.into_iter();
    let ast = parser::ExpParser::new().parse(input, lexer).unwrap();
    println!("{:?}", ast);
}
