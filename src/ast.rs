#![allow(warnings)]
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Statement>,
}

impl std::fmt::Display for BlockStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stmt in &self.statements {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

// TODO: handle instances that require one to perform variable mutation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// statements that evaluate to a variable declaration
    // var name = expr;
    // NOTE: this implies that the value assigned to name includes functions
    Let {
        token: Token,
        name: Identifier,
        value: Expression,
    },
    Return {
        token: Token,
        return_value: Expression,
    },
    Expression {
        token: Token,
        expression: Expression,
    },
    Block(BlockStatement),
    If {
        token: Token,
        condition: Expression,
        consequence: BlockStatement,
        alternative: Option<BlockStatement>,
    },
    While {
        token: Token,
        condition: Expression,
        body: BlockStatement,
    },
    For {
        token: Token,
        initializer: Option<Box<Statement>>,
        condition: Option<Expression>,
        update: Option<Box<Statement>>,
        body: BlockStatement,
    },
    Break {
        token: Token,
    },
    Continue {
        token: Token,
    },
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let { token, name, value } => {
                write!(f, "{} {} = {};", token.literal, name, value)
            }
            Statement::Return {
                token,
                return_value,
            } => {
                write!(f, "{} {};", token.literal, return_value)
            }
            Statement::Expression { expression, .. } => {
                write!(f, "{}", expression)
            }
            Statement::Block(block) => {
                write!(f, "{}", block)
            }
            Statement::If {
                condition,
                consequence,
                alternative,
                ..
            } => {
                write!(f, "if ({}) {}", condition, consequence)?;
                if let Some(alt) = alternative {
                    write!(f, " else {}", alt)?;
                }
                Ok(())
            }
            Statement::While {
                condition, body, ..
            } => {
                write!(f, "while ({}) {}", condition, body)
            }
            Statement::For {
                initializer,
                condition,
                update,
                body,
                ..
            } => {
                write!(f, "for (")?;
                if let Some(init) = initializer {
                    let init_str = init.to_string();
                    // Let / Expression statement ends with semicolon. So it already has it.
                    write!(f, "{}", init_str)?;
                } else {
                    write!(f, ";")?;
                }
                write!(f, " ")?;
                if let Some(cond) = condition {
                    write!(f, "{}", cond)?;
                }
                write!(f, ";")?;
                if let Some(upd) = update {
                    let mut upd_str = upd.to_string();
                    if upd_str.ends_with(';') {
                        upd_str.pop();
                    }
                    write!(f, " {}", upd_str)?;
                }
                write!(f, ") {}", body)
            }
            Statement::Break { .. } => write!(f, "break;"),
            Statement::Continue { .. } => write!(f, "continue;"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Int {
        token: Token,
        value: i64,
    },
    Float {
        token: Token,
        value: f64,
    },
    String {
        token: Token,
        value: String,
    },
    Bool {
        token: Token,
        value: bool,
    },
    Prefix {
        token: Token,
        operator: String,
        right: Box<Expression>,
    },
    Infix {
        token: Token,
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    Function {
        token: Token,
        parameters: Vec<Identifier>,
        body: BlockStatement,
    },
    Call {
        token: Token,
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Identifier(ident) => write!(f, "{}", ident),
            Expression::Int { token, .. } => write!(f, "{}", token.literal),
            Expression::Float { token, .. } => write!(f, "{}", token.literal),
            Expression::String { token, .. } => write!(f, "{}", token.literal),
            Expression::Bool { token, .. } => write!(f, "{}", token.literal),
            Expression::Prefix {
                operator, right, ..
            } => {
                write!(f, "({}{})", operator, right)
            }
            Expression::Infix {
                left,
                operator,
                right,
                ..
            } => {
                write!(f, "({} {} {})", left, operator, right)
            }
            Expression::Function {
                parameters, body, ..
            } => {
                let params: Vec<String> = parameters.iter().map(|p| p.to_string()).collect();
                write!(f, "func({}) {}", params.join(", "), body)
            }
            Expression::Call {
                function,
                arguments,
                ..
            } => {
                let args: Vec<String> = arguments.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", function, args.join(", "))
            }
            _ => unimplemented!("Whatever expr you are trying to print is yet to be implemented!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stmt in &self.statements {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}
