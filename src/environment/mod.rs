use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::global_scope::*;
use crate::handle_errors::EnvironmentError;
use crate::values::RuntimeVal;
use crate::values::make_native_function;

#[derive(PartialEq)]
pub enum Scope {
    Global,
    Class(String),
    Method(String),
    Constructor(String),
    Function(String),
    Loop,
    VarDeclaration,
}

pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    pub variables: HashMap<String, RuntimeVal>,
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
    let _ = declare_var(env, "clock", make_native_function(clock, "clock"), true);
    let _ = declare_var(env, "scan", make_native_function(scan, "scan"), true);
    let _ = declare_var(env, "min", make_native_function(min, "min"), true);
    let _ = declare_var(env, "max", make_native_function(max, "max"), true);
    let _ = declare_var(env, "number", make_native_function(number, "number"), true);
    let _ = declare_var(env, "bool", make_native_function(bool, "bool"), true);
    let _ = declare_var(env, "string", make_native_function(string, "string"), true);
    let _ = declare_var(env, "len", make_native_function(len, "len"), true);
    let _ = declare_var(env, "type_of", make_native_function(type_of, "type_of"), true);
    let _ = declare_var(env, "reverse", make_native_function(reverse, "reverse"), true);
}

pub fn declare_var(
    env: &Rc<RefCell<Environment>>,
    var_name: &str,
    value: RuntimeVal,
    constant: bool,
) -> Result<RuntimeVal, EnvironmentError> {
    let mut env = env.borrow_mut();
    if env.variables.contains_key(var_name) {
        return Err(EnvironmentError::ReDeclareVar);
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
) -> Result<RuntimeVal, EnvironmentError> {
    let final_env = resolve(env, var_name)?;
    let mut env = final_env.borrow_mut();

    if env.constants.contains(var_name) {
        return Err(EnvironmentError::ConstReassign);
    }
    env.variables.insert(var_name.to_string(), value.clone());
    Ok(value)
}

pub fn lookup_var(
    env: &Rc<RefCell<Environment>>,
    var_name: &str,
) -> Result<RuntimeVal, EnvironmentError> {
    let final_env = resolve(env, var_name)?;
    let env = final_env.borrow();
    Ok(env.variables.get(var_name).unwrap().clone())
}

pub fn resolve(
    env: &Rc<RefCell<Environment>>,
    var_name: &str,
) -> Result<Rc<RefCell<Environment>>, EnvironmentError> {
    if env.borrow().variables.contains_key(var_name) {
        return Ok(Rc::clone(env));
    }
    match &env.borrow().parent {
        Some(parent) => resolve(parent, var_name),
        None => Err(EnvironmentError::VarNotDeclared),
    }
}
