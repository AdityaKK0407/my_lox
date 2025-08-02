use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::environment::*;
use crate::interpreter::expression::*;
use crate::interpreter::statement::*;
use crate::values::*;
use crate::handle_errors::RuntimeError;

pub fn evaluate_program(program: &[Stmt], env: &Rc<RefCell<Environment>>, is_repl: bool) -> Result<(), RuntimeError> {
    for statement in program {
        if let Stmt::Function(function) = statement {
            let func = make_function(
                &function.name[..],
                &function.parameters,
                &function.body,
                env,
            );
            declare_var(env, &function.name[..], func, true)?;
        } else if let Stmt::Class(class) = statement {
            let mut fields = HashMap::new();
            for var in &class.static_fields {
                let res = var_declaration(var, env)?;
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
                    &func.name[..],
                    &func.parameters,
                    &func.body,
                    env,
                );
                methods.insert(name.clone(), res);
            }
            let class_val = make_class(
                &class.name[..],
                fields,
                methods,
                class.superclass.clone(),
            );
            let _ = declare_var(env, &class.name[..], class_val, true);
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
            caller: Box::new(Expr::Identifier(String::from("main"), 0)),
            line: 0,
        }); // Calling main function happens outside the code, thus denoted by line 0. NOT A MISTAKE
        evaluate(&main_stmt, env)?;
    }
    Ok(())
}

pub fn evaluate(ast_node: &Stmt, env: &Rc<RefCell<Environment>>) -> Result<EvalResult, RuntimeError> {
    match ast_node {
        Stmt::Expression(expr) => Ok(EvalResult::Value(evaluate_expr(expr, env)?)),
        Stmt::VarDeclaration(declaration) => var_declaration(declaration, env),
        Stmt::Print(value, new_line) => print_stmt(value, env, *new_line),
        Stmt::IfElse(if_collection) => if_else_stmt(if_collection, env),
        Stmt::While((expr, stmt)) => while_stmt(expr, stmt, env),
        Stmt::For(((var_stmt, expr1, expr2), statement)) => for_stmt(
            var_stmt,
            expr1,
            expr2,
            statement,
            env,
        ),
        Stmt::Block(stmts) => block_stmt(stmts.clone(), env),
        Stmt::Return(expr) => Ok(make_return(evaluate_expr(expr, env)?)),
        Stmt::Break => Ok(make_break()),
        Stmt::Continue => Ok(make_continue()),
        Stmt::Function(FunctionDeclaration {
            name,
            parameters,
            body,
        }) => {
            let function = make_function(name, parameters, body, env);
            let _ = declare_var(env, &name[..], function, true);
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
                let res = var_declaration(var, env)?;
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
                    &func.name[..],
                    &func.parameters,
                    &func.body,
                    env,
                );
                method.insert(name.clone(), res);
            }
            let class_val = make_class(
                &name[..],
                fields,
                method,
                superclass.clone(),
            );
            let _ = declare_var(env, &name[..], class_val, true);
            Ok(make_none())
        }
    }
}
