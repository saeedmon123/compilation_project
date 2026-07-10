mod minifun;
mod miniimp;

use miniimp::ast::{AExpr, BExpr, Command, Program};

use miniimp::cfg::program_to_cfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
        MiniImp test program:

        y := 0;

        while y < x do
            if y < 5 then
                y := y + 1
            else
                y := y + 2
    */

    let program = Program {
        input: "x".to_string(),
        output: "y".to_string(),

        body: Command::Seq(
            Box::new(Command::Assign("y".to_string(), AExpr::Int(0))),
            Box::new(Command::While(
                BExpr::Less(
                    Box::new(AExpr::Var("y".to_string())),
                    Box::new(AExpr::Var("x".to_string())),
                ),
                Box::new(Command::If(
                    BExpr::Less(
                        Box::new(AExpr::Var("y".to_string())),
                        Box::new(AExpr::Int(5)),
                    ),
                    Box::new(Command::Assign(
                        "y".to_string(),
                        AExpr::Add(
                            Box::new(AExpr::Var("y".to_string())),
                            Box::new(AExpr::Int(1)),
                        ),
                    )),
                    Box::new(Command::Assign(
                        "y".to_string(),
                        AExpr::Add(
                            Box::new(AExpr::Var("y".to_string())),
                            Box::new(AExpr::Int(2)),
                        ),
                    )),
                )),
            )),
        ),
    };

    let cfg = program_to_cfg(&program);

    println!("{}", cfg);

    std::fs::write("cfg.dot", cfg.to_dot())?;

    println!("The CFG was written to cfg.dot");

    Ok(())
}
