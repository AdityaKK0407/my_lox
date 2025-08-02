#[derive(Debug)]
pub enum ParserError {
    EOF,
    UnExpectedToken(String, usize),
    ObjectKey(String, usize),
    MemberExpr(usize),
    PrimaryExpr(String, usize),
    ConstValueNull(usize),
    ForLoopDeclaration(String, usize),
    ScopeError(String, usize),
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

pub fn handle_lexer_error(line: usize, message: &str, code: &str) {
    eprintln!("Line {}: {}", line, code);
    eprintln!("Error: {}", message);
}

pub fn handle_parser_error(error: ParserError, code: &[String]) {
    match error {
        ParserError::EOF => eprintln!("Unexpected end of file: incomplete program structure"),
        ParserError::UnExpectedToken(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1].trim());
            eprintln!("Error: {}", s);
        },
        ParserError::ObjectKey(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1].trim());
            eprintln!(
                "Error: Expected string or identifier for object keys. {}",
                s
            );
        },
        ParserError::ConstValueNull(line) => {
            eprintln!("Line {}: {}", line, code[line - 1].trim());
            eprintln!("Error: Constant variable is not initialized.")
        },
        ParserError::ForLoopDeclaration(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1].trim());
            eprintln!("Error: Invalid for loop declaration. {}", s);
        },
        ParserError::MemberExpr(line) => {
            eprintln!("Line {}: {}", line, code[line - 1].trim());
            eprintln!(
                "Error: Expected identifier or 'this' and 'super' keywords before dot operator"
            );
        },
        ParserError::PrimaryExpr(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1].trim());
            eprintln!("Error: Invalid expression. Found '{}'", s);
        },
        ParserError::ScopeError(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1].trim());
            eprintln!("Error: {}", s);            
        },
    }
}

pub fn handle_runtime_error(error: RuntimeError) {
    println!("{:#?}", error);
    match error {
        _ => {}
    }
}
