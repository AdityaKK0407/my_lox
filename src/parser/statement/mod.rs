use std::collections::HashMap;

use crate::ast::*;
use crate::handle_errors::*;
use crate::lexer::*;
use crate::parser::expression::*;
use crate::parser::parser::*;

pub fn parse_var_declaration(tokens: &mut Vec<Token>) -> Result<Stmt, ParserError> {
    let is_constant = eat(tokens).token_type == TokenType::CONST;
    let identifier = expect(
        tokens,
        TokenType::IDENTIFIER,
        "Expected identifier name following var | const keyword",
    )?
    .lexeme;

    if at(tokens).token_type == TokenType::SEMICOLON {
        let _ = eat(tokens);
        if is_constant {
            return Err(ParserError::ConstValueNull);
        }

        return Ok(Stmt::VarDeclaration(VarDeclaration {
            constant: false,
            identifier: identifier,
            value: Box::new(Expr::Null),
        }));
    }
    expect(tokens, TokenType::EQUAL, "Expected equals in assignment")?;
    let declaration = Stmt::VarDeclaration(VarDeclaration { // Original code
        constant: is_constant,
        identifier: identifier,
        value: Box::new(parse_expr(tokens)?),
    });

    expect(
        tokens,
        TokenType::SEMICOLON,
        "Expected semicolon at end of statement",
    )?;
    Ok(declaration)
}

pub fn parse_print_statement(
    tokens: &mut Vec<Token>,
    new_line: bool,
) -> Result<Stmt, ParserError> {
    eat(tokens);
    if at(tokens).token_type == TokenType::SEMICOLON {
        eat(tokens);
        return Ok(Stmt::Print(None, new_line));
    }
    let expr = parse_expr(tokens)?;
    let mut expressions = vec![expr];
    
    while at(tokens).token_type == TokenType::COMMA {
        eat(tokens);
        expressions.push(parse_expr(tokens)?);
    }
    
    expect(
        tokens,
        TokenType::SEMICOLON,
        "Every statement must end in semicolon",
    )?;
    Ok(Stmt::Print(Some(expressions), new_line))
}

pub fn parse_if_else_statement(tokens: &mut Vec<Token>) -> Result<Stmt, ParserError> {
    eat(tokens);
    let expr = parse_expr(tokens)?;
    expect(
        tokens,
        TokenType::LEFTBRACE,
        "Expected open brace for if block",
    )?;
    let mut statements = vec![];
    while at(tokens).token_type != TokenType::RIGHTBRACE {
        statements.push(parse_stmt(tokens)?);
    }
    expect(
        tokens,
        TokenType::RIGHTBRACE,
        "Expected close brace for if block",
    )?;
    let mut if_collection = vec![(expr, statements)];

    loop {
        if at(tokens).token_type != TokenType::ELSE {
            break;
        }
        eat(tokens);

        let expr;
        if at(tokens).token_type == TokenType::IF {
            eat(tokens);
            expr = parse_expr(tokens)?;
        } else {
            expr = Expr::BoolLiteral(true);
        }

        expect(
            tokens,
            TokenType::LEFTBRACE,
            "Expected open brace for else if block",
        )?;
        let mut statements = vec![];
        while at(tokens).token_type != TokenType::RIGHTBRACE {
            match parse_stmt(tokens) {
                Ok(s) => statements.push(s),
                Err(e) => return Err(e),
            };
        }
        expect(
            tokens,
            TokenType::RIGHTBRACE,
            "Expected close brace for else if block",
        )?;
        if_collection.push((expr, statements));
    }
    Ok(Stmt::IfElse(if_collection))
}

pub fn parse_for_statement(tokens: &mut Vec<Token>) -> Result<Stmt, ParserError> {
    let _ = eat(tokens);

    if at(tokens).token_type == TokenType::SEMICOLON {
        return Err(ParserError::ForLoopDeclaration(""));
    }
    let var_stmt = parse_stmt(tokens)?;

    if at(tokens).token_type == TokenType::SEMICOLON {
        return Err(ParserError::ForLoopDeclaration(""));
    }
    let expr1 = parse_expr(tokens)?;
    let _ = eat(tokens);

    if at(tokens).token_type == TokenType::LEFTBRACE {
        return Err(ParserError::ForLoopDeclaration(""));
    }
    let expr2 = parse_expr(tokens)?;

    expect(
        tokens,
        TokenType::LEFTBRACE,
        "Expected open brace for while loop",
    )?;

    let mut stmt = vec![];
    while at(tokens).token_type != TokenType::RIGHTBRACE {
        stmt.push(parse_stmt(tokens)?);
    }

    expect(
        tokens,
        TokenType::RIGHTBRACE,
        "Expected close brace for while loop",
    )?;

    Ok(Stmt::For(((Box::new(var_stmt), expr1, expr2), stmt)))
}

