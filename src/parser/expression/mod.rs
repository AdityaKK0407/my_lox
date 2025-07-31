use crate::ast::*;
use crate::lexer::*;
use crate::parser::parser::*;
use crate::handle_errors::*;

pub fn parse_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    parse_assignment_expr(tokens)
}

fn parse_assignment_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    let left = parse_obj_expr(tokens)?;

    if at(tokens).token_type == TokenType::EQUAL {
        eat(tokens);
        let value = parse_assignment_expr(tokens)?;
        return Ok(Expr::AssignmentExpr {
            assignee: Box::new(left),
            value: Box::new(value),
        });
    }
    
    let (token, lexeme) = match at(tokens).token_type {
        TokenType::MINUSEQUAL => (TokenType::MINUSEQUAL, String::from("-")),
        TokenType::PLUSEQUAL => (TokenType::PLUSEQUAL, String::from("+")),
        TokenType::SLASHEQUAL => (TokenType::SLASHEQUAL, String::from("/")),
        TokenType::STAREQUAL => (TokenType::STAREQUAL, String::from("*")),
        _ => return Ok(left),
    };
    let line = eat(tokens).line;
    let value = parse_expr(tokens)?;

    Ok(Expr::AssignmentExpr {
            assignee: Box::new(left.clone()),
            value: Box::new(Expr::BinaryExpr { left: Box::new(left), operator: Token { token_type: token, lexeme, line: line }, right: Box::new(value) }),
    })
}

fn parse_obj_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    if at(tokens).token_type != TokenType::LEFTBRACE {
        return parse_logical_expr(tokens);
    }

    let _ = eat(tokens);
    let mut properties = vec![];

    while not_eof(tokens) && at(tokens).token_type != TokenType::RIGHTBRACE {
        if at(tokens).token_type != TokenType::IDENTIFIER && at(tokens).token_type != TokenType::STRING {
            return Err(ParserError::ObjectKey("Only string values and identifiers are allowed for object declaration"));
        }
        let key = eat(tokens);

        if at(tokens).token_type == TokenType::COMMA {
            eat(tokens);
            properties.push(Property {
                key: key.lexeme,
                value: None,
            });
            continue;
        } else if at(tokens).token_type == TokenType::RIGHTBRACE {
            properties.push(Property {
                key: key.lexeme,
                value: None,
            });
            continue;
        }
        let _ = expect(
            tokens,
            TokenType::COLON,
            "Expected colon for declaring value",
        )?;
        let value = parse_expr(tokens)?;

        properties.push(Property {
            key: key.lexeme,
            value: Some(Box::new(value)),
        });

        if at(tokens).token_type != TokenType::RIGHTBRACE {
            let _ = expect(tokens, TokenType::COMMA, "Expected comma or closing brace")?;
        }
    }

    let _ = expect(tokens, TokenType::RIGHTBRACE, "Expected closing brace")?;

    Ok(Expr::ObjectLiteral {
        properties: properties,
    })
}

