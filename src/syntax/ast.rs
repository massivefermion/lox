#![allow(dead_code)]

use crate::runtime::value::Value;

/// Placeholder type for the `Option<Type>` annotation field on every AST node.
/// The real Type enum arrives in milestone 2; for now this is a unit-like marker.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Type;

// ---------------------------------------------------------------------------
// Operator enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UnaryOp {
    Negate, // -
    Not,    // not
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BinaryOp {
    Add,          // +
    Subtract,     // -
    Multiply,     // *
    Divide,       // /
    Rem,          // %
    Concat,       // <>
    Equal,        // ==
    NotEqual,     // !=
    Greater,      // >
    GreaterEqual, // >=
    Less,         // <
    LessEqual,    // <=
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LogicalOp {
    And,
    Or,
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Literal {
        value: Value,
        line: usize,
        ty: Option<Type>,
    },
    Variable {
        name: String,
        line: usize,
        ty: Option<Type>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
        line: usize,
        ty: Option<Type>,
    },
    Binary {
        operator: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        line: usize,
        ty: Option<Type>,
    },
    Logical {
        operator: LogicalOp,
        left: Box<Expr>,
        right: Box<Expr>,
        line: usize,
        ty: Option<Type>,
    },
    Grouping {
        expr: Box<Expr>,
        line: usize,
        ty: Option<Type>,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
        line: usize,
        ty: Option<Type>,
    },
    Assignment {
        name: String,
        value: Box<Expr>,
        line: usize,
        ty: Option<Type>,
    },
    Narrowing {
        expr: Box<Expr>,
        narrowing: String,
        line: usize,
        ty: Option<Type>,
    },
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Expression {
        expr: Expr,
        line: usize,
        ty: Option<Type>,
    },
    Let {
        name: String,
        initializer: Option<Expr>,
        line: usize,
        ty: Option<Type>,
    },
    Fun {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        line: usize,
        ty: Option<Type>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
        line: usize,
        ty: Option<Type>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
        line: usize,
        ty: Option<Type>,
    },
    Block {
        stmts: Vec<Stmt>,
        line: usize,
        ty: Option<Type>,
    },
    Return {
        value: Expr,
        line: usize,
        ty: Option<Type>,
    },
}

// ---------------------------------------------------------------------------
// Program (top-level)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub(crate) stmts: Vec<Stmt>,
}
