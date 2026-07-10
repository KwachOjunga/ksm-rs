use crate::token::{self, Token, TokenType};

#[derive(Debug, Clone)]
pub struct Config {
    pub allow_comments: bool,
    pub string_delimiters: Vec<char>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            allow_comments: true,
            string_delimiters: vec!['"', '\''],
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    char: char,
    pub line: usize,
    pub column: usize,
    config: Config,
}

impl Lexer {
    pub fn new(input: &str, config: Config) -> Self {
        let mut l = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            char: '\0',
            line: 1,
            column: 0,
            config,
        };
        l.read_current_character();
        l
    }

    fn read_current_character(&mut self) {
        if self.read_position >= self.input.len() {
            self.char = '\0';
        } else {
            self.char = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
        if self.char == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
    }

    fn peek_next_character(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    fn skip_whitespace(&mut self) {
        while self.char.is_whitespace() {
            self.read_current_character();
        }

        if self.config.allow_comments {
            // Handle single-line comments
            if self.char == '/' && self.peek_next_character() == '/' {
                while self.char != '\n' && self.char != '\0' {
                    self.read_current_character();
                }
                self.skip_whitespace();
            }

            // Handle multi-line comments
            if self.char == '/' && self.peek_next_character() == '*' {
                self.read_current_character();
                self.read_current_character();
                while (self.char != '*' || self.peek_next_character() != '/') && self.char != '\0' {
                    self.read_current_character();
                }
                if self.char == '*' {
                    self.read_current_character();
                    self.read_current_character();
                } else {
                    println!(
                        "Unterminated comment at line {}, column {}",
                        self.line, self.column
                    );
                }
                self.skip_whitespace();
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while is_letter(self.char) {
            self.read_current_character();
        }
        self.input[start..self.position].iter().collect()
    }

    fn read_number(&mut self) -> (String, TokenType) {
        let start = self.position;
        let mut token_type = TokenType::Int;

        // Check for hexadecimal (0x prefix)
        if self.char == '0' && (self.peek_next_character() == 'x' || self.peek_next_character() == 'X') {
            self.read_current_character(); // consume '0'
            self.read_current_character(); // consume 'x' / 'X'

            while is_hex_digit(self.char) {
                self.read_current_character();
            }
            let val = self.input[start..self.position].iter().collect();
            return (val, TokenType::Int);
        }

        // Check for binary (0b prefix)
        if self.char == '0' && (self.peek_next_character() == 'b' || self.peek_next_character() == 'B') {
            self.read_current_character(); // consume '0'
            self.read_current_character(); // consume 'b' / 'B'

            while is_binary_digit(self.char) {
                self.read_current_character();
            }
            let val = self.input[start..self.position].iter().collect();
            return (val, TokenType::Int);
        }

        while is_digit(self.char) {
            self.read_current_character();
        }

        // Check if it's a float (has a dot)
        if self.char == '.' {
            self.read_current_character();

            while is_digit(self.char) {
                self.read_current_character();
            }
            token_type = TokenType::Float;
        }

        let val = self.input[start..self.position].iter().collect();
        (val, token_type)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let tok = match self.char {
            '=' => self.match_two_char_token('=', TokenType::Assign, TokenType::Equal),
            '!' => self.match_two_char_token('=', TokenType::Not, TokenType::NotEqual),
            '<' => self.match_two_char_token('=', TokenType::LessThan, TokenType::LessEqual),
            '>' => self.match_two_char_token('=', TokenType::GreaterThan, TokenType::GreaterEqual),
            '&' => {
                if self.peek_next_character() == '&' {
                    self.read_current_character();
                    Token {
                        token_type: TokenType::And,
                        literal: "&&".to_string(),
                    }
                } else {
                    Token {
                        token_type: TokenType::Illegal,
                        literal: self.char.to_string(),
                    }
                }
            }
            '|' => {
                if self.peek_next_character() == '|' {
                    self.read_current_character();
                    Token {
                        token_type: TokenType::Or,
                        literal: "||".to_string(),
                    }
                } else {
                    Token {
                        token_type: TokenType::Illegal,
                        literal: self.char.to_string(),
                    }
                }
            }
            '"' | '\'' => {
                if self.config.string_delimiters.contains(&self.char) {
                    let start_delimiter = self.char;
                    self.read_current_character();
                    let start = self.position;

                    while self.char != start_delimiter && self.char != '\0' {
                        self.read_current_character();
                    }

                    let mut tok = Token {
                        token_type: TokenType::Illegal,
                        literal: "".to_string(),
                    };

                    if self.char == '\0' {
                        tok.token_type = TokenType::Illegal;
                        tok.literal = "unterminated string".to_string();
                    } else {
                        tok.token_type = TokenType::String;
                        tok.literal = self.input[start..self.position].iter().collect();
                    }

                    if self.char != '\0' {
                        self.read_current_character();
                    }
                    return tok;
                }
                Token {
                    token_type: TokenType::Illegal,
                    literal: self.char.to_string(),
                }
            }
            '.' => {
                if is_digit(self.peek_next_character()) {
                    let start = self.position;
                    self.read_current_character(); // consume '.'

                    if !is_digit(self.char) {
                        Token {
                            token_type: TokenType::Illegal,
                            literal: ".".to_string(),
                        }
                    } else {
                        while is_digit(self.char) {
                            self.read_current_character();
                        }
                        Token {
                            token_type: TokenType::Float,
                            literal: self.input[start..self.position].iter().collect(),
                        }
                    }
                } else {
                    Token {
                        token_type: TokenType::Illegal,
                        literal: ".".to_string(),
                    }
                }
            }
            '\0' => Token {
                token_type: TokenType::Eof,
                literal: "".to_string(),
            },
            _ => {
                if is_letter(self.char) {
                    let literal = self.read_identifier();
                    let token_type = token::lookup_identifier(&literal);
                    return Token { token_type, literal };
                } else if is_digit(self.char) {
                    let (literal, token_type) = self.read_number();
                    return Token { token_type, literal };
                } else if let Some(tok_type) = single_char_token(self.char) {
                    Token {
                        token_type: tok_type,
                        literal: self.char.to_string(),
                    }
                } else {
                    Token {
                        token_type: TokenType::Illegal,
                        literal: self.char.to_string(),
                    }
                }
            }
        };

        self.read_current_character();
        tok
    }

    fn match_two_char_token(
        &mut self,
        next_char: char,
        single_type: TokenType,
        double_type: TokenType,
    ) -> Token {
        if self.peek_next_character() == next_char {
            let current = self.char;
            self.read_current_character();
            let literal = format!("{}{}", current, self.char);
            Token {
                token_type: double_type,
                literal,
            }
        } else {
            Token {
                token_type: single_type,
                literal: self.char.to_string(),
            }
        }
    }
}

fn is_letter(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn is_binary_digit(c: char) -> bool {
    c == '0' || c == '1'
}

fn single_char_token(c: char) -> Option<TokenType> {
    match c {
        '+' => Some(TokenType::Plus),
        '-' => Some(TokenType::Minus),
        '*' => Some(TokenType::Asterisk),
        '/' => Some(TokenType::Slash),
        ',' => Some(TokenType::Comma),
        ';' => Some(TokenType::Semicolon),
        '(' => Some(TokenType::LeftParenthesis),
        ')' => Some(TokenType::RightParenthesis),
        '{' => Some(TokenType::LeftBrace),
        '}' => Some(TokenType::RightBrace),
        '[' => Some(TokenType::LeftBracket),
        ']' => Some(TokenType::RightBracket),
        '%' => Some(TokenType::Modulo),
        '^' => Some(TokenType::Power),
        '&' => Some(TokenType::BitAnd),
        '|' => Some(TokenType::BitOr),
        '~' => Some(TokenType::BitNot),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_edge_cases() {
        struct Test {
            name: &'static str,
            input: &'static str,
            expected: Vec<Token>,
        }

        let tests = vec![
            Test {
                name: "Empty input",
                input: "",
                expected: vec![
                    Token { token_type: TokenType::Eof, literal: "".to_string() },
                ],
            },
            Test {
                name: "Unterminated string",
                input: "\"unclosed string",
                expected: vec![
                    Token { token_type: TokenType::Illegal, literal: "unterminated string".to_string() },
                    Token { token_type: TokenType::Eof, literal: "".to_string() },
                ],
            },
            Test {
                name: "Unterminated multiline comment",
                input: "/* unclosed comment",
                expected: vec![
                    Token { token_type: TokenType::Eof, literal: "".to_string() },
                ],
            },
            Test {
                name: "Numbers with decimal points",
                input: "42.5 3.14159 .5 5.",
                expected: vec![
                    Token { token_type: TokenType::Float, literal: "42.5".to_string() },
                    Token { token_type: TokenType::Float, literal: "3.14159".to_string() },
                    Token { token_type: TokenType::Float, literal: ".5".to_string() },
                    Token { token_type: TokenType::Float, literal: "5.".to_string() },
                    Token { token_type: TokenType::Eof, literal: "".to_string() },
                ],
            },
            Test {
                name: "Complex nested expressions",
                input: "((x + y) * (z - 1)) / 2",
                expected: vec![
                    Token { token_type: TokenType::LeftParenthesis, literal: "(".to_string() },
                    Token { token_type: TokenType::LeftParenthesis, literal: "(".to_string() },
                    Token { token_type: TokenType::Identifier, literal: "x".to_string() },
                    Token { token_type: TokenType::Plus, literal: "+".to_string() },
                    Token { token_type: TokenType::Identifier, literal: "y".to_string() },
                    Token { token_type: TokenType::RightParenthesis, literal: ")".to_string() },
                    Token { token_type: TokenType::Asterisk, literal: "*".to_string() },
                    Token { token_type: TokenType::LeftParenthesis, literal: "(".to_string() },
                    Token { token_type: TokenType::Identifier, literal: "z".to_string() },
                    Token { token_type: TokenType::Minus, literal: "-".to_string() },
                    Token { token_type: TokenType::Int, literal: "1".to_string() },
                    Token { token_type: TokenType::RightParenthesis, literal: ")".to_string() },
                    Token { token_type: TokenType::RightParenthesis, literal: ")".to_string() },
                    Token { token_type: TokenType::Slash, literal: "/".to_string() },
                    Token { token_type: TokenType::Int, literal: "2".to_string() },
                    Token { token_type: TokenType::Eof, literal: "".to_string() },
                ],
            },
            Test {
                name: "Mixed comments and code",
                input: "// Line comment\n\t\t\tx = 1; /* Multi\n\t\t\tline comment */ y = 2;\n\t\t\t// Another comment",
                expected: vec![
                    Token { token_type: TokenType::Identifier, literal: "x".to_string() },
                    Token { token_type: TokenType::Assign, literal: "=".to_string() },
                    Token { token_type: TokenType::Int, literal: "1".to_string() },
                    Token { token_type: TokenType::Semicolon, literal: ";".to_string() },
                    Token { token_type: TokenType::Identifier, literal: "y".to_string() },
                    Token { token_type: TokenType::Assign, literal: "=".to_string() },
                    Token { token_type: TokenType::Int, literal: "2".to_string() },
                    Token { token_type: TokenType::Semicolon, literal: ";".to_string() },
                    Token { token_type: TokenType::Eof, literal: "".to_string() },
                ],
            },
            Test {
                name: "Special characters and operators",
                input: "!= == < <= > >= && ||",
                expected: vec![
                    Token { token_type: TokenType::NotEqual, literal: "!=".to_string() },
                    Token { token_type: TokenType::Equal, literal: "==".to_string() },
                    Token { token_type: TokenType::LessThan, literal: "<".to_string() },
                    Token { token_type: TokenType::LessEqual, literal: "<=".to_string() },
                    Token { token_type: TokenType::GreaterThan, literal: ">".to_string() },
                    Token { token_type: TokenType::GreaterEqual, literal: ">=".to_string() },
                    Token { token_type: TokenType::And, literal: "&&".to_string() },
                    Token { token_type: TokenType::Or, literal: "||".to_string() },
                    Token { token_type: TokenType::Eof, literal: "".to_string() },
                ],
            },
        ];

        for t in tests {
            let mut l = Lexer::new(t.input, Config::default());
            for (i, expected_tok) in t.expected.iter().enumerate() {
                let got = l.next_token();
                assert_eq!(
                    got, *expected_tok,
                    "test '{}' - token[{}] wrong. expected={:?}, got={:?}",
                    t.name, i, expected_tok, got
                );
            }
        }
    }

    #[test]
    fn test_lexer_configuration() {
        struct Test {
            name: &'static str,
            input: &'static str,
            config: Config,
            expected: Vec<Token>,
        }

        let tests = vec![
            Test {
                name: "Disabled comments",
                input: "x = 1; // comment \n y = 2;",
                config: Config {
                    allow_comments: false,
                    string_delimiters: vec!['"'],
                },
                expected: vec![
                    Token { token_type: TokenType::Identifier, literal: "x".to_string() },
                    Token { token_type: TokenType::Assign, literal: "=".to_string() },
                    Token { token_type: TokenType::Int, literal: "1".to_string() },
                    Token { token_type: TokenType::Semicolon, literal: ";".to_string() },
                    Token { token_type: TokenType::Slash, literal: "/".to_string() },
                    Token { token_type: TokenType::Slash, literal: "/".to_string() },
                    Token { token_type: TokenType::Identifier, literal: "comment".to_string() },
                    Token { token_type: TokenType::Identifier, literal: "y".to_string() },
                    Token { token_type: TokenType::Assign, literal: "=".to_string() },
                    Token { token_type: TokenType::Int, literal: "2".to_string() },
                    Token { token_type: TokenType::Semicolon, literal: ";".to_string() },
                    Token { token_type: TokenType::Eof, literal: "".to_string() },
                ],
            },
            Test {
                name: "Custom string delimiters",
                input: "'single' \"double\"",
                config: Config {
                    allow_comments: true,
                    string_delimiters: vec!['\'', '"'],
                },
                expected: vec![
                    Token { token_type: TokenType::String, literal: "single".to_string() },
                    Token { token_type: TokenType::String, literal: "double".to_string() },
                    Token { token_type: TokenType::Eof, literal: "".to_string() },
                ],
            },
        ];

        for t in tests {
            let mut l = Lexer::new(t.input, t.config);
            for (i, expected_tok) in t.expected.iter().enumerate() {
                let got = l.next_token();
                assert_eq!(
                    got, *expected_tok,
                    "test '{}' - token[{}] wrong. expected={:?}, got={:?}",
                    t.name, i, expected_tok, got
                );
            }
        }
    }
}
