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
    NumericLiteral(f64),
    Null,
    BoolLiteral(bool),
    StringLiteral(String),
    Identifier(String),
    This,
    Member {
        object: Box<Expr>,
        property: Box<Expr>,
        computed: bool,
    },
    Call {
        args: Vec<Expr>,
        caller: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    BinaryExpr {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    ComparisonLiteral {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    ObjectLiteral {
        properties: Vec<Property>,
    },
    AssignmentExpr {
        assignee: Box<Expr>,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub key: String,
    pub value: Option<Box<Expr>>,
}
