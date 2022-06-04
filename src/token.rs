use std::fmt;

pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: String,
    pub line: usize,
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Token::new(self.token_type,
                   self.lexeme.to_string(),
                   self.literal.to_string(),
                   self.line)
    }
}

impl Token {
    pub fn new(token_type: TokenType,
               lexeme: String,
               literal: String,
               line: usize ) -> Token {
        Token {
            token_type,
            lexeme,
            literal,
            line
        }
    }
    pub fn to_string(&self)->String {
        return [self.token_type.to_string(),
            self.lexeme.to_string(),
            self.literal.to_string()].join(", ")
    }
}

#[derive(Copy, Clone)]
#[derive(Eq, PartialEq)]
#[derive(Hash)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    PlusEqual,
    MinusEqual,
    // Identifiers
    Identifier,
    String,
    Number,
    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    Eof
}
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::LeftParen => write!(f, "LeftParen"),
            TokenType::RightParen => write!(f, "RightParen"),
            TokenType::LeftBrace => write!(f, "LeftBrace"),
            TokenType::RightBrace => write!(f, "RightBrace"),
            TokenType::Comma => write!(f, "COMMA"),
            TokenType::Dot => write!(f, "DOT"),
            TokenType::Minus => write!(f, "MINUS"),
            TokenType::Plus => write!(f, "PLUS"),
            TokenType::Semicolon => write!(f, "SEMICOLON"),
            TokenType::Slash => write!(f, "SLASH"),
            TokenType::Star => write!(f, "STAR"),
            TokenType::Bang => write!(f, "BANG"),
            TokenType::BangEqual => write!(f, "BangEqual"),
            TokenType::Equal => write!(f, "EQUAL"),
            TokenType::EqualEqual => write!(f, "EqualEqual"),
            TokenType::Greater => write!(f, "GREATER"),
            TokenType::GreaterEqual => write!(f, "GreaterEqual"),
            TokenType::Less => write!(f, "LESS"),
            TokenType::LessEqual => write!(f, "LessEqual"),
            TokenType::PlusEqual => write!(f, "PlusEqual"),
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::String => write!(f, "String"),
            TokenType::Number => write!(f, "Number"),
            TokenType::And => write!(f, "And"),
            TokenType::Class => write!(f, "Class"),
            TokenType::Else => write!(f, "Else"),
            TokenType::This => write!(f, "This"),
            TokenType::True => write!(f, "True"),
            TokenType::Var => write!(f, "Var"),
            TokenType::While => write!(f, "While"),
            TokenType::Print => write!(f, "Print"),
            TokenType::Return => write!(f, "Return"),
            TokenType::Fun => write!(f, "Fun"),
            TokenType::Eof => write!(f, "Eof"),
            _ =>  write!(f, "TOKEN"),
        }
    }
}


