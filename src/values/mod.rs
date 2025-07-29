use std::{cell::RefCell, collections::HashMap, rc::Rc};
use crate::handle_errors::RuntimeError;

use crate::{
    ast::Stmt,
    environment::Environment,
};

pub enum EvalResult {
    Value(RuntimeVal),
    Return(RuntimeVal),
    Break,
    Continue,
    NoDisplay,
}

#[derive(Debug, Clone)]
pub enum RuntimeVal {
    Bool(bool),
    Nil,
    Number(f64),
    String(String),
    Object(HashMap<String, RuntimeVal>),
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    },
    NativeFunction(fn(Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError>),
    Method {
        func: Box<RuntimeVal>,
        instance: Box<RuntimeVal>,
    },
    Class {
        name: String,
        static_fields: HashMap<String, RuntimeVal>,
        methods: HashMap<String, RuntimeVal>,
        superclass: Option<String>,
    },
    Instance {
        class_name: String,
        instance_env: Rc<RefCell<Environment>>,
    }
}

pub fn make_number(num: f64) -> RuntimeVal {
    RuntimeVal::Number(num)
}

pub fn make_bool(bit: bool) -> RuntimeVal {
    RuntimeVal::Bool(bit)
}

pub fn make_nil() -> RuntimeVal {
    RuntimeVal::Nil
}

pub fn make_string(str: String) -> RuntimeVal {
    RuntimeVal::String(str)
}

pub fn make_obj(map: HashMap<String, RuntimeVal>) -> RuntimeVal {
    RuntimeVal::Object(map)
}

pub fn make_function(
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
    env: &Rc<RefCell<Environment>>,
) -> RuntimeVal {
    RuntimeVal::Function {
        name: name,
        params: params,
        body: body,
        closure: env.clone(),
    }
}

pub fn make_native_function(func: fn(Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError>) -> RuntimeVal {
    RuntimeVal::NativeFunction(func)
}

pub fn make_method(func: RuntimeVal, instance_var: RuntimeVal) -> RuntimeVal {
    RuntimeVal::Method { func: Box::new(func), instance: Box::new(instance_var) }
}

pub fn make_class(
    name: String,
    static_fields: HashMap<String, RuntimeVal>,
    methods: HashMap<String, RuntimeVal>,
    superclass: Option<String>,
) -> RuntimeVal {
    RuntimeVal::Class {
        name,
        static_fields,
        methods,
        superclass,
    }
}

pub fn make_instance(name: String, env: Rc<RefCell<Environment>>) -> RuntimeVal {
    RuntimeVal::Instance { class_name: name, instance_env: env }
}

pub fn make_return(expr_value: RuntimeVal) -> EvalResult {
    EvalResult::Return(expr_value)
}

pub fn make_break() -> EvalResult {
    EvalResult::Break
}

pub fn make_continue() -> EvalResult {
    EvalResult::Continue
}

pub fn make_none() -> EvalResult {
    EvalResult::NoDisplay
}
