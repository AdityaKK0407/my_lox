use crate::ast::*;
use crate::environment::Scope;
use crate::handle_errors::*;
use crate::lexer::*;
use crate::parser::parser::*;

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        let result = self.parse_assignment_expr()?;
        if self.scope.last().unwrap() == &Scope::Global && !self.is_repl {
            return Err(ParserError::ScopeError(
                "Invalid expression in global scope. Only declarations are allowed.".to_string(),
                self.at().line,
            ));
        }

        if let Scope::Class(name) = self.scope.last().unwrap() {
            return Err(ParserError::ScopeError(
                format!("Invalid expression in class '{}'.", name),
                self.at().line,
            ));
        }
        Ok(result)
    }

    fn parse_assignment_expr(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_obj_expr()?;

        if self.at().token_type == TokenType::EQUAL {
            let line = self.eat().line;
            let value = self.parse_assignment_expr()?;
            return Ok(Expr::AssignmentExpr {
                assignee: Box::new(left),
                value: Box::new(value),
                line,
            });
        }

        let (token, lexeme) = match self.at().token_type {
            TokenType::MINUSEQUAL => (TokenType::MINUSEQUAL, String::from("-")),
            TokenType::MODULUSEQUAL => (TokenType::MODULUSEQUAL, String::from("%")),
            TokenType::PLUSEQUAL => (TokenType::PLUSEQUAL, String::from("+")),
            TokenType::SLASHEQUAL => (TokenType::SLASHEQUAL, String::from("/")),
            TokenType::STAREQUAL => (TokenType::STAREQUAL, String::from("*")),
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
                    line,
                },
                right: Box::new(value),
                line,
            }),
            line,
        })
    }

    fn parse_obj_expr(&mut self) -> Result<Expr, ParserError> {
        if self.at().token_type != TokenType::LEFTBRACE {
            return self.parse_logical_expr();
        }

        let _ = self.eat();
        let mut properties = vec![];

        while self.not_eof() && self.at().token_type != TokenType::RIGHTBRACE {
            if self.at().token_type != TokenType::IDENTIFIER
                && self.at().token_type != TokenType::STRING
            {
                return Err(ParserError::ObjectKey(
                    format!("Found '{}'", self.at().lexeme),
                    self.at().line,
                ));
            }
            let key = self.eat();

            if self.at().token_type == TokenType::COMMA {
                let line = self.eat().line;
                properties.push(Property {
                    key: key.lexeme,
                    value: None,
                    line,
                });
                continue;
            } else if self.at().token_type == TokenType::RIGHTBRACE {
                properties.push(Property {
                    key: key.lexeme,
                    value: None,
                    line: self.at().line,
                });
                continue;
            }
            let _ = self.expect(
                TokenType::COLON,
                "Missing ':' for declaring value of object fields",
            )?;
            let value = self.parse_expr()?;

            properties.push(Property {
                key: key.lexeme,
                value: Some(Box::new(value)),
                line: self.at().line,
            });

            if self.at().token_type != TokenType::RIGHTBRACE {
                let _ = self.expect(TokenType::COMMA, "Missing ',' or '}' after object fields")?;
            }
        }

        let _ = self.expect(TokenType::RIGHTBRACE, "Missing closing '}' for object")?;

        Ok(Expr::ObjectLiteral {
            properties,
        })
    }

    fn parse_logical_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_equality_expr()?;

        while self.at().token_type == TokenType::AND || self.at().token_type == TokenType::OR {
            let operator = self.eat();
            let line = operator.line;
            let right = self.parse_equality_expr()?;
            left = Expr::ComparisonLiteral {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_equality_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_comparison_expr()?;

        while self.at().token_type == TokenType::EQUALEQUAL
            || self.at().token_type == TokenType::BANGEQUAL
        {
            let operator = self.eat();
            let line = operator.line;
            let right = self.parse_comparison_expr()?;
            left = Expr::ComparisonLiteral {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_additive_expr()?;

        while self.at().token_type == TokenType::GREATER
            || self.at().token_type == TokenType::GREATEREQUAL
            || self.at().token_type == TokenType::LESS
            || self.at().token_type == TokenType::LESSEQUAL
        {
            let operator = self.eat();
            let line = operator.line;
            let right = self.parse_additive_expr()?;
            left = Expr::ComparisonLiteral {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_multiplicative_expr()?;

        while self.at().lexeme == "+" || self.at().lexeme == "-" {
            let operator = self.eat();
            let line = operator.line;
            let right = self.parse_multiplicative_expr()?;
            left = Expr::BinaryExpr {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.parse_unary_expr()?;

        while self.at().lexeme == "*" || self.at().lexeme == "/" || self.at().lexeme == "%" {
            let operator = self.eat();
            let line = operator.line;
            let right = self.parse_unary_expr()?;
            left = Expr::BinaryExpr {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParserError> {
        if self.at().token_type == TokenType::BANG || self.at().token_type == TokenType::MINUS {
            let operator = self.eat();
            let line = operator.line;
            let right = self.parse_expr()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
                line,
            })
        } else {
            self.parse_call_member_expr()
        }
    }

    fn parse_call_member_expr(&mut self) -> Result<Expr, ParserError> {
        let member = self.parse_member_expr()?;

        if self.at().token_type == TokenType::LEFTPAREN {
            return self.parse_call_expr(member);
        }
        Ok(member)
    }

    fn parse_call_expr(&mut self, caller: Expr) -> Result<Expr, ParserError> {
        if self.scope.last().unwrap() == &Scope::Global && !self.is_repl {
            return Err(ParserError::UnExpectedToken("Unexpected function call expression in global scope. Did you forget to declare it using 'fun'?".to_string(), self.at().line));
        }

        if let Scope::Class(name) = self.scope.last().unwrap() {
            return Err(ParserError::UnExpectedToken(
                format!(
                    "Unexpected function call expression in class '{}'. Did you forget to declare it using 'fun'?",
                    name
                ),
                self.at().line,
            ));
        }
        let (args, line) = self.parse_args()?;
        let mut call_expr = Expr::Call {
            args,
            caller: Box::new(caller),
            line,
        };

        if self.at().token_type == TokenType::LEFTPAREN {
            call_expr = self.parse_call_expr(call_expr)?;
        }

        Ok(call_expr)
    }

    fn parse_args(&mut self) -> Result<(Vec<Expr>, usize), ParserError> {
        let line = self
            .expect(TokenType::LEFTPAREN, "Missing '(' for function call")?
            .line;
        let args = if self.at().token_type == TokenType::RIGHTPAREN {
            vec![]
        } else {
            self.parse_arguments_list()?
        };
        let _ = self.expect(TokenType::RIGHTPAREN, "Missing ')' for function call")?;
        Ok((args, line))
    }

    fn parse_arguments_list(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut args = vec![self.parse_assignment_expr()?];

        while self.at().token_type == TokenType::COMMA {
            let _ = self.eat();
            args.push(self.parse_assignment_expr()?);
        }

        Ok(args)
    }

    fn parse_member_expr(&mut self) -> Result<Expr, ParserError> {
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
                    Expr::Identifier(..) | Expr::This(_) | Expr::Super(_, _) => {}
                    _ => return Err(ParserError::MemberExpr(operator.line)),
                }
            } else {
                computed = true;
                property = self.parse_expr()?;
                let _ = self.expect(
                    TokenType::RIGHTBRACKET,
                    "Missing closing ']' in member expression",
                )?;
            }
            object = Expr::Member {
                object: Box::new(object),
                property: Box::new(property),
                computed,
                line: operator.line,
            };
        }

        Ok(object)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ParserError> {
        let tk = self.eat();
        let line = tk.line;

        match tk.token_type {
            TokenType::IDENTIFIER => Ok(Expr::Identifier(tk.lexeme, line)),
            TokenType::STRING => Ok(Expr::StringLiteral(tk.lexeme, line)),
            TokenType::NUMBER => Ok(Expr::NumericLiteral(
                tk.lexeme.parse::<f64>().unwrap(),
                line,
            )),
            TokenType::THIS => {
                let valid = self.scope.iter().rev().any(|scope| match scope {
                    Scope::Class(_) | Scope::Method(_) | Scope::Constructor(_) => true,
                    _ => false,
                });
                if !valid {
                    Err(ParserError::ScopeError(
                        "'this' keyword is only allowed inside class methods or constructors"
                            .to_string(),
                        line,
                    ))
                } else {
                    Ok(Expr::This(line))
                }
            }
            TokenType::SUPER => {
                if let Some(Scope::Class(class_name)) = self
                    .scope
                    .iter()
                    .rev()
                    .find(|class| matches!(class, Scope::Class(_)))
                {
                    Ok(Expr::Super(class_name.into(), line))
                } else {
                    Err(ParserError::ScopeError(
                        "'super' keyword is only allowed inside class methods or constructors"
                            .to_string(),
                        line,
                    ))
                }
            }
            TokenType::TRUE => Ok(Expr::BoolLiteral(true, line)),
            TokenType::FALSE => Ok(Expr::BoolLiteral(false, line)),
            TokenType::NIL => Ok(Expr::Null(line)),
            TokenType::LEFTPAREN => {
                let value = self.parse_expr()?;
                let _ = self.expect(
                    TokenType::RIGHTPAREN,
                    "Missing closing ')' for grouping expression",
                )?;
                Ok(value)
            },
            TokenType::LEFTBRACKET => {
                let mut value = vec![];

                while self.at().token_type != TokenType::RIGHTBRACKET {
                    value.push(self.parse_primary_expr()?);
                    if self.at().token_type == TokenType::RIGHTBRACKET {
                        break;
                    }
                    let _ = self.expect(TokenType::COMMA, "Missing ',' after array element")?;
                }

                let _ = self.eat();
                Ok(Expr::Array(value, line))
            }
            _ => Err(ParserError::PrimaryExpr(tk.lexeme, tk.line)),
        }
    }
}