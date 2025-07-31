use std::collections::HashMap;

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'a> {
    Expression(Expr<'a>),
    VarDeclaration(VarDeclaration<'a>),
    Print(Option<Vec<Expr<'a>>>, bool),
    IfElse(Vec<(Expr<'a>, Vec<Stmt<'a>>)>),
    For(((Box<Stmt<'a>>, Expr<'a>, Expr<'a>), Vec<Stmt<'a>>)),
    While((Expr<'a>, Vec<Stmt<'a>>)),
    Function(FunctionDeclaration<'a>),
    Block(Vec<Stmt<'a>>),
    Return(Expr<'a>),
    Break,
    Continue,
    Class(ClassDeclaration<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclaration<'a> {
    pub constant: bool,
    pub identifier: &'a str,
    pub value: Box<Expr<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration<'a> {
    pub name: &'a str,
    pub parameters: Vec<&'a str>,
    pub body: Vec<Stmt<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclaration<'a> {
    pub name: &'a str,
    pub static_fields: Vec<VarDeclaration<'a>>,
    pub methods: HashMap<&'a str, FunctionDeclaration<'a>>,
    pub superclass: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    NumericLiteral(f64),
    Null,
    BoolLiteral(bool),
    StringLiteral(&'a str),
    Identifier(&'a str),
    This,
    Member {
        object: Box<Expr<'a>>,
        property: Box<Expr<'a>>,
        computed: bool,
    },
    Call {
        args: Vec<Expr<'a>>,
        caller: Box<Expr<'a>>,
    },
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    BinaryExpr {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    ComparisonLiteral {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    ObjectLiteral {
        properties: Vec<Property<'a>>,
    },
    AssignmentExpr {
        assignee: Box<Expr<'a>>,
        value: Box<Expr<'a>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property<'a> {
    pub key: &'a str,
    pub value: Option<Box<Expr<'a>>>,
}
