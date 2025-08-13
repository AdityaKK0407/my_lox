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
    TypeMismatch(String, usize),

    TypeCastingError(String, usize),

    InvalidArgumentCount(String, usize),

    ArrayIndexOutOfBounds(String, usize),
    InvalidArrayIndex(String, usize),

    InvalidMemberAccess(String, usize),
    UndefinedField(String, usize),
    UndefinedProperty(String, usize),

    EnvironmentError(String, usize),

    InternalError, // Error should not occur but made to satisfy rust compiler
}

#[derive(Debug, PartialEq)]
pub enum EnvironmentError {
    ReDeclareVar,
    ConstReassign,
    VarNotDeclared,
}

pub fn handle_lexer_error(line: usize, message: &str, code: &str) {
    eprintln!("Line {}: {}", line, code);
    eprintln!("Error: {}", message);
}

pub fn handle_parser_error(error: ParserError, code: &[&str]) {
    match error {
        ParserError::EOF => eprintln!("Unexpected end of file: incomplete program structure"),

        ParserError::UnExpectedToken(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }

        ParserError::ObjectKey(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!(
                "Error: Expected string or identifier for object keys. {}",
                s
            );
        }

        ParserError::ConstValueNull(line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: Constant variable is not initialized.")
        }

        ParserError::ForLoopDeclaration(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: Invalid for loop declaration. {}", s);
        }

        ParserError::MemberExpr(line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!(
                "Error: Expected identifier or 'this' and 'super' keywords before dot operator"
            );
        }

        ParserError::PrimaryExpr(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: Invalid expression. Found '{}'", s);
        }

        ParserError::ScopeError(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }
    }
}

pub fn handle_runtime_error(error: RuntimeError, code: &[&str]) {
    match error {
        RuntimeError::TypeMismatch(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }

        RuntimeError::TypeCastingError(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }

        RuntimeError::InvalidArgumentCount(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }

        RuntimeError::ArrayIndexOutOfBounds(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }

        RuntimeError::InvalidArrayIndex(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }

        RuntimeError::InvalidMemberAccess(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: Invalid use of '{}' for member expression", s);
        }
        RuntimeError::UndefinedField(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }
        RuntimeError::UndefinedProperty(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }

        RuntimeError::EnvironmentError(s, line) => {
            eprintln!("Line {}: {}", line, code[line - 1]);
            eprintln!("Error: {}", s);
        }

        RuntimeError::InternalError => {
            unreachable!(
                "Internal Error: This should not have happened. Please report this as a bug."
            );
        }
    }
}
