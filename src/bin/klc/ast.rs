//! Abstract syntax tree (AST) types for the Kisumu Lang.
//!
//! The language supports:
//!   - Non-negative integer literals
//!   - Arithmetic operators: `+`, `-`, `*`
//!   - Comparison operators: `<`, `>`, `==`, `!=`, `<=`, `>=`
//!   - Variable references                 // What does this mean exactly?????
//!   - Function calls: `name(args...)`
//!   - `if expr { stmts } else { stmts }` — a conditional statement    : DELETE
//!   - `if (expr) {statements} else {statements}` - if conditional.
//!   - `while expr { stmts }` — a conditional loop                     : DELETE
//!   - `while (expr) {statments}
//!   - `const name;` / `const name = expr;` — variable declaration  statement              // This needs to be looked at seriously. What does it mean to have a const def that can be mut.
//!   - `name = expr;` — assignment statement
//!   - `return expr;` — return statement
//!   - Function definitions: `func name(params) { stmts }`

// NOTES:
// The current implementation similar to the go implementation does not support `break` and `continue`.

// This  implementation surpasses what is currently defined by Go by:
//  - assignment is defined
//  - we have the option to switch the parser easily here. (room for more experimentation)
//  - we are closer to machine code
use combine::{
    EasyParser, Parser, Stream, attempt, between, choice, eof,
    error::{ParseError, StdParseResult},
    not_followed_by, optional,
    parser::{
        char::{alpha_num, char, digit, letter, spaces, string},
        repeat::{many, many1, sep_by},
    },
    token,
};

// NOTE: Replace this with the repl impl!
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    /// Addition `+`
    Add,
    /// Subtraction `-`
    Sub,
    /// Multiplication `*`
    Mul,
    /// Less-than `<`
    Lt,
    /// Greater-than `>`
    Gt,
    /// Less-or-equal `<=`
    Le,
    /// Greater-or-equal `>=`
    Ge,
    /// Equal `==`
    Eq,
    /// Not-equal `!=`
    Ne,
}
// ANCHOR_END: ast_bin_op

// ANCHOR: ast_expr
/// An expression (has a value).

// NOTE: Replace this with the repl impl!
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A non-negative integer literal.
    Integer(i64),
    /// A variable reference.
    Variable(String),
    /// A binary operation.
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// A function call.
    Call { callee: String, args: Vec<Expr> },
}

// NOTE: Replace this with the repl impl!
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Variable declaration: `var name;` or `var name = expr;`
    VarDecl { name: String, init: Option<Expr> },
    /// Assignment to an existing variable: `name = expr;`
    Assign { name: String, value: Expr },
    /// Return statement: `return expr;`
    Return(Expr),
    /// While loop: `while cond { body }`.
    While { cond: Expr, body: Vec<Stmt> },
    /// Conditional statement: `if cond { then_body } else { else_body }`.
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    /// An expression used as a statement: `expr;`
    Expr(Expr),
}

// NOTE: Replace this with the repl impl!
/// A top-level function definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}
