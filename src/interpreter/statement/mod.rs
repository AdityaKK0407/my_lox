use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::environment::*;
use crate::interpreter::expression::*;
use crate::interpreter::interpreter::*;
use crate::values::*;
use crate::handle_errors::RuntimeError;

pub fn var_declaration<'a>(declaration: VarDeclaration<'a>, env: &Rc<RefCell<Environment<'a>>>) -> Result<EvalResult<'a>, RuntimeError> {
    let value = evaluate_expr(*declaration.value, env)?;
    declare_var(env, declaration.identifier, value, declaration.constant)?;
    Ok(make_none())
}

pub fn print_stmt<'a>(
    value: Option<Vec<Expr<'a>>>,
    env: &Rc<RefCell<Environment<'a>>>,
    new_line: bool,
) -> Result<EvalResult<'a>, RuntimeError> {
    if let Some(exprs) = value {
        for expr in exprs {
            let runtime_val = evaluate_expr(expr, env)?;
            print_runtime_val(runtime_val);
        }
    }
    if new_line {
        println!();
    }
    Ok(make_none())
}

pub fn print_runtime_val(runtime_val: RuntimeVal) {
    match runtime_val {
        RuntimeVal::Number(num) => print!("{}", num),
        RuntimeVal::Bool(bit) => print!("{}", bit),
        RuntimeVal::Nil => print!("nil"),
        RuntimeVal::String(s) => print!("{}", s),
        RuntimeVal::Object(obj) => print_obj(obj),
        _ => {}
    }
}

fn print_obj<'a>(obj: HashMap<&'a str, RuntimeVal<'a>>) {
    println!("{{");
    for (key, value) in obj.iter() {
        print!("    \"{}\": ", key);
        print_runtime_val(value.clone());
        println!(",");
    }
    println!("}}");
}

pub fn if_else_stmt<'a>(
    collection: Vec<(Expr<'a>, Vec<Stmt<'a>>)>,
    env: &Rc<RefCell<Environment<'a>>>,
) -> Result<EvalResult<'a>, RuntimeError> {
    let local_env = Environment::new(Some(Rc::clone(env)));
    for (expr, statements) in collection {
        let condition = evaluate_expr(expr, &local_env)?;
        if let RuntimeVal::Bool(bit) = condition {
            if !bit {
                continue;
            } else {
                for statement in statements {
                    match evaluate(&statement, &local_env)? {
                        EvalResult::Return(val) => return Ok(EvalResult::Return(val)),
                EvalResult::Break => return Ok(EvalResult::Break),
                EvalResult::Continue => return Ok(EvalResult::Continue),
                        _ => continue,
                    }
                }
                break;
            }
        }
        panic!("Invalid expression with if. Only booleans are allowed");
    }
    Ok(make_none())
}

pub fn for_stmt<'a>(
    stmt: Stmt<'a>,
    expr1: Expr<'a>,
    expr2: Expr<'a>,
    statements: Vec<Stmt<'a>>,
    env: &Rc<RefCell<Environment<'a>>>,
) -> Result<EvalResult<'a>, RuntimeError> {
    let local_env = Environment::new(Some(Rc::clone(env)));
    evaluate(&stmt, &local_env)?;

    for_loop(expr1, expr2, statements, &local_env)
}

fn for_loop<'a>(
    expr1: Expr<'a>,
    expr2: Expr<'a>,
    statements: Vec<Stmt<'a>>,
    local_env: &Rc<RefCell<Environment<'a>>>,
) -> Result<EvalResult<'a>, RuntimeError> {
    while let RuntimeVal::Bool(bit) = evaluate_expr(expr1.clone(), local_env)? {
        if !bit {
            break;
        }
        for statement in &statements {
            match evaluate(&statement, &local_env)? {
                EvalResult::Return(val) => return Ok(EvalResult::Return(val)),
                EvalResult::Break => return Ok(make_none()),
                EvalResult::Continue => break,
                _ => continue,
            }
        }
        evaluate(&Stmt::Expression(expr2.clone()), local_env)?;
    }

    Ok(make_none())
}

pub fn while_stmt<'a>(expr: Expr<'a>, statements: Vec<Stmt<'a>>, env: &Rc<RefCell<Environment<'a>>>) -> Result<EvalResult<'a>, RuntimeError> {
    let local_env = Environment::new(Some(Rc::clone(env)));
    while let RuntimeVal::Bool(bit) = evaluate_expr(expr.clone(), &local_env)? {
        if !bit {
            break;
        }
        for statement in &statements {
            match evaluate(&statement, &local_env)? {
                EvalResult::Return(val) => return Ok(EvalResult::Return(val)),
                EvalResult::Break => return Ok(make_none()),
                EvalResult::Continue => break,
                _ => continue,
            }
        }
    }

    Ok(make_none())
}

pub fn block_stmt<'a>(stmts: Vec<Stmt<'a>>, env: &Rc<RefCell<Environment<'a>>>) -> Result<EvalResult<'a>, RuntimeError> {
    let local_env = Environment::new(Some(Rc::clone(env)));
    for stmt in stmts {
        match evaluate(&stmt, &local_env)? {
            EvalResult::Return(val) => return Ok(EvalResult::Return(val)),
                EvalResult::Break => return Ok(EvalResult::Break),
                EvalResult::Continue => return Ok(EvalResult::Continue),
            _ => continue,
        }
    }
    Ok(make_none())
}
