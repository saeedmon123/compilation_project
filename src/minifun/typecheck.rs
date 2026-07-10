use crate::minifun::ast::{BinOp, Term};
use crate::minifun::types::{Type, TypeEnvironment};

pub fn typecheck(term: &Term, env: &mut TypeEnvironment) -> Result<Type, String> {
    match term {
        Term::Int(_) => Ok(Type::Int),

        Term::Bool(_) => Ok(Type::Bool),

        Term::Var(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Type error: variable '{}' is not defined", name)),

        Term::Fun {
            param,
            param_type,
            body,
        } => {
            let mut new_env = env.clone();
            new_env.insert(param.clone(), param_type.clone());

            let body_type = typecheck(body, &mut new_env)?;

            Ok(Type::Fun(Box::new(param_type.clone()), Box::new(body_type)))
        }

        Term::App(function_term, argument_term) => {
            let function_type = typecheck(function_term, env)?;
            let argument_type = typecheck(argument_term, env)?;

            match function_type {
                Type::Fun(input_type, output_type) => {
                    if *input_type == argument_type {
                        Ok(*output_type)
                    } else {
                        Err(format!(
                            "Type error: function expected argument of type {:?}, but got {:?}",
                            input_type, argument_type
                        ))
                    }
                }

                other => Err(format!(
                    "Type error: attempted to apply a non-function value of type {:?}",
                    other
                )),
            }
        }

        Term::BinOp(left, op, right) => {
            let left_type = typecheck(left, env)?;
            let right_type = typecheck(right, env)?;

            typecheck_binop(op, left_type, right_type)
        }

        Term::Not(value) => {
            let value_type = typecheck(value, env)?;

            if value_type == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(format!(
                    "Type error: not expects Bool, but got {:?}",
                    value_type
                ))
            }
        }

        Term::If(condition, then_branch, else_branch) => {
            let condition_type = typecheck(condition, env)?;

            if condition_type != Type::Bool {
                return Err(format!(
                    "Type error: if condition must be Bool, but got {:?}",
                    condition_type
                ));
            }

            let then_type = typecheck(then_branch, env)?;
            let else_type = typecheck(else_branch, env)?;

            if then_type == else_type {
                Ok(then_type)
            } else {
                Err(format!(
                    "Type error: if branches have different types: {:?} and {:?}",
                    then_type, else_type
                ))
            }
        }

        Term::Let(name, value_term, body_term) => {
            let value_type = typecheck(value_term, env)?;

            let mut new_env = env.clone();
            new_env.insert(name.clone(), value_type);

            typecheck(body_term, &mut new_env)
        }

        Term::LetFun {
            name,
            param,
            param_type,
            return_type,
            body,
            in_term,
        } => {
            let function_type =
                Type::Fun(Box::new(param_type.clone()), Box::new(return_type.clone()));

            let mut function_env = env.clone();
            function_env.insert(name.clone(), function_type.clone());
            function_env.insert(param.clone(), param_type.clone());

            let body_type = typecheck(body, &mut function_env)?;

            if body_type != *return_type {
                return Err(format!(
                    "Type error: recursive function '{}' declared return type {:?}, but body has type {:?}",
                    name, return_type, body_type
                ));
            }

            let mut new_env = env.clone();
            new_env.insert(name.clone(), function_type);

            typecheck(in_term, &mut new_env)
        }
    }
}

fn typecheck_binop(op: &BinOp, left: Type, right: Type) -> Result<Type, String> {
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul => {
            if left == Type::Int && right == Type::Int {
                Ok(Type::Int)
            } else {
                Err(format!(
                    "Type error: arithmetic operator expects Int and Int, but got {:?} and {:?}",
                    left, right
                ))
            }
        }

        BinOp::And => {
            if left == Type::Bool && right == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(format!(
                    "Type error: and expects Bool and Bool, but got {:?} and {:?}",
                    left, right
                ))
            }
        }

        BinOp::Less => {
            if left == Type::Int && right == Type::Int {
                Ok(Type::Bool)
            } else {
                Err(format!(
                    "Type error: < expects Int and Int, but got {:?} and {:?}",
                    left, right
                ))
            }
        }
    }
}
