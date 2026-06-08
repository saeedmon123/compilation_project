mod miniimp;
mod minifun;

use minifun::ast::{BinOp, Term};
use minifun::eval::eval;
use minifun::runtime::empty_env;

fn main() {
    let program = Term::App(
        Box::new(Term::Fun(
            "x".to_string(),
            Box::new(Term::BinOp(
                Box::new(Term::Var("x".to_string())),
                BinOp::Add,
                Box::new(Term::Int(1)),
            )),
        )),
        Box::new(Term::Int(5)),
    );

    let mut env = empty_env();

    match eval(&program, &mut env) {
        Ok(value) => println!("MiniFun result: {:?}", value),
        Err(error) => println!("{}", error),
    }
}