#[derive(Debug)]
pub enum ParserError {
    EOF,
    UnExpectedToken(&'static str),
    ObjectKey(&'static str),
    MemberExpr(&'static str),
    PrimaryExpr,
    ConstValueNull,
    ForLoopDeclaration(&'static str),
    InvalidClassStmt,
}

#[derive(Debug)]
pub enum RuntimeError {
    DeclareVar,
    UnidentifiedVar,
    ConstReassign,
    MisMatchTypes,
    MoreFuncArguments,
    LessFuncArguments,
    InvalidMemberExpr,
    ObjectField,

    UnReachableError, // Error should not occur but made to satify rust compiler
}

pub fn handle_lexer_error(line: usize, message: &str) {
    eprintln!("[line {}] Error: {}", line, message);
}

pub fn handle_parser_error(error: ParserError) {
    match error {
        ParserError::EOF => println!("Reached end of file without completion of program"),
        ParserError::UnExpectedToken(s) => println!("{s}"),
        _ => {}
    }
}

pub fn handle_runtime_error(error: RuntimeError) {
    println!("{:#?}", error);
    match error {
        _ => {}
    }
}
