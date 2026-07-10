#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    Illegal,
    Eof,

    Identifier,
    Float,
    Int,
    String,
    Bool,
    UnterminatedString,

    // Operators
    Assign,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Modulo,
    Power,

    // Bitwise operators
    BitAnd,
    BitOr,
    BitNot,

    // Logical operators
    And,
    Or,

    // Comparisons
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    Not,
    NotEqual,

    Comma,
    Semicolon,

    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    Function,
    True,
    False,
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    Return,
    Const,
    Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl TokenType {
    pub fn precedence(&self) -> Precedence {
        match self {
            TokenType::Equal | TokenType::NotEqual => Precedence::Equals,
            TokenType::LessThan | TokenType::GreaterThan => Precedence::LessGreater,
            TokenType::Plus | TokenType::Minus => Precedence::Sum,
            TokenType::Slash | TokenType::Asterisk => Precedence::Product,
            TokenType::LeftParenthesis => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Precedence {
    Lowest = 1,
    Equals = 2,      // ==
    LessGreater = 3, // > or <
    Sum = 4,         // +
    Product = 5,     // *
    Prefix = 6,      // -X or !X
    Call = 7,        // myFunction(X)
}

pub fn lookup_identifier(identifier: &str) -> TokenType {
    match identifier {
        "func" => TokenType::Function,
        "const" => TokenType::Const,
        "int" => TokenType::Type,
        "string" => TokenType::Type,
        "bool" => TokenType::Type,
        "true" => TokenType::True,
        "false" => TokenType::False,
        "if" => TokenType::If,
        "else" => TokenType::Else,
        "while" => TokenType::While,
        "for" => TokenType::For,
        "break" => TokenType::Break,
        "continue" => TokenType::Continue,
        "return" => TokenType::Return,
        _ => TokenType::Identifier,
    }
}
