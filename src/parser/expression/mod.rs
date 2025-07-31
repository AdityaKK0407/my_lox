use crate::ast::*;
use crate::handle_errors::*;
use crate::lexer::*;
use crate::parser::parser::*;

impl<'a> Parser<'a> {
    pub fn parse_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let left = self.parse_obj_expr()?;

        if self.at().token_type == TokenType::EQUAL {
            self.eat();
            let value = self.parse_assignment_expr()?;
            return Ok(Expr::AssignmentExpr {
                assignee: Box::new(left),
                value: Box::new(value),
            });
        }

        let (token, lexeme) = match self.at().token_type {
            TokenType::MINUSEQUAL => (TokenType::MINUSEQUAL, "-"),
            TokenType::MODULUSEQUAL => (TokenType::MODULUSEQUAL, "%"),
            TokenType::PLUSEQUAL => (TokenType::PLUSEQUAL, "+"),
            TokenType::SLASHEQUAL => (TokenType::SLASHEQUAL, "/"),
            TokenType::STAREQUAL => (TokenType::STAREQUAL, "*"),
            _ => return Ok(left),
        };
        let line = self.eat().line;
        let value = self.parse_expr()?;

        Ok(Expr::AssignmentExpr {
            assignee: Box::new(left.clone()),
            value: Box::new(Expr::BinaryExpr {
                left: Box::new(left),
                operator: Token {
                    token_type: token,
                    lexeme,
                    line: line,
                },
                right: Box::new(value),
            }),
        })
    }

    fn parse_obj_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        if self.at().token_type != TokenType::LEFTBRACE {
            return self.parse_logical_expr();
        }

        self.eat();
        let mut properties = vec![];

        while self.not_eof() && self.at().token_type != TokenType::RIGHTBRACE {
            if self.at().token_type != TokenType::IDENTIFIER
                && self.at().token_type != TokenType::STRING
            {
                return Err(ParserError::ObjectKey(String::from(
                    "Only string values and identifiers are allowed for object declaration",
                )));
            }
            let key = self.eat();

            if self.at().token_type == TokenType::COMMA {
                self.eat();
                properties.push(Property {
                    key: key.lexeme,
                    value: None,
                });
                continue;
            } else if self.at().token_type == TokenType::RIGHTBRACE {
                properties.push(Property {
                    key: key.lexeme,
                    value: None,
                });
                continue;
            }
            self.expect(TokenType::COLON, "Expected colon for declaring value")?;
            let value = self.parse_expr()?;

            properties.push(Property {
                key: key.lexeme,
                value: Some(Box::new(value)),
            });

            if self.at().token_type != TokenType::RIGHTBRACE {
                self.expect(TokenType::COMMA, "Expected comma or closing brace")?;
            }
        }

        self.expect(TokenType::RIGHTBRACE, "Expected closing brace")?;

        Ok(Expr::ObjectLiteral {
            properties: properties,
        })
    }

    fn parse_logical_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut left = self.parse_equality_expr()?;

        while self.at().token_type == TokenType::AND || self.at().token_type == TokenType::OR {
            let operator = self.eat();
            let right = self.parse_equality_expr()?;
            left = Expr::ComparisonLiteral {
                left: Box::new(left),
                operator: operator,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut left = self.parse_comparison_expr()?;

        while self.at().token_type == TokenType::EQUALEQUAL
            || self.at().token_type == TokenType::BANGEQUAL
        {
            let operator = self.eat();
            let right = self.parse_comparison_expr()?;
            left = Expr::ComparisonLiteral {
                left: Box::new(left),
                operator: operator,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut left = self.parse_additive_expr()?;

        while self.at().token_type == TokenType::GREATER
            || self.at().token_type == TokenType::GREATEREQUAL
            || self.at().token_type == TokenType::LESS
            || self.at().token_type == TokenType::LESSEQUAL
        {
            let operator = self.eat();
            let right = self.parse_additive_expr()?;
            left = Expr::ComparisonLiteral {
                left: Box::new(left),
                operator: operator,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut left = self.parse_multiplicative_expr()?;

        while self.at().lexeme == "+" || self.at().lexeme == "-" {
            let operator = self.eat();
            let right = self.parse_multiplicative_expr()?;
            left = Expr::BinaryExpr {
                left: Box::new(left),
                operator: operator,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut left = self.parse_unary_expr()?;

        while self.at().lexeme == "*" || self.at().lexeme == "/" || self.at().lexeme == "%" {
            let operator = self.eat();
            let right = self.parse_unary_expr()?;
            left = Expr::BinaryExpr {
                left: Box::new(left),
                operator: operator,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        if self.at().token_type == TokenType::BANG || self.at().token_type == TokenType::MINUS {
            let operator = self.eat();
            let right = self.parse_expr()?;
            Ok(Expr::Unary {
                operator: operator,
                right: Box::new(right),
            })
        } else {
            self.parse_call_member_expr()
        }
    }

    fn parse_call_member_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let member = self.parse_member_expr()?;

        if self.at().token_type == TokenType::LEFTPAREN {
            return self.parse_call_expr(member);
        }
        Ok(member)
    }

    fn parse_call_expr(&mut self, caller: Expr<'a>) -> Result<Expr<'a>, ParserError> {
        let mut call_expr = Expr::Call {
            args: self.parse_args()?,
            caller: Box::new(caller),
        };

        if self.at().token_type == TokenType::LEFTPAREN {
            call_expr = self.parse_call_expr(call_expr)?;
        }

        Ok(call_expr)
    }

    pub fn parse_args(&mut self) -> Result<Vec<Expr<'a>>, ParserError> {
        let _ = self.expect(TokenType::LEFTPAREN, "Expected open parenthesis")?;
        let args = if self.at().token_type == TokenType::RIGHTPAREN {
            vec![]
        } else {
            self.parse_arguments_list()?
        };
        let _ = self.expect(TokenType::RIGHTPAREN, "Expected close parenthesis")?;
        Ok(args)
    }

    fn parse_arguments_list(&mut self) -> Result<Vec<Expr<'a>>, ParserError> {
        let mut args = vec![self.parse_assignment_expr()?];

        while self.at().token_type == TokenType::COMMA {
            let _ = self.eat();
            args.push(self.parse_assignment_expr()?);
        }

        Ok(args)
    }

    fn parse_member_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut object = self.parse_primary_expr()?;

        while self.at().token_type == TokenType::DOT
            || self.at().token_type == TokenType::LEFTBRACKET
        {
            let operator = self.eat();
            let property;
            let computed;

            if operator.token_type == TokenType::DOT {
                computed = false;
                property = self.parse_primary_expr()?;

                match property {
                    Expr::Identifier(_) | Expr::This => {}
                    _ => {
                        return Err(ParserError::MemberExpr(String::from(
                            "Not use . operator without identifier",
                        )));
                    }
                }
            } else {
                computed = true;
                property = self.parse_expr()?;
                let _ = self.expect(TokenType::RIGHTBRACKET, "Missing closing bracket")?;
            }
            object = Expr::Member {
                object: Box::new(object),
                property: Box::new(property),
                computed: computed,
            };
        }

        Ok(object)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let tk = self.eat();

        match tk.token_type {
            TokenType::IDENTIFIER => Ok(Expr::Identifier(tk.lexeme)),
            TokenType::STRING => Ok(Expr::StringLiteral(tk.lexeme)),
            TokenType::NUMBER => Ok(Expr::NumericLiteral(tk.lexeme.parse::<f64>().unwrap())),
            TokenType::THIS => Ok(Expr::This),
            TokenType::TRUE => Ok(Expr::BoolLiteral(true)),
            TokenType::FALSE => Ok(Expr::BoolLiteral(false)),
            TokenType::NIL => Ok(Expr::Null),
            TokenType::LEFTPAREN => {
                let value = self.parse_expr()?;
                let _ = self.expect(
                    TokenType::RIGHTPAREN,
                    "Unexpected token. Closing paren expected.",
                )?;
                Ok(value)
            }
            _ => Err(ParserError::PrimaryExpr),
        }
    }
}
