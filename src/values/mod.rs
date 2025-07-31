use crate::handle_errors::RuntimeError;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::Stmt;
use crate::environment::Environment;

pub enum EvalResult<'a> {
    Value(RuntimeVal<'a>),
    Return(RuntimeVal<'a>),
    Break,
    Continue,
    NoDisplay,
}

#[derive(Debug, Clone)]
pub enum RuntimeVal<'a> {
    Bool(bool),
    Nil,
    Number(f64),
    String(&'a str),
    Object(HashMap<&'a str, RuntimeVal<'a>>),
    Function {
        name: &'a str,
        params: Vec<&'a str>,
        body: Vec<Stmt<'a>>,
        closure: Rc<RefCell<Environment<'a>>>,
    },
    NativeFunction(fn(Vec<RuntimeVal<'a>>) -> Result<RuntimeVal<'a>, RuntimeError>),
    Method {
        func: Box<RuntimeVal<'a>>,
        instance: Box<RuntimeVal<'a>>,
    },
    Class {
        name: &'a str,
        static_fields: HashMap<&'a str, RuntimeVal<'a>>,
        methods: HashMap<&'a str, RuntimeVal<'a>>,
        superclass: Option<&'a str>,
    },
    Instance {
        class_name: &'a str,
        instance_env: Rc<RefCell<Environment<'a>>>,
    },
}

pub fn make_number<'a>(num: f64) -> RuntimeVal<'a> {
    RuntimeVal::Number(num)
}

pub fn make_bool<'a>(bit: bool) -> RuntimeVal<'a> {
    RuntimeVal::Bool(bit)
}

pub fn make_nil<'a>() -> RuntimeVal<'a> {
    RuntimeVal::Nil
}

pub fn make_string<'a>(str: &'a str) -> RuntimeVal<'a> {
    RuntimeVal::String(str)
}

pub fn make_obj<'a>(map: HashMap<&'a str, RuntimeVal<'a>>) -> RuntimeVal<'a> {
    RuntimeVal::Object(map)
}

pub fn make_function<'a>(
    name: &'a str,
    params: Vec<&'a str>,
    body: Vec<Stmt<'a>>,
    env: &Rc<RefCell<Environment<'a>>>,
) -> RuntimeVal<'a> {
    RuntimeVal::Function {
        name: name,
        params: params,
        body: body,
        closure: Rc::clone(&env),
    }
}

pub fn make_native_function<'a>(
    func: fn(Vec<RuntimeVal>) -> Result<RuntimeVal, RuntimeError>,
) -> RuntimeVal<'a> {
    RuntimeVal::NativeFunction(func)
}

pub fn make_method<'a>(func: RuntimeVal<'a>, instance_var: RuntimeVal<'a>) -> RuntimeVal<'a> {
    RuntimeVal::Method {
        func: Box::new(func),
        instance: Box::new(instance_var),
    }
}

pub fn make_class<'a>(
    name: &'a str,
    static_fields: HashMap<&'a str, RuntimeVal<'a>>,
    methods: HashMap<&'a str, RuntimeVal<'a>>,
    superclass: Option<&'a str>,
) -> RuntimeVal<'a> {
    RuntimeVal::Class {
        name,
        static_fields,
        methods,
        superclass,
    }
}

pub fn make_instance<'a>(name: &'a str, env: Rc<RefCell<Environment<'a>>>) -> RuntimeVal<'a> {
    RuntimeVal::Instance {
        class_name: name,
        instance_env: env,
    }
}

pub fn make_return<'a>(expr_value: RuntimeVal<'a>) -> EvalResult<'a> {
    EvalResult::Return(expr_value)
}

pub fn make_break<'a>() -> EvalResult<'a> {
    EvalResult::Break
}

pub fn make_continue<'a>() -> EvalResult<'a> {
    EvalResult::Continue
}

pub fn make_none<'a>() -> EvalResult<'a> {
    EvalResult::NoDisplay
}
