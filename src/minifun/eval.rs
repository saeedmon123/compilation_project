use crate::minifun::ast::{BinOp, Term};
use crate::minifun::runtime::{Environment, Value};

pub fn eval(term: &Term, env: &mut Environment) -> Result<Value, String> {
    match term {
        Term::Int(n) => Ok(Value::Int(*n)),

        Term::Bool(b) => Ok(Value::Bool(*b)),

        Term::Var(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Runtime error: variable '{}' is undefined", name)),

        Term::Fun {
            param,
            param_type: _,
            body,
        } => Ok(Value::Closure {
            param: param.clone(),
            body: body.clone(),
            env: env.clone(),
        }),

        Term::App(function_term, argument_term) => {
            let function_value = eval(function_term, env)?;
            let argument_value = eval(argument_term, env)?;

            match function_value {
                Value::Closure {
                    param,
                    body,
                    mut env,
                } => {
                    env.insert(param, argument_value);
                    eval(&body, &mut env)
                }

                Value::RecursiveClosure {
                    name,
                    param,
                    body,
                    mut env,
                } => {
                    let recursive_closure = Value::RecursiveClosure {
                        name: name.clone(),
                        param: param.clone(),
                        body: body.clone(),
                        env: env.clone(),
                    };

                    env.insert(name, recursive_closure);
                    env.insert(param, argument_value);

                    eval(&body, &mut env)
                }

                _ => Err("Runtime error: attempted to apply a non-function value".to_string()),
            }
        }

        Term::BinOp(left, op, right) => {
            let left_value = eval(left, env)?;
            let right_value = eval(right, env)?;

            eval_binop(op, left_value, right_value)
        }

        Term::Not(value) => {
            let value = eval(value, env)?;

            match value {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err("Runtime error: not operator expects a boolean".to_string()),
            }
        }

        Term::If(condition, then_branch, else_branch) => {
            let condition_value = eval(condition, env)?;

            match condition_value {
                Value::Bool(true) => eval(then_branch, env),
                Value::Bool(false) => eval(else_branch, env),
                _ => Err("Runtime error: if condition must be a boolean".to_string()),
            }
        }

        Term::Let(name, value_term, body_term) => {
            let value = eval(value_term, env)?;

            let mut new_env = env.clone();
            new_env.insert(name.clone(), value);

            eval(body_term, &mut new_env)
        }

        Term::LetFun {
            name,
            param,
            param_type: _,
            return_type: _,
            body,
            in_term,
        } => {
            let recursive_closure = Value::RecursiveClosure {
                name: name.clone(),
                param: param.clone(),
                body: body.clone(),
                env: env.clone(),
            };

            let mut new_env = env.clone();
            new_env.insert(name.clone(), recursive_closure);

            eval(in_term, &mut new_env)
        }
    }
}

fn eval_binop(op: &BinOp, left: Value, right: Value) -> Result<Value, String> {
    match (op, left, right) {
        (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),

        (BinOp::And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),

        (BinOp::Less, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),

        _ => Err("Runtime error: invalid operands for binary operator".to_string()),
    }
}
