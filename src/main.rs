mod minifun;
mod miniimp;

use miniimp::ast::{AExpr, BExpr, Command, Program};

use miniimp::cfg::program_to_cfg;

use miniimp::dataflow::{
    defined_variables_analysis, live_variables_analysis, reaching_definitions_analysis,
};

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

    let defined = defined_variables_analysis(&cfg);

    let live = live_variables_analysis(&cfg);

    let reaching = reaching_definitions_analysis(&cfg);

    println!("Original CFG:\n{}", cfg);

    println!("\nDefined variables analysis:\n{}", defined);

    println!("\nLive variables analysis:\n{}", live);

    println!("\nReaching definitions analysis:\n{}", reaching);

    std::fs::write("cfg.dot", cfg.to_dot())?;

    std::fs::write("cfg_defined.dot", defined.to_dot())?;

    std::fs::write("cfg_live.dot", live.to_dot())?;

    std::fs::write("cfg_reaching.dot", reaching.to_dot())?;

    println!("\nDOT files written:");
    println!("- cfg.dot");
    println!("- cfg_defined.dot");
    println!("- cfg_live.dot");
    println!("- cfg_reaching.dot");

    Ok(())
}
