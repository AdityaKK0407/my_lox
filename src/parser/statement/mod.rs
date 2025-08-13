use std::collections::HashMap;

use crate::ast::*;
use crate::environment::Scope;
use crate::handle_errors::*;
use crate::lexer::*;
use crate::parser::parser::*;

impl Parser {
    pub fn parse_var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let line = self.at().line;
        let is_constant = self.eat().token_type == TokenType::CONST;
        let identifier = self
            .expect(
                TokenType::IDENTIFIER,
                "Expected identifier name following 'var' and 'const' keyword",
            )?
            .lexeme;

        if self.at().token_type == TokenType::SEMICOLON {
            let _ = self.eat().line;
            if is_constant {
                return Err(ParserError::ConstValueNull(self.at().line));
            }

            return Ok(Stmt::VarDeclaration(VarDeclaration {
                constant: false,
                identifier,
                value: Box::new(Expr::Null(self.at().line)),
                line,
            }));
        }
        let _ = self.expect(
            TokenType::EQUAL,
            "Expected '=' for initialization of variable",
        )?;
        self.scope.push(Scope::VarDeclaration);
        let declaration = Stmt::VarDeclaration(VarDeclaration {
            constant: is_constant,
            identifier,
            value: Box::new(self.parse_expr()?),
            line,
        });
        self.scope.pop();