fn parse_logical_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    let mut left = parse_equality_expr(tokens)?;

    while at(tokens).token_type == TokenType::AND || at(tokens).token_type == TokenType::OR {
        let operator = eat(tokens);
        let right = parse_equality_expr(tokens)?;
        left = Expr::ComparisonLiteral {
            left: Box::new(left),
            operator: operator,
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_equality_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    let mut left = parse_comparison_expr(tokens)?;

    while at(tokens).token_type == TokenType::EQUALEQUAL
        || at(tokens).token_type == TokenType::BANGEQUAL
    {
        let operator = eat(tokens);
        let right = parse_comparison_expr(tokens)?;
        left = Expr::ComparisonLiteral {
            left: Box::new(left),
            operator: operator,
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_comparison_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    let mut left = parse_additive_expr(tokens)?;

    while at(tokens).token_type == TokenType::GREATER
        || at(tokens).token_type == TokenType::GREATEREQUAL
        || at(tokens).token_type == TokenType::LESS
        || at(tokens).token_type == TokenType::LESSEQUAL
    {
        let operator = eat(tokens);
        let right = parse_additive_expr(tokens)?;
        left = Expr::ComparisonLiteral {
            left: Box::new(left),
            operator: operator,
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_additive_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    let mut left = parse_multiplicative_expr(tokens)?;

    while at(tokens).lexeme == "+" || at(tokens).lexeme == "-" {
        let operator = eat(tokens);
        let right = parse_multiplicative_expr(tokens)?;
        left = Expr::BinaryExpr {
            left: Box::new(left),
            operator: operator,
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_multiplicative_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    let mut left = parse_unary_expr(tokens)?;

    while at(tokens).lexeme == "*" || at(tokens).lexeme == "/" || at(tokens).lexeme == "%" {
        let operator = eat(tokens);
        let right = parse_unary_expr(tokens)?;
        left = Expr::BinaryExpr {
            left: Box::new(left),
            operator: operator,
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_unary_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    if at(tokens).token_type == TokenType::BANG || at(tokens).token_type == TokenType::MINUS {
        let operator = eat(tokens);
        let right = parse_expr(tokens)?;
        Ok(Expr::Unary {
            operator: operator,
            right: Box::new(right),
        })
    } else {
        parse_call_member_expr(tokens)
    }
}

fn parse_call_member_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    let member = parse_member_expr(tokens)?;

    if at(tokens).token_type == TokenType::LEFTPAREN {
        return parse_call_expr(tokens, member);
    }
    Ok(member)
}

fn parse_call_expr(tokens: &mut Vec<Token>, caller: Expr) -> Result<Expr, ParserError> {
    let mut call_expr = Expr::Call {
        args: parse_args(tokens)?,
        caller: Box::new(caller),
    };

    if at(tokens).token_type == TokenType::LEFTPAREN {
        call_expr = parse_call_expr(tokens, call_expr)?;
    }

    Ok(call_expr)
}

pub fn parse_args(tokens: &mut Vec<Token>) -> Result<Vec<Expr>, ParserError> {
    let _ = expect(tokens, TokenType::LEFTPAREN, "Expected open parenthesis")?;
    let args = if at(tokens).token_type == TokenType::RIGHTPAREN {
        vec![]
    } else {
        parse_arguments_list(tokens)?
    };
    let _ = expect(tokens, TokenType::RIGHTPAREN, "Expected close parenthesis")?;
    Ok(args)
}

fn parse_arguments_list(tokens: &mut Vec<Token>) -> Result<Vec<Expr>, ParserError> {
    let mut args = vec![parse_assignment_expr(tokens)?];

    while at(tokens).token_type == TokenType::COMMA {
        eat(tokens);
        args.push(parse_assignment_expr(tokens)?);
    }

    Ok(args)
}

fn parse_member_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    let mut object = parse_primary_expr(tokens)?;

    while at(tokens).token_type == TokenType::DOT || at(tokens).token_type == TokenType::LEFTBRACKET
    {
        let operator = eat(tokens);
        let property;
        let computed;

        if operator.token_type == TokenType::DOT {
            computed = false;
            property = parse_primary_expr(tokens)?;

            match property {
                Expr::Identifier(_) | Expr::This => {},
                _ => return Err(ParserError::MemberExpr("Not use . operator without identifier")),
            }
        } else {
            computed = true;
            property = parse_expr(tokens)?;
            expect(tokens, TokenType::RIGHTBRACKET, "Missing closing bracket")?;
        }
        object = Expr::Member {
            object: Box::new(object),
            property: Box::new(property),
            computed: computed,
        };
    }

    Ok(object)
}

fn parse_primary_expr(tokens: &mut Vec<Token>) -> Result<Expr, ParserError> {
    let tk = eat(tokens);

    match tk.token_type {
        TokenType::IDENTIFIER => Ok(Expr::Identifier(tk.lexeme)),
        TokenType::STRING => Ok(Expr::StringLiteral(tk.lexeme)),
        TokenType::NUMBER => Ok(Expr::NumericLiteral(tk.lexeme.parse::<f64>().unwrap())),
        TokenType::THIS => Ok(Expr::This),
        TokenType::TRUE => Ok(Expr::BoolLiteral(true)),
        TokenType::FALSE => Ok(Expr::BoolLiteral(false)),
        TokenType::NIL => Ok(Expr::Null),
        TokenType::LEFTPAREN => {
            let value = parse_expr(tokens)?;
            let _ = expect(
                tokens,
                TokenType::RIGHTPAREN,
                "Unexpected token. Closing paren expected.",
            )?;
            Ok(value)
        }
        _ => Err(ParserError::PrimaryExpr),
    }
}