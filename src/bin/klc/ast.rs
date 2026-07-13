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

// NOTE: Replace this with the repl impl!
#[pliron_attr(name = "kaleidoscope.binop_kind", format, verifier = "succ")]
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

/// Captures white space
fn ws<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().map(|_| ())
}

/// Parse an identifier
/// Identifiers start with a letter or `_`, followed by letters, digits, or `_`.
fn ident_<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (letter().or(char('_')), many(alpha_num().or(char('_'))))
        .map(|(first, rest): (char, String)| {
            let mut s = String::new();
            s.push(first);
            s.push_str(&rest);
            s
        })
        .skip(ws())
}
// ANCHOR_END: ident_parser

// NONNEGATIVEINT_parser
/// Parse a non-negative integer literal
fn integer_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1::<String, _, _>(digit())
        .map(|digits: String| Expr::Integer(digits.parse::<i64>().unwrap()))
        .skip(ws())
}

// KEYWORD_parser
/// Parse an exact keyword followed by a non-identifier character, then skip
/// trailing whitespace.  Prevents `iffy` from matching keyword `if`.
// ANCHOR: keyword_parser
fn keyword<Input>(kw: &'static str) -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    attempt(string(kw).skip(not_followed_by(alpha_num().or(char('_')))))
        .skip(ws())
        .map(|_| ())
}

/// Parse a single character token and skip trailing whitespace.
fn tok<Input>(c: char) -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    token(c).skip(ws())
}

/// Parse an exact multi-character symbol and skip trailing whitespace.
/// Uses `attempt` so that e.g. `<=` doesn't partially consume `<`.
fn sym<Input>(s: &'static str) -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    attempt(string(s)).skip(ws()).map(|_| ())
}

// ---- Recursive function pointers -----------------------------------------
//
// combine's type system does not allow `impl Parser` to reference itself, so
// recursive grammars (expr -> primary -> expr) need a concrete named type to
// break the cycle.  Using a regular Rust function pointer achieves this:
// `fn(&mut Input) -> StdParseResult<O, Input>` directly implements
// `Parser<Input>` in combine 4, so functions with that signature can be used
// anywhere a parser is expected without any wrapping.
fn expr_fn<Input>(input: &mut Input) -> StdParseResult<Expr, Input>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    cmp_expr_().parse_stream(input).into_result()
}
// ANCHOR_END: expr_fn

// ANCHOR: stmt_fn
fn stmt_fn<Input>(input: &mut Input) -> StdParseResult<Stmt, Input>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    stmt_().parse_stream(input).into_result()
}
// ANCHOR_END: stmt_fn

// ---- Expression parsers --------------------------------------------------
//
// Operator precedence (highest to lowest):
//   primary  — literals, variables, calls, parenthesized expressions
//   mul_expr — `*`
//   add_expr — `+`, `-`
//   cmp_expr — `<`, `>`, `==`, `!=`, `<=`, `>=` (single comparison per expr)

/// Parse a `primary` expression:
///   - integer literal
///   - function call: `name(args...)`
///   - variable reference
///   - parenthesized expression: `(expr)`
// ANCHOR: primary_parser
fn primary_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice!(
        integer_(),
        // Function call must be tried before plain variable reference so that
        // `foo(...)` doesn't parse as variable `foo` followed by junk.
        attempt(
            ident_()
                .and(between(
                    tok('('),
                    tok(')'),
                    sep_by(
                        // Each argument is a full expression; trailing whitespace
                        // after the comma is already consumed by tok(',').
                        combine::parser(expr_fn::<Input>),
                        tok(','),
                    ),
                ))
                .map(|(callee, args)| Expr::Call { callee, args })
        ),
        between(tok('('), tok(')'), combine::parser(expr_fn::<Input>)),
        ident_().map(Expr::Variable)
    )
}
// ANCHOR_END: primary_parser

/// Parse a multiplicative expression: `primary ('*' primary)*`.
// ANCHOR: mul_expr_parser
fn mul_expr_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        primary_(),
        many::<Vec<_>, _, _>((tok('*').map(|_| BinOp::Mul)).and(primary_())),
    )
        .map(|(first, rest)| {
            rest.into_iter().fold(first, |acc, (op, rhs)| Expr::BinOp {
                op,
                lhs: Box::new(acc),
                rhs: Box::new(rhs),
            })
        })
}
// ANCHOR_END: mul_expr_parser

/// Parse an additive expression: `mul_expr (('+' | '-') mul_expr)*`.
// ANCHOR: add_expr_parser
fn add_expr_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        mul_expr_(),
        many::<Vec<_>, _, _>(
            choice!(tok('+').map(|_| BinOp::Add), tok('-').map(|_| BinOp::Sub)).and(mul_expr_()),
        ),
    )
        .map(|(first, rest)| {
            rest.into_iter().fold(first, |acc, (op, rhs)| Expr::BinOp {
                op,
                lhs: Box::new(acc),
                rhs: Box::new(rhs),
            })
        })
}
// ANCHOR_END: add_expr_parser