        let _ = self.expect(
            TokenType::SEMICOLON,
            "Expected ';' at the end of variable declaration",
        )?;
        Ok(declaration)
    }

    pub fn parse_print_statement(&mut self, new_line: bool) -> Result<Stmt, ParserError> {
        if self.scope.last().unwrap() == &Scope::Global && !self.is_repl {
            return Err(ParserError::ScopeError(
                "Print statement not allowed in global scope".to_string(),
                self.at().line,
            ));
        }
        if let Scope::Class(class_name) = self.scope.last().unwrap() {
            return Err(ParserError::ScopeError(
                format!(
                    "Invalid print statement inside class '{}'. Only method and field declarations are allowed.",
                    class_name
                ),
                self.at().line,
            ));
        }
        let _ = self.eat();
        if self.at().token_type == TokenType::SEMICOLON {
            let _ = self.eat();
            return Ok(Stmt::Print(None, new_line));
        }
        let expr = self.parse_expr()?;
        let mut expressions = vec![expr];

        while self.at().token_type == TokenType::COMMA {
            let _ = self.eat();
            expressions.push(self.parse_expr()?);
        }

        let _ = self.expect(
            TokenType::SEMICOLON,
            "Expected ';' at end of print statement",
        )?;
        Ok(Stmt::Print(Some(expressions), new_line))
    }

    pub fn parse_if_else_statement(&mut self) -> Result<Stmt, ParserError> {
        let line = self.at().line;
        if self.scope.last().unwrap() == &Scope::Global && !self.is_repl {
            return Err(ParserError::ScopeError(
                "if-else statements not allowed in global scope".to_string(),
                self.at().line,
            ));
        }
        if let Scope::Class(class_name) = self.scope.last().unwrap() {
            return Err(ParserError::ScopeError(
                format!(
                    "Invalid if-else statements inside class '{}'. Only method and field declarations are allowed.",
                    class_name
                ),
                self.at().line,
            ));
        }
        let _ = self.eat();
        let expr = self.parse_expr()?;
        let _ = self.expect(
            TokenType::LEFTBRACE,
            "Missing '{' to start the body of the if block",
        )?;
        let mut statements = vec![];
        while self.at().token_type != TokenType::RIGHTBRACE {
            statements.push(self.parse_stmt()?);
        }
        let _ = self.expect(
            TokenType::RIGHTBRACE,
            "Missing '}' to end the body of the if block",
        )?;
        let mut if_collection = vec![(expr, statements, line)];

        let mut is_else_block;
        let messages1 = [
            "Missing '{' to start the body of the else if block",
            "Missing '{' to start the body of the else block",
        ];
        let messages2 = [
            "Missing '}' to end the body of the else if block",
            "Missing '}' to end the body of the else block",
        ];
        loop {
            if self.at().token_type != TokenType::ELSE {
                break;
            }
            let line = self.eat().line;

            let expr;
            if self.at().token_type == TokenType::IF {
                let _ = self.eat();
                expr = self.parse_expr()?;
                is_else_block = 0;
            } else {
                expr = Expr::BoolLiteral(true, line);
                is_else_block = 1;
            }

            let _ = self.expect(TokenType::LEFTBRACE, messages1[is_else_block])?;
            let mut statements = vec![];
            while self.at().token_type != TokenType::RIGHTBRACE {
                match self.parse_stmt() {
                    Ok(s) => statements.push(s),
                    Err(e) => return Err(e),
                };
            }
            let _ = self.expect(TokenType::RIGHTBRACE, messages2[is_else_block])?;
            if_collection.push((expr, statements, line));
        }
        Ok(Stmt::IfElse(if_collection))
    }

    pub fn parse_for_statement(&mut self) -> Result<Stmt, ParserError> {
        if self.scope.last().unwrap() == &Scope::Global && !self.is_repl {
            return Err(ParserError::ScopeError(
                "for loop not allowed in global scope".to_string(),
                self.at().line,
            ));
        }
        if let Scope::Class(class_name) = self.scope.last().unwrap() {
            return Err(ParserError::ScopeError(
                format!(
                    "Invalid for loop inside class '{}'. Only method and field declarations are allowed.",
                    class_name
                ),
                self.at().line,
            ));
        }
        self.scope.push(Scope::Loop);
        let line = self.eat().line;

        if self.at().token_type == TokenType::SEMICOLON {
            return Err(ParserError::ForLoopDeclaration(
                "".to_string(),
                self.at().line,
            ));
        }
        let var_stmt = self.parse_stmt()?;

        if self.at().token_type == TokenType::SEMICOLON {
            return Err(ParserError::ForLoopDeclaration(
                "".to_string(),
                self.at().line,
            ));
        }
        let expr1 = self.parse_expr()?;
        let _ = self.eat();

        if self.at().token_type == TokenType::LEFTBRACE {
            return Err(ParserError::ForLoopDeclaration(
                "".to_string(),
                self.at().line,
            ));
        }
        let expr2 = self.parse_expr()?;

        let _ = self.expect(
            TokenType::LEFTBRACE,
            "Missing '{' to start the body of the for loop",
        )?;

        let mut stmt = vec![];
        while self.at().token_type != TokenType::RIGHTBRACE {
            stmt.push(self.parse_stmt()?);
        }

        let _ = self.expect(
            TokenType::RIGHTBRACE,
            "Missing '}' to end the body of the for loop",
        )?;

        self.scope.pop();
        Ok(Stmt::For((Box::new(var_stmt), expr1, expr2), stmt, line))
    }

    pub fn parse_while_statement(&mut self) -> Result<Stmt, ParserError> {
        if self.scope.last().unwrap() == &Scope::Global && !self.is_repl {
            return Err(ParserError::ScopeError(
                "while loop not allowed in global scope".to_string(),
                self.at().line,
            ));
        }

        if let Scope::Class(class_name) = self.scope.last().unwrap() {
            return Err(ParserError::ScopeError(
                format!(
                    "Invalid while loop inside class '{}'. Only method and field declarations are allowed.",
                    class_name
                ),
                self.at().line,
            ));
        }
        self.scope.push(Scope::Loop);
        let line = self.eat().line;
        let expr = self.parse_expr()?;
        let _ = self.expect(
            TokenType::LEFTBRACE,
            "Missing '{' to start the body of the while loop",
        )?;

        let mut stmt = vec![];
        while self.at().token_type != TokenType::RIGHTBRACE {
            match self.parse_stmt() {
                Ok(s) => stmt.push(s),
                Err(e) => return Err(e),
            }
        }

        let _ = self.expect(
            TokenType::RIGHTBRACE,
            "Missing '}' to start the body of the while loop",
        )?;
        self.scope.pop();
        Ok(Stmt::While(expr, stmt, line))
    }

    pub fn parse_block_statement(&mut self) -> Result<Stmt, ParserError> {
        if self.scope.last().unwrap() == &Scope::Global && !self.is_repl {
            return Err(ParserError::ScopeError(
                "block statements not allowed in global scope".to_string(),
                self.at().line,
            ));
        }

        if let Scope::Class(class_name) = self.scope.last().unwrap() {
            return Err(ParserError::ScopeError(
                format!(
                    "Invalid block statement inside class '{}'. Only method and field declarations are allowed.",
                    class_name
                ),
                self.at().line,
            ));
        }
        let _ = self.eat();
        let mut stmts = vec![];
        while self.at().token_type != TokenType::RIGHTBRACE {
            stmts.push(self.parse_stmt()?);
        }
        let _ = self.expect(
            TokenType::RIGHTBRACE,
            "Missing '}' to end the body of the block statement",
        )?;
        Ok(Stmt::Block(stmts))
    }

    pub fn parse_functional_statement(&mut self) -> Result<Stmt, ParserError> {
        let line = self.eat().line;

        let name = self
            .expect(
                TokenType::IDENTIFIER,
                "Expected function name after 'fun' keyword",
            )?
            .lexeme;

        if let Scope::Class(class_name) = self.scope.last().unwrap() {
            if class_name == &name {
                self.scope.push(Scope::Constructor(name.clone()));
            } else {
                self.scope.push(Scope::Method(name.clone()));
            }
        } else {
            self.scope.push(Scope::Function(name.clone()));
        }

        let _ = self.expect(
            TokenType::LEFTPAREN,
            format!("Missing '(' to declare parameters of function {}", name).as_str(),
        )?;

        let mut parameters = vec![];

        while self.at().token_type != TokenType::RIGHTPAREN {
            parameters.push(
                self.expect(
                    TokenType::IDENTIFIER,
                    format!("Expected parameter name in function '{}'", name).as_str(),
                )?
                .lexeme,
            );
            if self.at().token_type != TokenType::COMMA
                && self.at().token_type != TokenType::RIGHTPAREN
            {
                return Err(ParserError::UnExpectedToken(
                    format!("Expected ',' or ')' in {} function declaration", name),
                    self.at().line,
                ));
            }
            if self.at().token_type == TokenType::COMMA {
                let _ = self.eat();
            }
        }

        let _ = self.expect(
            TokenType::RIGHTPAREN,
            format!("Missing ')' for parameter declaration in function {}", name).as_str(),
        )?;

        let mut body = vec![];
        let _ = self.expect(
            TokenType::LEFTBRACE,
            format!("Missing '{{' to start the body of function {}", name).as_str(),
        )?;

        while self.at().token_type != TokenType::RIGHTBRACE {
            body.push(self.parse_stmt()?);
        }

        let _ = self.expect(
            TokenType::RIGHTBRACE,
            format!("Missing '}}' to end the body of function {}", name).as_str(),
        )?;
        self.scope.pop();

        Ok(Stmt::Function(FunctionDeclaration {
            name,
            parameters,
            body,
            line,
        }))
    }

    pub fn parse_class_statement(&mut self) -> Result<Stmt, ParserError> {
        if self.scope.last().unwrap() != &Scope::Global {
            return Err(ParserError::ScopeError(
                "Class declarations are only allowed in the global scope".to_string(),
                self.at().line,
            ));
        }
        let line = self.eat().line;

        let name = self
            .expect(
                TokenType::IDENTIFIER,
                "Expected class name after 'class' keyword",
            )?
            .lexeme;
        self.scope.push(Scope::Class(name.clone()));

        let mut superclass = None;

        if self.at().token_type == TokenType::LESS {
            let _ = self.eat();
            superclass = Some(
                self.expect(TokenType::IDENTIFIER, "Expected superclass name after '<'")?
                    .lexeme,
            );
        }

        let mut var = vec![];
        let mut methods = HashMap::new();

        let _ = self.expect(
            TokenType::LEFTBRACE,
            format!("Missing '{{' to start the body of class {}", name).as_str(),
        )?;

        while self.at().token_type != TokenType::RIGHTBRACE {
            let stmt = match self.parse_stmt() {
                Ok(s) => s,
                Err(e) => return match e {
                    ParserError::ScopeError(message, line) => {
                        Err(ParserError::ScopeError(
                            format!(
                                "Invalid {} inside class body. Only method and field declarations are allowed.",
                                message
                            ),
                            line,
                        ))
                    }
                    _ => Err(e),
                },
            };
            match stmt {
                Stmt::VarDeclaration(var_stmt) => var.push(var_stmt),
                Stmt::Function(method_stmt) => {
                    methods.insert(method_stmt.name.clone(), method_stmt);
                }
                _ => {}
            };
        }

        let _ = self.expect(
            TokenType::RIGHTBRACE,
            format!("Missing '}}' to end the body of class {}", name).as_str(),
        )?;

        self.scope.pop();
        Ok(Stmt::Class(ClassDeclaration {
            name,
            static_fields: var,
            methods,
            superclass,
            line,
        }))
    }
}
