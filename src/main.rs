mod miniimp;

use miniimp::ast::{AExpr, BExpr, Command, Program};
use miniimp::eval::eval_program;

fn main() {
    let factorial_program = Program {
        input: "in".to_string(),
        output: "out".to_string(),
        body: Command::Seq(
            Box::new(Command::Assign(
                "out".to_string(),
                AExpr::Int(1),
            )),
            Box::new(Command::Seq(
                Box::new(Command::Assign(
                    "i".to_string(),
                    AExpr::Int(1),
                )),
                Box::new(Command::While(
                    BExpr::Less(
                        Box::new(AExpr::Var("i".to_string())),
                        Box::new(AExpr::Add(
                            Box::new(AExpr::Var("in".to_string())),
                            Box::new(AExpr::Int(1)),
                        )),
                    ),
                    Box::new(Command::Seq(
                        Box::new(Command::Assign(
                            "out".to_string(),
                            AExpr::Mul(
                                Box::new(AExpr::Var("out".to_string())),
                                Box::new(AExpr::Var("i".to_string())),
                            ),
                        )),
                        Box::new(Command::Assign(
                            "i".to_string(),
                            AExpr::Add(
                                Box::new(AExpr::Var("i".to_string())),
                                Box::new(AExpr::Int(1)),
                            ),
                        )),
                    )),
                )),
            )),
        ),
    };

    match eval_program(&factorial_program, 3) {
        Ok(result) => println!("Program result: {}", result),
        Err(error) => println!("{}", error),
    }
}