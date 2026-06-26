use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Fun(Box<Type>, Box<Type>),
}

pub type TypeEnvironment = HashMap<String, Type>;

pub fn empty_type_env() -> TypeEnvironment {
    HashMap::new()
}