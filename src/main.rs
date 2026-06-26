mod miniimp;
mod minifun;

use minifun::ast::{BinOp, Term};
use minifun::eval::eval;
use minifun::runtime::empty_env;
use minifun::typecheck::typecheck;
use minifun::types::{empty_type_env, Type};

fn main() {
    let program = Term::App(
        Box::new(Term::Fun {
            param: "x".to_string(),
            param_type: Type::Int,
            body: Box::new(Term::BinOp(
                Box::new(Term::Var("x".to_string())),
                BinOp::Add,
                Box::new(Term::Int(1)),
            )),
        }),
        Box::new(Term::Bool(true))
    );

    let mut type_env = empty_type_env();

    match typecheck(&program, &mut type_env) {
        Ok(program_type) => {
            println!("Typecheck result: {:?}", program_type);

            let mut runtime_env = empty_env();

            match eval(&program, &mut runtime_env) {
                Ok(value) => println!("Evaluation result: {:?}", value),
                Err(error) => println!("{}", error),
            }
        }

        Err(error) => println!("{}", error),
    }
}