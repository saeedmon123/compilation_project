use std::collections::HashMap;

use crate::minifun::ast::Term;

pub type Environment = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),

    Closure {
        param: String,
        body: Box<Term>,
        env: Environment,
    },

    RecursiveClosure {
        name: String,
        param: String,
        body: Box<Term>,
        env: Environment,
    },
}

pub fn empty_env() -> Environment {
    HashMap::new()
}
