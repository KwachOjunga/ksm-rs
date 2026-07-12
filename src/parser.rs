use crate::ast::{BlockStatement, Expression, Identifier, Program, Statement};
use crate::lexer::Lexer;
use crate::token::{Precedence, Token, TokenType};

// the current parser implemntation eliminates the previous fields that involved ParseFns
pub struct Parser<'a> {
    lexer: &'a mut Lexer,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer) -> Self {
        let mut p = Parser {
            lexer,
            cur_token: Token {
                token_type: TokenType::Illegal,
                literal: "".to_string(),
            },
            peek_token: Token {
                token_type: TokenType::Illegal,
                literal: "".to_string(),
            },
            errors: Vec::new(),
        };

        p.next_token();
        p.next_token();
        p
    }

    pub fn next_token(&mut self) {
        self.cur_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    fn peek_error(&mut self, t: TokenType) {
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead",
            t, self.peek_token.token_type
        );
        self.errors.push(msg);
    }

    fn peek_token_is(&self, t: &TokenType) -> bool {
        self.peek_token.token_type == *t
    }

    fn cur_token_is(&self, t: TokenType) -> bool {
        self.cur_token.token_type == t
    }

    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token_is(&t) {
            self.next_token();
            true
        } else {
            self.peek_error(t);
            false
        }
    }

    fn peek_precedence(&self) -> Precedence {
        self.peek_token.token_type.precedence()
    }

    fn cur_precedence(&self) -> Precedence {
        self.cur_token.token_type.precedence()
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new(),
        };

        while self.cur_token.token_type != TokenType::Eof {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.token_type {
            TokenType::Const => self.parse_let_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::Break => self.parse_break_statement(),
            TokenType::Continue => self.parse_continue_statement(),
            TokenType::LeftBrace => self.parse_block_statement().map(Statement::Block),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::Identifier) {
            return None;
        }

        let name = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::Assign) {
            return None;
        }

        self.next_token();

        let value = match self.parse_expression(Precedence::Lowest) {
            Ok(val) => val,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }

        Some(Statement::Let { token, name, value })
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        self.next_token();

        let return_value = match self.parse_expression(Precedence::Lowest) {
            Ok(val) => val,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }

        Some(Statement::Return {
            token,
            return_value,
        })
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        let expression = match self.parse_expression(Precedence::Lowest) {
            Ok(val) => val,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }

        Some(Statement::Expression { token, expression })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, String> {
        let mut left_exp = match self.cur_token.token_type {
            TokenType::Identifier => self.parse_identifier(),
            TokenType::Int => self.parse_integer_literal()?,
            TokenType::Float => self.parse_float_literal()?,
            TokenType::String => self.parse_string_literal(),
            TokenType::True | TokenType::False => self.parse_boolean(),
            TokenType::Minus | TokenType::Not => self.parse_prefix_expression()?,
            TokenType::LeftParenthesis => self.parse_grouped_expression()?,
            TokenType::Function => self.parse_function_literal()?,
            ref t => {
                return Err(format!("no prefix parse function for {:?} found", t));
            }
        };

        while !self.peek_token_is(&TokenType::Semicolon) && precedence < self.peek_precedence() {
            match self.peek_token.token_type {
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Slash
                | TokenType::Asterisk
                | TokenType::Equal
                | TokenType::NotEqual
                | TokenType::LessThan
                | TokenType::GreaterThan => {
                    self.next_token();
                    left_exp = self.parse_infix_expression(left_exp)?;
                }
                TokenType::LeftParenthesis => {
                    self.next_token();
                    left_exp = self.parse_call_expression(left_exp)?;
                }
                _ => return Ok(left_exp),
            }
        }

        Ok(left_exp)
    }

    fn parse_identifier(&self) -> Expression {
        Expression::Identifier(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        })
    }

    fn parse_integer_literal(&mut self) -> Result<Expression, String> {
        let literal = &self.cur_token.literal;
        let val = if literal.len() > 2 {
            match &literal[0..2] {
                "0x" | "0X" => i64::from_str_radix(&literal[2..], 16),
                "0b" | "0B" => i64::from_str_radix(&literal[2..], 2),
                _ => literal.parse::<i64>(),
            }
        } else {
            literal.parse::<i64>()
        };

        match val {
            Ok(value) => Ok(Expression::Int {
                token: self.cur_token.clone(),
                value,
            }),
            Err(_) => Err(format!("could not parse {:?} as integer", literal)),
        }
    }

    fn parse_float_literal(&mut self) -> Result<Expression, String> {
        let literal = &self.cur_token.literal;
        match literal.parse::<f64>() {
            Ok(value) => Ok(Expression::Float {
                token: self.cur_token.clone(),
                value,
            }),
            Err(_) => Err(format!("could not parse {:?} as float", literal)),
        }
    }

    fn parse_string_literal(&self) -> Expression {
        Expression::String {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }
    }

    fn parse_boolean(&self) -> Expression {
        Expression::Bool {
            token: self.cur_token.clone(),
            value: self.cur_token.token_type == TokenType::True,
        }
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();

        self.next_token();

        let right = self.parse_expression(Precedence::Prefix)?;

        Ok(Expression::Prefix {
            token,
            operator,
            right: Box::new(right),
        })
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();

        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;

        Ok(Expression::Infix {
            token,
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression, String> {
        self.next_token();

        let exp = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenType::RightParenthesis) {
            return Err("expected right parenthesis".to_string());
        }

        Ok(exp)
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LeftParenthesis) {
            return None;
        }

        self.next_token();
        let condition = match self.parse_expression(Precedence::Lowest) {
            Ok(val) => val,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if !self.expect_peek(TokenType::RightParenthesis) {
            return None;
        }

        if !self.expect_peek(TokenType::LeftBrace) {
            return None;
        }

        let consequence = self.parse_block_statement()?;

        let mut alternative = None;
        if self.peek_token_is(&TokenType::Else) {
            self.next_token();

            if !self.expect_peek(TokenType::LeftBrace) {
                return None;
            }

            alternative = Some(self.parse_block_statement()?);
        }

        Some(Statement::If {
            token,
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_while_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LeftParenthesis) {
            return None;
        }

        self.next_token();
        let condition = match self.parse_expression(Precedence::Lowest) {
            Ok(val) => val,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if !self.expect_peek(TokenType::RightParenthesis) {
            return None;
        }

        if !self.expect_peek(TokenType::LeftBrace) {
            return None;
        }

        let body = self.parse_block_statement()?;

        Some(Statement::While {
            token,
            condition,
            body,
        })
    }

    fn parse_for_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LeftParenthesis) {
            return None;
        }

        self.next_token();

        // Parse initializer (optional)
        let mut initializer = None;
        if self.cur_token.token_type != TokenType::Semicolon {
            if self.cur_token.token_type == TokenType::Const {
                initializer = self.parse_let_statement();
            } else {
                initializer = self.parse_expression_statement();
            }
            if self.cur_token.token_type != TokenType::Semicolon
                && self.peek_token_is(&TokenType::Semicolon)
            {
                self.next_token();
            }
        }

        if self.cur_token.token_type != TokenType::Semicolon {
            self.peek_error(TokenType::Semicolon);
            return None;
        }

        self.next_token();

        // Parse condition (optional)
        let mut condition = None;
        if self.cur_token.token_type != TokenType::Semicolon {
            condition = match self.parse_expression(Precedence::Lowest) {
                Ok(val) => Some(val),
                Err(e) => {
                    self.errors.push(e);
                    return None;
                }
            };
            if !self.expect_peek(TokenType::Semicolon) {
                return None;
            }
        } else {
            // Already at semicolon, do nothing
        }

        self.next_token();

        // Parse update (optional)
        let mut update = None;
        if self.cur_token.token_type != TokenType::RightParenthesis {
            if self.cur_token.token_type == TokenType::Const {
                update = self.parse_let_statement();
            } else {
                update = self.parse_expression_statement();
            }
            if self.cur_token.token_type != TokenType::Semicolon
                && self.peek_token_is(&TokenType::Semicolon)
            {
                self.next_token();
            }
        }

        if !self.expect_peek(TokenType::RightParenthesis) {
            return None;
        }

        if !self.expect_peek(TokenType::LeftBrace) {
            return None;
        }

        let body = self.parse_block_statement()?;

        Some(Statement::For {
            token,
            initializer: initializer.map(Box::new),
            condition,
            update: update.map(Box::new),
            body,
        })
    }

    fn parse_break_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }

        Some(Statement::Break { token })
    }

    fn parse_continue_statement(&mut self) -> Option<Statement> {
        let token = self.cur_token.clone();

        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }

        Some(Statement::Continue { token })
    }

    fn parse_block_statement(&mut self) -> Option<BlockStatement> {
        let token = self.cur_token.clone();
        let mut statements = Vec::new();

        self.next_token();

        while !self.cur_token_is(TokenType::RightBrace) && !self.cur_token_is(TokenType::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Some(BlockStatement { token, statements })
    }

    fn parse_function_literal(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LeftParenthesis) {
            return Err("expected left parenthesis".to_string());
        }

        let parameters = self.parse_function_parameters()?;

        if !self.expect_peek(TokenType::LeftBrace) {
            return Err("expected left brace".to_string());
        }

        let body = match self.parse_block_statement() {
            Some(b) => b,
            None => return Err("failed to parse block statement".to_string()),
        };

        Ok(Expression::Function {
            token,
            parameters,
            body,
        })
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Identifier>, String> {
        let mut identifiers = Vec::new();

        if self.peek_token_is(&TokenType::RightParenthesis) {
            self.next_token();
            return Ok(identifiers);
        }

        self.next_token();

        let ident = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };
        identifiers.push(ident);

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let ident = Identifier {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            };
            identifiers.push(ident);
        }

        if !self.expect_peek(TokenType::RightParenthesis) {
            return Err("expected right parenthesis".to_string());
        }

        Ok(identifiers)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let arguments = self.parse_call_arguments()?;
        Ok(Expression::Call {
            token,
            function: Box::new(function),
            arguments,
        })
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Expression>, String> {
        let mut args = Vec::new();

        if self.peek_token_is(&TokenType::RightParenthesis) {
            self.next_token();
            return Ok(args);
        }

        self.next_token();
        args.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Lowest)?);
        }

        if !self.expect_peek(TokenType::RightParenthesis) {
            return Err("expected right parenthesis".to_string());
        }

        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Config;

    fn check_parser_errors(parser: &Parser) {
        let errors = parser.errors();
        if errors.is_empty() {
            return;
        }

        panic!("parser has {} errors:\n{}", errors.len(), errors.join("\n"));
    }

    #[test]
    fn test_parse_int_node() {
        let input = "42";
        let mut l = Lexer::new(input, Config::default());
        let mut p = Parser::new(&mut l);
        let program = p.parse_program();

        check_parser_errors(&p);

        assert_eq!(program.statements.len(), 1);
        let stmt = &program.statements[0];

        match stmt {
            Statement::Expression { expression, .. } => match expression {
                Expression::Int { value, .. } => {
                    assert_eq!(*value, 42);
                }
                other => panic!("expected Int expression, got {:?}", other),
            },
            other => panic!("expected Expression statement, got {:?}", other),
        }
    }

    #[test]
    fn test_integer_literal_expression() {
        struct Test {
            input: &'static str,
            expected: i64,
        }

        let tests = vec![
            Test {
                input: "5;",
                expected: 5,
            },
            Test {
                input: "10;",
                expected: 10,
            },
            Test {
                input: "0;",
                expected: 0,
            },
            Test {
                input: "123456789;",
                expected: 123456789,
            },
            Test {
                input: "0x2A;",
                expected: 42,
            },
            Test {
                input: "0b101010;",
                expected: 42,
            },
        ];

        for t in tests {
            let mut l = Lexer::new(t.input, Config::default());
            let mut p = Parser::new(&mut l);
            let program = p.parse_program();

            check_parser_errors(&p);

            assert_eq!(program.statements.len(), 1);
            let stmt = &program.statements[0];

            match stmt {
                Statement::Expression { expression, .. } => match expression {
                    Expression::Int { value, .. } => {
                        assert_eq!(*value, t.expected);
                    }
                    other => panic!("expected Int expression, got {:?}", other),
                },
                other => panic!("expected Expression statement, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        struct Test {
            input: &'static str,
            operator: &'static str,
            value: i64,
        }

        let tests = vec![
            Test {
                input: "-15;",
                operator: "-",
                value: 15,
            },
            Test {
                input: "-0x2A;",
                operator: "-",
                value: 42,
            },
            Test {
                input: "-0b1010;",
                operator: "-",
                value: 10,
            },
        ];

        for t in tests {
            let mut l = Lexer::new(t.input, Config::default());
            let mut p = Parser::new(&mut l);
            let program = p.parse_program();

            check_parser_errors(&p);

            assert_eq!(program.statements.len(), 1);
            let stmt = &program.statements[0];

            match stmt {
                Statement::Expression { expression, .. } => match expression {
                    Expression::Prefix {
                        operator, right, ..
                    } => {
                        assert_eq!(operator, t.operator);
                        match &**right {
                            Expression::Int { value, .. } => {
                                assert_eq!(*value, t.value);
                            }
                            other => panic!("expected right to be Int, got {:?}", other),
                        }
                    }
                    other => panic!("expected Prefix expression, got {:?}", other),
                },
                other => panic!("expected Expression statement, got {:?}", other),
            }
        }
    }
}
