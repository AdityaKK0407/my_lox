use crate::ast::*;
use crate::environment::Scope;
use crate::handle_errors::*;
use crate::lexer::*;

pub struct Parser {
    tokens: Vec<Token>,
    pub scope: Vec<Scope>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            scope: vec![Scope::Global],
        }
    }

    pub fn at(&self) -> &Token {
        &self.tokens[0]
    }

    pub fn eat(&mut self) -> Token {
        let token = self.tokens.remove(0);
        token
    }

    pub fn expect(&mut self, token: TokenType, message: &str) -> Result<Token, ParserError> {
        if !self.not_eof() {
            return Err(ParserError::EOF);
        }
        let tk = self.at();
        if tk.token_type != token {
            return Err(ParserError::UnExpectedToken(message.to_string(), tk.line));
        }
        Ok(self.eat())
    }

    pub fn not_eof(&self) -> bool {
        match self.tokens[0].token_type {
            TokenType::EOF => false,
            _ => true,
        }
    }

    pub fn produce_ast(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut program = vec![];

        while self.not_eof() {
            program.push(self.parse_stmt()?);
        }

        Ok(program)
    }

    pub fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        match self.at().token_type {
            TokenType::VAR | TokenType::CONST => self.parse_var_declaration(),
            TokenType::IDENTIFIER
            | TokenType::NUMBER
            | TokenType::NIL
            | TokenType::TRUE
            | TokenType::FALSE
            | TokenType::MINUS
            | TokenType::STRING
            | TokenType::THIS
            | TokenType::LEFTPAREN => {
                let stmt = Stmt::Expression(self.parse_expr()?);
                let _ =
                    self.expect(TokenType::SEMICOLON, "Missing ';' at the end of expression")?;
                Ok(stmt)
            }
            TokenType::LEFTBRACE => self.parse_block_statement(),
            TokenType::PRINT => self.parse_print_statement(false),
            TokenType::PRINTLN => self.parse_print_statement(true),
            TokenType::IF => self.parse_if_else_statement(),
            TokenType::WHILE => self.parse_while_statement(),
            TokenType::FOR => self.parse_for_statement(),
            TokenType::FUN => self.parse_function_statement(),
            TokenType::CLASS => self.parse_class_statement(),
            TokenType::RETURN => {
                let line = self.eat().line;
                match self.scope.last().unwrap() {
                    Scope::Global => {
                        return Err(ParserError::ScopeError("Return statement not allowed in global scope. Must be inside a function or method.".to_string(), line));
                    }
                    Scope::Class(class_name) => {
                        return Err(ParserError::ScopeError(
                            format!(
                                "Invalid return statement in class '{}'. Returns must be inside a method.",
                                class_name
                            ),
                            line,
                        ));
                    }
                    Scope::Constructor(name) => {
                        return Err(ParserError::ScopeError(
                            format!(
                                "Invalid return statement in constructor of class '{}'. Use return only inside functions or methods.",
                                name
                            ),
                            line,
                        ));
                    }
                    Scope::Loop => {
                        let valid = self.scope.iter().rev().any(|scope| match scope {
                            Scope::Function(_) | Scope::Method(_) => true,
                            _ => false,
                        });
                        if !valid {
                            return Err(ParserError::ScopeError("Invalid return statement inside loop. Must be within a function or method.".to_string(), line));
                        }
                    }
                    _ => {}
                }
                let mut expr = Expr::Null(line);
                if self.at().token_type != TokenType::SEMICOLON {
                    expr = self.parse_expr()?;
                }
                let _ = self.expect(
                    TokenType::SEMICOLON,
                    "Missing ';' at end of return statement",
                )?;
                Ok(Stmt::Return(expr))
            }
            TokenType::BREAK => {
                let line = self.eat().line;
                match self.scope.last().unwrap() {
                    Scope::Global => {return Err(ParserError::ScopeError("Invalid use of 'break' at global scope. 'break' is only allowed inside loops.".to_string(), line))},
                    Scope::Class(class_name) => {return Err(ParserError::ScopeError(format!("Invalid use of 'break' in class '{}'. 'break' is only allowed inside loops", class_name), line))},
                    Scope::Method(name) => {return Err(ParserError::ScopeError(format!("Invalid use of 'break' in method '{}'. 'break' is only allowed inside loops", name), line))},
                    Scope::Constructor(name) => {return Err(ParserError::ScopeError(format!("Invalid use of 'break' in constructor of class '{}'. 'break' is only allowed inside loops", name), line))},
                    Scope::Function(name) => {return Err(ParserError::ScopeError(format!("Invalid use of 'break' in function '{}'. 'break' is only allowed inside loops", name), line))},
                    _ => {},
                }
                let _ = self.expect(
                    TokenType::SEMICOLON,
                    "Missing ';' at end of break statement",
                )?;
                Ok(Stmt::Break)
            }
            TokenType::CONTINUE => {
                let line = self.eat().line;
                match self.scope.last().unwrap() {
                    Scope::Global => {return Err(ParserError::ScopeError("Invalid use of 'continue' at global scope. 'continue' is only allowed inside loops.".to_string(), line))},
                    Scope::Class(class_name) => {return Err(ParserError::ScopeError(format!("Invalid use of 'continue' in class '{}'. 'continue' is only allowed inside loops", class_name), line))},
                    Scope::Method(name) => {return Err(ParserError::ScopeError(format!("Invalid use of 'continue' in method '{}'. 'continue' is only allowed inside loops", name), line))},
                    Scope::Constructor(name) => {return Err(ParserError::ScopeError(format!("Invalid use of 'continue' in constructor of class '{}'. 'continue' is only allowed inside loops", name), line))},
                    Scope::Function(name) => {return Err(ParserError::ScopeError(format!("Invalid use of 'continue' in function '{}'. 'continue' is only allowed inside loops", name), line))},
                    _ => {},
                }
                let _ = self.expect(
                    TokenType::SEMICOLON,
                    "Missing ';' at end of continue statement",
                )?;
                Ok(Stmt::Continue)
            }
            _ => Err(ParserError::UnExpectedToken(
                format!("Invalid statement. Found {}", self.at().lexeme),
                self.at().line,
            )),
        }
    }
}
