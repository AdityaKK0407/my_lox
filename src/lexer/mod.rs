use crate::handle_errors::handle_lexer_error;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-Character Tokens
    LEFTPAREN,
    RIGHTPAREN,
    LEFTBRACE,
    RIGHTBRACE,
    LEFTBRACKET,
    RIGHTBRACKET,
    COLON,
    COMMA,
    DOT,
    MINUS,
    MODULUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or Two Character Tokens
    BANG,
    BANGEQUAL,
    EQUAL,
    EQUALEQUAL,
    GREATER,
    GREATEREQUAL,
    LESS,
    LESSEQUAL,
    MINUSEQUAL,
    MODULUSEQUAL,
    PLUSEQUAL,
    SLASHEQUAL,
    STAREQUAL,

    // Literals
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords
    AND,
    BREAK,
    CLASS,
    CONST,
    CONTINUE,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    PRINTLN,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    // End of File
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}

pub struct Tokenizer {
    tokens: Vec<Token>,
    source_code: String,
    start: usize,
    current: usize,
    line: usize,
    had_error: bool,
}

impl Tokenizer {
    pub fn new(source_code: String) -> Tokenizer {
        Tokenizer {
            tokens: vec![],
            source_code,
            start: 0,
            current: 0,
            line: 1,
            had_error: false,
        }
    }

    pub fn scan_tokens(mut self) -> (Vec<Token>, bool) {
        while !&self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::EOF, String::new(), self.line));
        (self.tokens, self.had_error)
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LEFTPAREN),
            ')' => self.add_token(TokenType::RIGHTPAREN),
            '{' => self.add_token(TokenType::LEFTBRACE),
            '}' => self.add_token(TokenType::RIGHTBRACE),
            '[' => self.add_token(TokenType::LEFTBRACKET),
            ']' => self.add_token(TokenType::RIGHTBRACKET),
            ':' => self.add_token(TokenType::COLON),
            ',' => self.add_token(TokenType::COMMA),
            '.' => self.add_token(TokenType::DOT),
            '-' => {
                let matched = self.match_char('=');
                self.add_token(if matched {
                    TokenType::MINUSEQUAL
                } else {
                    TokenType::MINUS
                });
            }
            '+' => {
                let matched = self.match_char('=');
                self.add_token(if matched {
                    TokenType::PLUSEQUAL
                } else {
                    TokenType::PLUS
                });
            }
            ';' => self.add_token(TokenType::SEMICOLON),
            '*' => {
                let matched = self.match_char('=');
                self.add_token(if matched {
                    TokenType::STAREQUAL
                } else {
                    TokenType::STAR
                });
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('=') {
                    self.add_token(TokenType::SLASHEQUAL);
                } else {
                    self.add_token(TokenType::SLASH);
                }
            }
            '%' => {
                let matched = self.match_char('=');
                self.add_token(if matched {
                    TokenType::MODULUSEQUAL
                } else {
                    TokenType::MODULUS
                });
            }
            '!' => {
                let matched = self.match_char('=');
                self.add_token(if matched {
                    TokenType::BANGEQUAL
                } else {
                    TokenType::BANG
                });
            }
            '=' => {
                let matched = self.match_char('=');
                self.add_token(if matched {
                    TokenType::EQUALEQUAL
                } else {
                    TokenType::EQUAL
                });
            }
            '<' => {
                let matched = self.match_char('=');
                self.add_token(if matched {
                    TokenType::LESSEQUAL
                } else {
                    TokenType::LESS
                });
            }
            '>' => {
                let matched = self.match_char('=');
                self.add_token(if matched {
                    TokenType::GREATEREQUAL
                } else {
                    TokenType::GREATER
                });
            }

            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' | '\'' => self.string(c),

            _ => {
                if is_digit(c) {
                    self.number();
                } else if is_alpha(c) {
                    self.identifier();
                } else {
                    handle_lexer_error(self.line, &format!("Unexpected character {c}."));
                    self.had_error = true;
                }
            }
        };
    }

    fn identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = &self.source_code[self.start..self.current];
        self.add_token(match_keyword(text));
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.add_token(TokenType::NUMBER);
    }

    fn string(&mut self, c: char) {
        while self.peek() != c && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            handle_lexer_error(self.line, "Unterminated string.");
            self.had_error = true;
            return;
        }
        self.advance();
        self.add_token(TokenType::STRING);
    }

    fn get_current_char(&self, buf: usize) -> char {
        self.source_code.as_bytes()[self.current + buf] as char
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.get_current_char(0) != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.get_current_char(0)
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source_code.len() {
            return '\0';
        }
        self.get_current_char(1)
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source_code.len();
    }

    fn advance(&mut self) -> char {
        let c = self.get_current_char(0);
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        let mut buf = 0;
        if token_type == TokenType::STRING {
            buf = 1;
        }
        let text = &self.source_code[self.start + buf..self.current - buf];
        self.tokens
            .push(Token::new(token_type, text.to_string(), self.line));
    }
}

fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn match_keyword(s: &str) -> TokenType {
    match s {
        "and" => TokenType::AND,
        "break" => TokenType::BREAK,
        "class" => TokenType::CLASS,
        "const" => TokenType::CONST,
        "continue" => TokenType::CONTINUE,
        "else" => TokenType::ELSE,
        "false" => TokenType::FALSE,
        "for" => TokenType::FOR,
        "fun" => TokenType::FUN,
        "if" => TokenType::IF,
        "nil" => TokenType::NIL,
        "or" => TokenType::OR,
        "print" => TokenType::PRINT,
        "println" => TokenType::PRINTLN,
        "return" => TokenType::RETURN,
        "super" => TokenType::SUPER,
        "this" => TokenType::THIS,
        "true" => TokenType::TRUE,
        "var" => TokenType::VAR,
        "while" => TokenType::WHILE,
        _ => TokenType::IDENTIFIER,
    }
}
