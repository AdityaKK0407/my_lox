use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::global_scope::*;
use crate::handle_errors::RuntimeError;
use crate::values::RuntimeVal;
use crate::values::make_native_function;

#[derive(Debug, PartialEq)]
pub enum Scope {
    Global,
    Class(String),
    Method(String),
    Constructor(String),
    Function(String),
    Loop,
}

#[derive(Debug)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    variables: HashMap<String, RuntimeVal>,
    constants: HashSet<String>,
}

impl Environment {
    pub fn new(parent_env: Option<Rc<RefCell<Environment>>>) -> Rc<RefCell<Self>> {
        let env = Rc::new(RefCell::new(Environment {
            parent: parent_env,
            variables: HashMap::new(),
            constants: HashSet::new(),
        }));
        set_global_scope(&env);
        env
    }
}

pub fn set_global_scope(env: &Rc<RefCell<Environment>>) {
    let _ = declare_var(
        env,
        "clock",
        make_native_function(clock),
        true,
    );
    let _ = declare_var(env, "min", make_native_function(min), true);
    let _ = declare_var(env, "max", make_native_function(max), true);
    let _ = declare_var(
        env,
        "number",
        make_native_function(number),
        true,
    );
    let _ = declare_var(env, "bool", make_native_function(bool), true);
    let _ = declare_var(
        env,
        "string",
        make_native_function(string),
        true,
    );
    let _ = declare_var(
        env,
        "var_type",
        make_native_function(var_type),
        true,
    );

    let _ = declare_var(
        env,
        "reverse",
        make_native_function(reverse),
        true,
    );
}

pub fn declare_var(
    env: &Rc<RefCell<Environment>>,
    var_name: &str,
    value: RuntimeVal,
    constant: bool,
) -> Result<RuntimeVal, RuntimeError> {
    let mut env = env.borrow_mut();
    if env.variables.contains_key(var_name) {
        return Err(RuntimeError::DeclareVar);
    }
    env.variables.insert(var_name.to_string(), value.clone());
    if constant {
        env.constants.insert(var_name.to_string());
    }
    Ok(value)
}

pub fn assign_var(
    env: &Rc<RefCell<Environment>>,
    var_name: &str,
    value: RuntimeVal,
) -> Result<RuntimeVal, RuntimeError> {
    let final_env = resolve(env, var_name)?;
    let mut env = final_env.borrow_mut();

    if env.constants.contains(var_name) {
        return Err(RuntimeError::ConstReassign);
    }
    env.variables.insert(var_name.to_string(), value.clone());
    Ok(value)
}

pub fn lookup_var(env: &Rc<RefCell<Environment>>, var_name: &str) -> Result<RuntimeVal, RuntimeError> {
    let final_env = resolve(env, var_name)?;
    let env = final_env.borrow();
    match env.variables.get(var_name) {
        Some(v) => Ok(v.clone()),
        None => Err(RuntimeError::UnidentifiedVar),
    }
}

pub fn resolve(
    env: &Rc<RefCell<Environment>>,
    var_name: &str,
) -> Result<Rc<RefCell<Environment>>, RuntimeError> {
    if env.borrow().variables.contains_key(var_name) {
        return Ok(Rc::clone(env));
    }
    match &env.borrow().parent {
        Some(parent) => resolve(parent, var_name),
        None => Err(RuntimeError::UnidentifiedVar),
    }
}