/// Parse a comparison expression: `add_expr (cmp_op add_expr)?`.
/// Only a single comparison is allowed per expression (no chaining).
fn cmp_expr_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        add_expr_(),
        optional(
            choice!(
                // Two-character operators must be tried before single-char ones.
                sym("<=").map(|_| BinOp::Le),
                sym(">=").map(|_| BinOp::Ge),
                sym("==").map(|_| BinOp::Eq),
                sym("!=").map(|_| BinOp::Ne),
                tok('<').map(|_| BinOp::Lt),
                tok('>').map(|_| BinOp::Gt)
            )
            .and(add_expr_()),
        ),
    )
        .map(|(lhs, rhs_opt)| match rhs_opt {
            None => lhs,
            Some((op, rhs)) => Expr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
        })
}

// ---- Statement parsers ---------------------------------------------------

/// Parse a variable declaration: `var name;` or `var name = expr;`
fn var_decl_stmt_<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        keyword("const"),
        ident_(),
        optional(tok('=').with(combine::parser(expr_fn::<Input>))),
        tok(';'),
    )
        .map(|(_, name, init, _)| Stmt::VarDecl { name, init })
}

/// Parse an assignment statement: `name = expr;`
fn assign_stmt_<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        ident_(),
        tok('='),
        combine::parser(expr_fn::<Input>),
        tok(';'),
    )
        .map(|(name, _, value, _)| Stmt::Assign { name, value })
}

/// Parse a `return` statement: `return expr;`
fn return_stmt_<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        keyword("return"),
        combine::parser(expr_fn::<Input>),
        tok(';'),
    )
        .map(|(_, value, _)| Stmt::Return(value))
}

/// Parse a `while` statement: `while cond { body }`
// ANCHOR: while_stmt_parser
fn while_stmt_<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        keyword("while"),
        combine::parser(expr_fn::<Input>),
        between(tok('{'), tok('}'), many(combine::parser(stmt_fn::<Input>))),
    )
        .map(|(_, cond, body)| Stmt::While { cond, body })
}
// ANCHOR_END: while_stmt_parser

/// Parse an `if` statement: `if cond { then_body } else { else_body }`
// ANCHOR: if_stmt_parser
fn if_stmt_<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        keyword("if"),
        combine::parser(expr_fn::<Input>),
        between(tok('{'), tok('}'), many(combine::parser(stmt_fn::<Input>))),
        keyword("else"),
        between(tok('{'), tok('}'), many(combine::parser(stmt_fn::<Input>))),
    )
        .map(|(_, cond, then_body, _, else_body)| Stmt::If {
            cond,
            then_body,
            else_body,
        })
}
// ANCHOR_END: if_stmt_parser

/// Parse an expression statement: `expr;`
fn expr_stmt_<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (combine::parser(expr_fn::<Input>), tok(';')).map(|(expr, _)| Stmt::Expr(expr))
}

/// Parse any statement (dispatcher).
// ANCHOR: stmt_parser
fn stmt_<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice!(
        var_decl_stmt_(),
        return_stmt_(),
        while_stmt_(),
        if_stmt_(),
        attempt(assign_stmt_()),
        expr_stmt_()
    )
}
// ANCHOR_END: stmt_parser

// ---- Top-level parsers ---------------------------------------------------

/// Parse a block of statements: `{ stmt* }`
fn block_<Input>() -> impl Parser<Input, Output = Vec<Stmt>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    between(tok('{'), tok('}'), many(combine::parser(stmt_fn::<Input>)))
}

/// Parse a function definition:
///   `def name(param, ...) { stmts }`
// ANCHOR: func_def_parser
fn func_def_<Input>() -> impl Parser<Input, Output = Function>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        keyword("func"),
        ident_(),
        between(tok('('), tok(')'), sep_by(ident_(), tok(','))),
        block_(),
    )
        .map(|(_, name, params, body)| Function { name, params, body })
}
// ANCHOR_END: func_def_parser

/// Parse a complete program: zero or more function definitions.
// ANCHOR: program_parser
fn program_<Input>() -> impl Parser<Input, Output = Vec<Function>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    ws().with(many(func_def_())).skip(eof())
}
// ANCHOR_END: program_parser

// ANCHOR: parse_program
pub fn parse_program(src: &str) -> Result<Vec<Function>, String> {
    program_()
        .easy_parse(src)
        .map(|(funcs, _rest)| funcs)
        .map_err(|err| err.to_string())
}
// ANCHOR_END: parse_program
