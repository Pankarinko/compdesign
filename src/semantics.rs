use crate::ast::Statement;

pub fn return_check(statements: &Vec<Statement<'_>>) -> bool {
    return statements.iter().any(|s| match s {
        Statement::Return(_) => return true,
        _ => return false,
    });
}
