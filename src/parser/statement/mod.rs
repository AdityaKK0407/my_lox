use std::collections::HashMap;

use crate::ast::*;
use crate::handle_errors::*;
use crate::lexer::*;
use crate::parser::parser::*;

impl<'a> Parser<'a> {
    pub fn parse_var_declaration(&mut self) -> Result<Stmt<'a>, ParserError> {
        let is_constant = self.eat().token_type == TokenType::CONST;
        let identifier = self
            .expect(
                TokenType::IDENTIFIER,
                "Expected identifier name following var | const keyword",
            )?
            .lexeme;

        if self.at().token_type == TokenType::SEMICOLON {
            let _ = self.eat();
            if is_constant {
                return Err(ParserError::ConstValueNull);
            }

            return Ok(Stmt::VarDeclaration(VarDeclaration {
                constant: false,
                identifier: identifier,
                value: Box::new(Expr::Null),
            }));
        }
        let _ = self.expect(TokenType::EQUAL, "Expected equals in assignment")?;
        let declaration = Stmt::VarDeclaration(VarDeclaration {
            constant: is_constant,
            identifier: identifier,
            value: Box::new(self.parse_expr()?),
        });

        self.expect(
            TokenType::SEMICOLON,
            "Expected semicolon at end of statement",
        )?;
        Ok(declaration)
    }

    pub fn parse_print_statement(&mut self, new_line: bool) -> Result<Stmt<'a>, ParserError> {
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
            "Every statement must end in semicolon",
        )?;
        Ok(Stmt::Print(Some(expressions), new_line))
    }

    pub fn parse_if_else_statement(&mut self) -> Result<Stmt<'a>, ParserError> {
        let _ = self.eat();
        let expr = self.parse_expr()?;
        let _ = self.expect(TokenType::LEFTBRACE, "Expected open brace for if block")?;
        let mut statements = vec![];
        while self.at().token_type != TokenType::RIGHTBRACE {
            statements.push(self.parse_stmt()?);
        }
        let _ = self.expect(TokenType::RIGHTBRACE, "Expected close brace for if block")?;
        let mut if_collection = vec![(expr, statements)];

        loop {
            if self.at().token_type != TokenType::ELSE {
                break;
            }
            let _ = self.eat();

            let expr;
            if self.at().token_type == TokenType::IF {
                let _ = self.eat();
                expr = self.parse_expr()?;
            } else {
                expr = Expr::BoolLiteral(true);
            }

            let _ = self.expect(
                TokenType::LEFTBRACE,
                "Expected open brace for else if block",
            )?;
            let mut statements = vec![];
            while self.at().token_type != TokenType::RIGHTBRACE {
                match self.parse_stmt() {
                    Ok(s) => statements.push(s),
                    Err(e) => return Err(e),
                };
            }
            let _ = self.expect(
                TokenType::RIGHTBRACE,
                "Expected close brace for else if block",
            )?;
            if_collection.push((expr, statements));
        }
        Ok(Stmt::IfElse(if_collection))
    }

    pub fn parse_for_statement(&mut self) -> Result<Stmt<'a>, ParserError> {
        let _ = self.eat();

        if self.at().token_type == TokenType::SEMICOLON {
            return Err(ParserError::ForLoopDeclaration(1));
        }
        let var_stmt = self.parse_stmt()?;

        if self.at().token_type == TokenType::SEMICOLON {
            return Err(ParserError::ForLoopDeclaration(2));
        }
        let expr1 = self.parse_expr()?;
        let _ = self.eat();

        if self.at().token_type == TokenType::LEFTBRACE {
            return Err(ParserError::ForLoopDeclaration(3));
        }
        let expr2 = self.parse_expr()?;

        let _ = self.expect(TokenType::LEFTBRACE, "Expected open brace for while loop")?;

        let mut stmt = vec![];
        while self.at().token_type != TokenType::RIGHTBRACE {
            stmt.push(self.parse_stmt()?);
        }

        let _ = self.expect(TokenType::RIGHTBRACE, "Expected close brace for while loop")?;

        Ok(Stmt::For(((Box::new(var_stmt), expr1, expr2), stmt)))
    }

    pub fn parse_while_statement(&mut self) -> Result<Stmt<'a>, ParserError> {
        let _ = self.eat();
        let expr = self.parse_expr()?;
        let _ = self.expect(TokenType::LEFTBRACE, "Expected open brace for while loop")?;

        let mut stmt = vec![];
        while self.at().token_type != TokenType::RIGHTBRACE {
            match self.parse_stmt() {
                Ok(s) => stmt.push(s),
                Err(e) => return Err(e),
            }
        }

        let _ = self.expect(TokenType::RIGHTBRACE, "Expected close brace for while loop")?;

        Ok(Stmt::While((expr, stmt)))
    }

    pub fn parse_block_statement(&mut self) -> Result<Stmt<'a>, ParserError> {
        let _ = self.eat();
        let mut stmts = vec![];
        while self.at().token_type != TokenType::RIGHTBRACE {
            stmts.push(self.parse_stmt()?);
        }
        let _ = self.expect(
            TokenType::RIGHTBRACE,
            "Expected right brace at end of block",
        )?;
        Ok(Stmt::Block(stmts))
    }

    pub fn parse_function_statement(&mut self) -> Result<Stmt<'a>, ParserError> {
        let _ = self.eat();

        let name = self
            .expect(
                TokenType::IDENTIFIER,
                "Expected identifier as function name",
            )?
            .lexeme;
        let _ = self.expect(TokenType::LEFTPAREN, "Expected open paren for function")?;

        let mut parameters = vec![];

        while self.at().token_type != TokenType::RIGHTPAREN {
            parameters.push(
                self.expect(
                    TokenType::IDENTIFIER,
                    "Expected identifier as parameter of function",
                )?
                .lexeme,
            );
            if self.at().token_type != TokenType::COMMA
                && self.at().token_type != TokenType::RIGHTPAREN
            {
                return Err(ParserError::UnExpectedToken(
                    "Expected comma | closing paranthesis in function declaration".to_string(),
                ));
            }
            if self.at().token_type == TokenType::COMMA {
                self.eat();
            }
        }

        let _ = self.expect(TokenType::RIGHTPAREN, "Expected closed paren for function")?;

        let mut body = vec![];
        let _ = self.expect(
            TokenType::LEFTBRACE,
            "Expected open brace for function definition",
        )?;

        while self.at().token_type != TokenType::RIGHTBRACE {
            body.push(self.parse_stmt()?);
        }

        let _ = self.expect(
            TokenType::RIGHTBRACE,
            "Expected closed brace for function definition",
        )?;
        Ok(Stmt::Function(FunctionDeclaration {
            name: name,
            parameters: parameters,
            body: body,
        }))
    }

    pub fn parse_class_statement(&mut self) -> Result<Stmt<'a>, ParserError> {
        let _ = self.eat();

        let name = self
            .expect(TokenType::IDENTIFIER, "Class name missing")?
            .lexeme;

        let mut superclass = None;

        if self.at().token_type == TokenType::LESS {
            let _ = self.eat();
            superclass = Some(
                self.expect(TokenType::IDENTIFIER, "Expected class name for inheritance")?
                    .lexeme,
            );
        }

        let mut var = vec![];
        let mut methods = HashMap::new();

        let _ = self.expect(TokenType::LEFTBRACE, "Left brace for class")?;

        while self.at().token_type != TokenType::RIGHTBRACE {
            let stmt = self.parse_stmt()?;
            match stmt {
                Stmt::VarDeclaration(var_stmt) => var.push(var_stmt),
                Stmt::Function(method_stmt) => {
                    methods.insert(method_stmt.name, method_stmt);
                }
                _ => return Err(ParserError::InvalidClassStmt),
            };
        }

        let _ = self.expect(TokenType::RIGHTBRACE, "Right brace for class")?;

        Ok(Stmt::Class(ClassDeclaration {
            name: name,
            static_fields: var,
            methods: methods,
            superclass: superclass,
        }))
    }
}
