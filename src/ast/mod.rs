use std::collections::HashMap;

use crate::lexer::Token;

#[derive(Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    VarDeclaration(VarDeclaration),
    Print(Option<Vec<Expr>>, bool),
    IfElse(Vec<(Expr, Vec<Stmt>, usize)>),
    For((Box<Stmt>, Expr, Expr), Vec<Stmt>, usize),
    While(Expr, Vec<Stmt>, usize),
    Block(Vec<Stmt>),
    Return(Expr),
    Break,
    Continue,
    Function(FunctionDeclaration),
    Class(ClassDeclaration),
}

#[derive(Clone, PartialEq)]
pub struct VarDeclaration {
    pub constant: bool,
    pub identifier: String,
    pub value: Box<Expr>,
    pub line: usize,
}

#[derive(Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Stmt>,
    pub line: usize,
}

#[derive(Clone, PartialEq)]
pub struct ClassDeclaration {
    pub name: String,
    pub static_fields: Vec<VarDeclaration>,
    pub methods: HashMap<String, FunctionDeclaration>,
    pub superclass: Option<String>,
    pub line: usize,
}

#[derive(Clone, PartialEq)]
pub enum Expr {
    NumericLiteral(f64, usize),
    Null(usize),
    BoolLiteral(bool, usize),
    StringLiteral(String, usize),
    Identifier(String, usize),
    This(usize),
    Super(String, usize),
    Array(Vec<Expr>, usize),
    Member {
        object: Box<Expr>,
        property: Box<Expr>,
        computed: bool,
        line: usize,
    },
    Call {
        args: Vec<Expr>,
        caller: Box<Expr>,
        line: usize,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
        line: usize,
    },
    BinaryExpr {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
        line: usize,
    },
    ComparisonLiteral {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
        line: usize,
    },
    ObjectLiteral {
        properties: Vec<Property>,
    },
    AssignmentExpr {
        assignee: Box<Expr>,
        value: Box<Expr>,
        line: usize,
    },
}

#[derive(Clone, PartialEq)]
pub struct Property {
    pub key: String,
    pub value: Option<Box<Expr>>,
    pub line: usize,
}
