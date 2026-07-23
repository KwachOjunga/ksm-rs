//! Abstract syntax tree (AST) types for the Kisumu Lang.
//!
//! The language supports:
//!   - Non-negative integer literals
//!   - Arithmetic operators: `+`, `-`, `*`
//!   - Comparison operators: `<`, `>`, `==`, `!=`, `<=`, `>=`
//!   - Variable references                 // What does this mean exactly?????
//!   - Function calls: `name(args...)`
//!   - `if expr { stmts } else { stmts }` — a conditional statement
//!    ::current GO impl!::  - `if (expr) {statements} else {statements}` - if conditional.
//!   - `while expr { stmts }` — a conditional loop
//!    ::current GO impl!::  - `while (expr) {statments}
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

#![allow(unused)]
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
use pliron::derive::pliron_attr;
use std::fmt::Debug;

// NOTE: Replace this with the repl impl!
#[pliron_attr(name = "kisumu_lang.binop_kind", format, verifier = "succ")]
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
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

#[derive(Debug, Clone, PartialEq)]
enum BuiltinTypes {
    Float(),
    Int(),
    String(String),
    Array,
}

// ANCHOR: ast_expr
/// An expression (has a value).
// NOTE: Replace this with the repl impl!
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /*
     * Integer type can be replaced with a trait bound in its inner type to offer room for custom integer types.
     */
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

/*
 * #####################
 * DELETE ME WHEN DONE
 * #####################
 *
 * The present landmark work serves to append the definition of a struct to the AST.
 * It will neccessitate the update of how the parser is to capture struct definitions.
 */

// Blanket trait to mark types that can be used as struct fields.
trait StructField: Debug + PartialEq {}

#[derive(Debug, Clone, PartialEq)]
enum Type<T: StructField> {
    Builtin(BuiltinTypes),
    Struct(Struct<T>),
}

impl<T: StructField> StructField for Type<T> {}

/// A field in a struct declaration.
#[derive(Debug, Clone, PartialEq)]
struct Field<T: StructField> {
    pub name: String,
    pub ty: T,
}

#[derive(Debug, Clone, PartialEq)]
struct Struct<T: StructField> {
    pub name: String,
    pub fields: Option<Vec<Field<T>>>,
}

// #####################
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

// allow for model methods to be defined within the struct scope
impl StructField for Function {}
