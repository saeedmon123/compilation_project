mod miniimp;
mod minifun;

use minifun::ast::{BinOp, Term};
use minifun::inference::typecheck;
use minifun::types::Type;

fn main() {
    let term = Term::Let(
        "id".to_string(),
        Box::new(Term::Fun {
            param: "x".to_string(),
            param_type: Type::Int,
            body: Box::new(Term::Var("x".to_string())),
        }),
        Box::new(Term::App(
            Box::new(Term::Var("id".to_string())),
            Box::new(Term::Int(5)),
        )),
    );

    match typecheck(&term) {
        Ok(ty) => println!("Type inferred successfully: {:?}", ty),
        Err(e) => println!("Type error: {}", e),
    }

    let bad_term = Term::BinOp(
        Box::new(Term::Bool(true)),
        BinOp::Add,
        Box::new(Term::Int(3)),
    );

    match typecheck(&bad_term) {
        Ok(ty) => println!("Type inferred successfully: {:?}", ty),
        Err(e) => println!("Type error: {}", e),
    }
}