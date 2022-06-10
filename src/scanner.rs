use std::collections::HashMap;
use substring::Substring;
use crate::token::{Token, TokenType};

///
pub struct Scanner {
    pub source: String,
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub is_block_comment: bool,
    pub keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: &String) -> Self {
        Scanner {
            source: source.to_string(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
            is_block_comment: false,
            keywords: HashMap::from([
                ("and".to_string(), TokenType::And),
                ("class".to_string(), TokenType::Class),
                ("false".to_string(), TokenType::False),
                ("for".to_string(), TokenType::For),
                ("fun".to_string(), TokenType::Fun),
                ("if".to_string(), TokenType::If),
                ("else".to_string(), TokenType::Else),
                ("nil".to_string(), TokenType::Nil),
                ("or".to_string(), TokenType::Or),
                ("print".to_string(), TokenType::Print),
                ("super".to_string(), TokenType::Super),
                ("this".to_string(), TokenType::This),
                ("true".to_string(), TokenType::True),
                ("var".to_string(), TokenType::Var),
                ("while".to_string(), TokenType::While),
                ("extend".to_string(), TokenType::Extend),
                ("return".to_string(), TokenType::Return)
            ]),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            // Beginning of next lexeme
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(TokenType::Eof, "".to_string(), "".to_string(), self.line));
        self.tokens.to_vec()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        if self.is_block_comment {
            if c == '*' && self._match(&'/') {
                self.is_block_comment = false;
            }
            return; // Ignore processing rest of the token in block comment mode
        }
        match c {
            '(' => { self.add_token(&TokenType::LeftParen) }
            ')' => { self.add_token(&TokenType::RightParen) }
            '{' => { self.add_token(&TokenType::LeftBrace) }
            '}' => { self.add_token(&TokenType::RightBrace) }
            ',' => { self.add_token(&TokenType::Comma) }
            '.' => { self.add_token(&TokenType::Dot) }
            '-' => {
                let is_match = self._match(&'=');
                self.add_token(&if is_match  { TokenType::MinusEqual } else { TokenType::Minus})
            }
            '+' => {
                let is_match = self._match(&'=');
                self.add_token(&if is_match  { TokenType::PlusEqual } else { TokenType::Plus})
            }
            ';' => { self.add_token(&TokenType::Semicolon) }
            '*' => { self.add_token(&TokenType::Star) }
            '!' => {
                let is_match = self._match(&'=');
                self.add_token(&if is_match { TokenType::BangEqual } else { TokenType::Bang })
            }
            '=' => {
                let is_match = self._match(&'=');
                self.add_token(&if is_match { TokenType::EqualEqual } else { TokenType::Equal })
            }
            '<' => {
                let is_match = self._match(&'=');
                self.add_token(&if is_match { TokenType::LessEqual } else { TokenType::Less })
            }
            '>' => {
                let is_match = self._match(&'=');
                self.add_token(&if is_match { TokenType::GreaterEqual } else { TokenType::Greater })
            }
            '/' => {
                let is_match_slash = self._match(&'/');
                let is_match_star = self._match(&'*');
                if is_match_slash {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if is_match_star {
                    self.is_block_comment = true;
                    while self.peek() != '*' && self.peek_next() != '/' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(&TokenType::Slash)
                }
            }
            |' '| '\r' |'\t' => { /* ignore me */ }
            '\n' => {
                self.line = self.line + 1
            }
            '"' => {
                self.string()
            }
            'o' => {
                if self._match(&'r') {
                    self.add_token(&TokenType::Or);
                } else if self.is_alpha(c) {
                    self.identifier();
                }
            }
            _ => {
                if self.is_digit(c) {
                    self.number();
                }  else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    self.error(self.line, "".to_string(), "Unexpected character .".to_string());
                }
            }
        }
    }

    fn error(&self, line: usize, location: String, message: String) {
        eprintln!("[line {0} ] Error {1} : {2}", line, location, message );
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }
        //Look for fractional bit
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
        self.add_token_literal(&TokenType::Number,
                               &self.source.substring(self.start, self.current).to_string());
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let text = self.source.substring(self.start, self.current).to_string();
        let token_type: TokenType;
        let optional_token_type = self.keywords.get(&text);
        match optional_token_type {
            Some(p) => {
                token_type = *p;
            }
            None => {
                token_type = TokenType::Identifier;
            }
        }
        self.add_token(&token_type);
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        return self.is_alpha(c) || self.is_digit(c);
    }

    fn is_alpha(&self, c: char) -> bool {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn advance(&mut self) -> char {
        let result = self.source.chars().nth(self.current).unwrap();
        self.current = self.current + 1;
        return result;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return char::default();
        }
        return self.source.chars().nth(self.current).unwrap();
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return char::default();
        }
        return self.source.chars().nth(self.current + 1).unwrap();
    }

    fn _match(&mut self, expected: &char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != *expected {
            return false;
        }
        self.current = self.current + 1;
        return true;
    }

    fn add_token_literal(&mut self, token: &TokenType, literal: &String) {
        let text = self.source.substring(self.start, self.current).to_string();
        self.tokens.push(Token::new(*token, text, literal.to_string(), self.line));
    }

    fn add_token(&mut self, token: &TokenType) {
        self.add_token_literal(token, &"".to_string());
    }

    fn is_digit(&self, c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line = self.line + 1
            }
            self.advance();
        }
        if self.is_at_end() {
            self.error(self.line, "".to_string(),"Unterminated string.".to_string());
            return;
        }
        self.advance(); // closing "
        let value = self.source.substring(self.start + 1, self.current - 1).to_string();
        self.add_token_literal(&TokenType::String, &value);
    }
}
