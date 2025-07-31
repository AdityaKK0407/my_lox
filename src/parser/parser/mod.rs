use crate::ast::*;
use crate::handle_errors::*;
use crate::lexer::*;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {

    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        return Parser { tokens: tokens }
    }

    pub fn at(&self) -> &Token<'a> {
        &self.tokens[0]
    }

    pub fn eat(&mut self) -> Token<'a> {
        let token = self.tokens.remove(0);
        token
    }

    pub fn expect(&mut self, token: TokenType, message: &'a str) -> Result<Token<'a>, ParserError> {
        if !self.not_eof() {
            return Err(ParserError::EOF);
        }

        let matches = {
            let tk = self.at();
            tk.token_type == token
        };

        if !matches {
            return Err(ParserError::UnExpectedToken(message.to_string()));
        }
        Ok(self.eat())
    }
    pub fn not_eof(&self) -> bool {
        match self.tokens[0].token_type {
            TokenType::EOF => false,
            _ => true,
        }
    }
    pub fn produce_ast(&mut self) -> Result<Vec<Stmt<'a>>, ParserError> {
        let mut program = vec![];

        while self.not_eof() {
            program.push(self.parse_stmt()?);
        }

        Ok(program)
    }

    pub fn parse_stmt(&mut self) -> Result<Stmt<'a>, ParserError> {
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
                self.expect(
                    TokenType::SEMICOLON,
                    "Expected semicolon at end of statement",
                )?;
                Ok(stmt)
            }
            TokenType::LEFTBRACE => self.parse_block_statement(),
            TokenType::PRINT => self.parse_print_statement(false),
            TokenType::PRINTLN => self.parse_print_statement(true),
            TokenType::IF => self.parse_if_else_statement(),
            TokenType::WHILE => self.parse_while_statement(),
            TokenType::FOR => self.parse_for_statement(),
            TokenType::FUN => self.parse_function_statement(),
            TokenType::RETURN => {
                let _ = self.eat();
                let mut expr = Expr::Null;
                if self.at().token_type != TokenType::SEMICOLON {
                    expr = self.parse_expr()?;
                }
                let _ = self.expect(
                    TokenType::SEMICOLON,
                    "Expected semicolon at end of statement",
                )?;
                Ok(Stmt::Return(expr))
            }
            TokenType::BREAK => {
                self.eat();
                self.expect(
                    TokenType::SEMICOLON,
                    "Expected semicolon at end of statement",
                )?;
                Ok(Stmt::Break)
            }
            TokenType::CONTINUE => {
                self.eat();
                self.expect(
                    TokenType::SEMICOLON,
                    "Expected semicolon at end of statement",
                )?;
                Ok(Stmt::Continue)
            }
            TokenType::CLASS => self.parse_class_statement(),
            _ => Err(ParserError::UnExpectedToken(String::new())),
        }
    }
}
