use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::environment::*;
use crate::interpreter::expression::*;
use crate::interpreter::statement::*;
use crate::values::*;
use crate::handle_errors::RuntimeError;

pub fn evaluate_program(program: Vec<Stmt>, env: &Rc<RefCell<Environment>>, is_repl: bool) -> Result<(), RuntimeError> {
    for statement in &program {
        if let Stmt::Function(function) = statement {
            let func = make_function(
                function.name.clone(),
                function.parameters.clone(),
                function.body.clone(),
                env,
            );
            declare_var(env, function.name.clone(), func, true)?;
        } else if let Stmt::Class(class) = statement {
            let mut fields = HashMap::new();
            for var in &class.static_fields {
                let res = var_declaration(var.clone(), env)?;
                match res {
                    EvalResult::Value(value) => {
                        fields.insert(var.identifier.clone(), value);
                    }
                    _ => panic!(),
                }
            }
            let mut methods = HashMap::new();
            for (name, func) in &class.methods {
                let res = make_function(
                    func.name.clone(),
                    func.parameters.clone(),
                    func.body.clone(),
                    env,
                );
                methods.insert(name.clone(), res);
            }
            let class_val = make_class(
                class.name.clone(),
                fields,
                methods,
                class.superclass.clone(),
            );
            let _ = declare_var(env, class.name.clone(), class_val, true);
        } else if !is_repl && matches!(statement, Stmt::VarDeclaration(_)) {
            evaluate(statement, env)?;
        } else if !is_repl {
            panic!("Invalid declaration in global scope. Only allowed inside functions");
        }
    }
    if is_repl {
        for statement in program {
            let value = evaluate(&statement, env)?;
            if let EvalResult::Value(val) = value {
                print_runtime_val(val);
                println!();
            }
        }
    } else {
        let main_stmt = Stmt::Expression(Expr::Call {
            args: vec![],
            caller: Box::new(Expr::Identifier(String::from("main"))),
        });
        evaluate(&main_stmt, env)?;
    }
    Ok(())
}

pub fn evaluate(ast_node: &Stmt, env: &Rc<RefCell<Environment>>) -> Result<EvalResult, RuntimeError> {
    match ast_node {
        Stmt::Expression(expr) => Ok(EvalResult::Value(evaluate_expr(expr.clone(), env)?)),
        Stmt::VarDeclaration(declaration) => var_declaration(declaration.clone(), env),
        Stmt::Print(value, new_line) => print_stmt(value.clone(), env, *new_line),
        Stmt::IfElse(if_collection) => if_else_stmt(if_collection.clone(), env),
        Stmt::While((expr, stmt)) => while_stmt(expr.clone(), stmt.clone(), env),
        Stmt::For(((var_stmt, expr1, expr2), statement)) => for_stmt(
            *var_stmt.clone(),
            expr1.clone(),
            expr2.clone(),
            statement.clone(),
            env,
        ),
        Stmt::Block(stmts) => block_stmt(stmts.clone(), env),
        Stmt::Return(expr) => Ok(make_return(evaluate_expr(expr.clone(), env)?)),
        Stmt::Break => Ok(make_break()),
        Stmt::Continue => Ok(make_continue()),
        Stmt::Function(FunctionDeclaration {
            name,
            parameters,
            body,
        }) => {
            let function = make_function(name.clone(), parameters.clone(), body.clone(), env);
            let _ = declare_var(env, name.clone(), function, true);
            Ok(make_none())
        }
        Stmt::Class(ClassDeclaration {
            name,
            static_fields,
            methods,
            superclass,
        }) => {
            let mut fields = HashMap::new();
            for var in static_fields {
                let res = var_declaration(var.clone(), env)?;
                match res {
                    EvalResult::Value(value) => {
                        fields.insert(var.identifier.clone(), value);
                    }
                    _ => panic!(),
                }
            }
            let mut method = HashMap::new();
            for (name, func) in methods {
                let res = make_function(
                    func.name.clone(),
                    func.parameters.clone(),
                    func.body.clone(),
                    env,
                );
                method.insert(name.clone(), res);
            }
            let class_val = make_class(
                name.clone(),
                fields,
                method,
                superclass.clone(),
            );
            let _ = declare_var(env, name.clone(), class_val, true);
            Ok(make_none())
        }
        // _ => panic!(),
    }
}