pub fn parse_while_statement(tokens: &mut Vec<Token>) -> Result<Stmt, ParserError> {
    eat(tokens);
    let expr = parse_expr(tokens)?;
    expect(
        tokens,
        TokenType::LEFTBRACE,
        "Expected open brace for while loop",
    )?;

    let mut stmt = vec![];
    while at(tokens).token_type != TokenType::RIGHTBRACE {
        match parse_stmt(tokens) {
            Ok(s) => stmt.push(s),
            Err(e) => return Err(e),
        }
    }

    expect(
        tokens,
        TokenType::RIGHTBRACE,
        "Expected close brace for while loop",
    )?;

    Ok(Stmt::While((expr, stmt)))
}

pub fn parse_block_statement(tokens: &mut Vec<Token>) -> Result<Stmt, ParserError> {
    eat(tokens);
    let mut stmts = vec![];
    while at(tokens).token_type != TokenType::RIGHTBRACE {
        stmts.push(parse_stmt(tokens)?);
    }
    expect(
        tokens,
        TokenType::RIGHTBRACE,
        "Expected right brace at end of block",
    )?;
    Ok(Stmt::Block(stmts))
}

pub fn parse_function_statement(tokens: &mut Vec<Token>) -> Result<Stmt, ParserError> {
    eat(tokens);

    let name = expect(
        tokens,
        TokenType::IDENTIFIER,
        "Expected identifier as function name",
    )?
    .lexeme;
    expect(
        tokens,
        TokenType::LEFTPAREN,
        "Expected open paren for function",
    )?;

    let mut parameters = vec![];

    while at(tokens).token_type != TokenType::RIGHTPAREN {
        parameters.push(
            expect(
                tokens,
                TokenType::IDENTIFIER,
                "Expected identifier as parameter of function",
            )?
            .lexeme,
        );
        if at(tokens).token_type != TokenType::COMMA
            && at(tokens).token_type != TokenType::RIGHTPAREN
        {
            return Err(ParserError::UnExpectedToken(
                "Expected comma | closing paranthesis in function declaration",
            ));
        }
        if at(tokens).token_type == TokenType::COMMA {
            eat(tokens);
        }
    }

    expect(
        tokens,
        TokenType::RIGHTPAREN,
        "Expected closed paren for function",
    )?;

    let mut body = vec![];
    expect(
        tokens,
        TokenType::LEFTBRACE,
        "Expected open brace for function definition",
    )?;

    while at(tokens).token_type != TokenType::RIGHTBRACE {
        body.push(parse_stmt(tokens)?);
    }

    expect(
        tokens,
        TokenType::RIGHTBRACE,
        "Expected closed brace for function definition",
    )?;
    Ok(Stmt::Function(FunctionDeclaration {
        name: name,
        parameters: parameters,
        body: body,
    }))
}

pub fn parse_class_statement(tokens: &mut Vec<Token>) -> Result<Stmt, ParserError> {
    eat(tokens);

    let name = expect(tokens, TokenType::IDENTIFIER, "Class name missing")?.lexeme;

    let mut superclass = None;

    if at(tokens).token_type == TokenType::LESS {
        eat(tokens);
        superclass = Some(
            expect(
                tokens,
                TokenType::IDENTIFIER,
                "Expected class name for inheritance",
            )?
            .lexeme,
        );
    }

    let mut var = vec![];
    let mut methods = HashMap::new();

    expect(tokens, TokenType::LEFTBRACE, "Left brace for class")?;

    while at(tokens).token_type != TokenType::RIGHTBRACE {
        let stmt = parse_stmt(tokens)?;
        match stmt {
            Stmt::VarDeclaration(var_stmt) => var.push(var_stmt),
            Stmt::Function(method_stmt) => {
                methods.insert(method_stmt.name.clone(), method_stmt);
            }
            _ => return Err(ParserError::InvalidClassStmt),
        };
    }

    expect(tokens, TokenType::RIGHTBRACE, "Right brace for class")?;

    Ok(Stmt::Class(ClassDeclaration {
        name: name,
        static_fields: var,
        methods: methods,
        superclass: superclass,
    }))
}
