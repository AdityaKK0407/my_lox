use crate::ast::*;
use crate::lexer::*;
use crate::parser::expression::*;
use crate::parser::statement::*;
use crate::handle_errors::*;

pub fn at(tokens: &Vec<Token>) -> &Token {
    &tokens[0]
}

pub fn eat(tokens: &mut Vec<Token>) -> Token {
    let token = tokens.remove(0);
    token
}

pub fn expect(tokens: &mut Vec<Token>, token: TokenType, message: &'static str) -> Result<Token, ParserError> {
    if !not_eof(tokens) {
        return Err(ParserError::EOF);
    }
    let tk = at(tokens);
    if tk.token_type != token {
        return Err(ParserError::UnExpectedToken(message))
    }
    Ok(eat(tokens))
}

pub fn not_eof(tokens: &mut Vec<Token>) -> bool {
    match tokens[0].token_type {
        TokenType::EOF => false,
        _ => true,
    }
}

pub fn produce_ast(mut tokens: Vec<Token>) -> Result<Vec<Stmt>, ParserError> {
    let mut program = vec![];

    while not_eof(&mut tokens) {
        program.push(parse_stmt(&mut tokens)?);
    }

    Ok(program)
}

pub fn parse_stmt(tokens: &mut Vec<Token>) -> Result<Stmt, ParserError> {
    match at(&tokens).token_type {
        TokenType::VAR | TokenType::CONST => parse_var_declaration(tokens),
        TokenType::IDENTIFIER
        | TokenType::NUMBER
        | TokenType::NIL
        | TokenType::TRUE
        | TokenType::FALSE
        | TokenType::MINUS
        | TokenType::STRING
        | TokenType::THIS
        | TokenType::LEFTPAREN => {
            let stmt = Stmt::Expression(parse_expr(tokens)?);
            let _ = expect(
                tokens,
                TokenType::SEMICOLON,
                "Expected semicolon at end of statement",
            )?;
            Ok(stmt)
        }
        TokenType::LEFTBRACE => parse_block_statement(tokens),
        TokenType::PRINT => parse_print_statement(tokens, false),
        TokenType::PRINTLN => parse_print_statement(tokens, true),
        TokenType::IF => parse_if_else_statement(tokens),
        TokenType::WHILE => parse_while_statement(tokens),
        TokenType::FOR => parse_for_statement(tokens),
        TokenType::FUN => parse_function_statement(tokens),
        TokenType::RETURN => {
            let _ = eat(tokens);
            let mut expr = Expr::Null;
            if at(tokens).token_type != TokenType::SEMICOLON {
                expr = parse_expr(tokens)?;
            }
            let _ = expect(
                tokens,
                TokenType::SEMICOLON,
                "Expected semicolon at end of statement",
            )?;
            Ok(Stmt::Return(expr))
        }
        TokenType::BREAK => {
            let _ = eat(tokens);
            let _ = expect(
                tokens,
                TokenType::SEMICOLON,
                "Expected semicolon at end of statement",
            )?;
            Ok(Stmt::Break)
        }
        TokenType::CONTINUE => {
            let _ = eat(tokens);
            let _ = expect(
                tokens,
                TokenType::SEMICOLON,
                "Expected semicolon at end of statement",
            )?;
            Ok(Stmt::Continue)
        }
        TokenType::CLASS => parse_class_statement(tokens),
        _ => {
            Err(ParserError::UnExpectedToken(""))
        }
    }
}
