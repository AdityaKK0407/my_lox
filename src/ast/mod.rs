use std::collections::HashMap;

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    VarDeclaration(VarDeclaration),
    Print(Option<Vec<Expr>>, bool),
    IfElse(Vec<(Expr, Vec<Stmt>)>),
    For(((Box<Stmt>, Expr, Expr), Vec<Stmt>)),
    While((Expr, Vec<Stmt>)),
    Function(FunctionDeclaration),
    Block(Vec<Stmt>),
    Return(Expr),
    Break,
    Continue,
    Class(ClassDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclaration {
    pub constant: bool,
    pub identifier: String,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclaration {
    pub name: String,
    pub static_fields: Vec<VarDeclaration>,
    pub methods: HashMap<String, FunctionDeclaration>,
    pub superclass: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    NumericLiteral(f64, usize),
    Null(usize),
    BoolLiteral(bool, usize),
    StringLiteral(String, usize),
    Identifier(String, usize),
    This(usize),
    Super(usize),
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
        start_line: usize,
        end_line: usize,
    },
    AssignmentExpr {
        assignee: Box<Expr>,
        value: Box<Expr>,
        line: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub key: String,
    pub value: Option<Box<Expr>>,
}
