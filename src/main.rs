mod minifun;
mod miniimp;

use miniimp::ast::{AExpr, Command, Program};
use miniimp::cfg::program_to_cfg;
use miniimp::dataflow::{
    defined_variables_analysis, live_variables_analysis, reaching_definitions_analysis,
};
use miniimp::optimizations::{OptimizationPipeline, check_undefined_variables};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
        MiniImp program used to demonstrate Fragment 7:

        a := 2 + 3;
        b := a * 4;
        unused := 100;
        y := b + x;

        x is the program input.
        y is the program output.

        Expected optimizations:

        1. Constant folding:
           a := 2 + 3
           becomes
           a := 5

        2. Constant propagation and folding:
           b := a * 4
           becomes
           b := 20

        3. Constant propagation:
           y := b + x
           becomes
           y := 20 + x

        4. Dead-store elimination:
           assignments to a, b, and unused become skip
           after their values are no longer needed.
    */

    let program = Program {
        input: "x".to_string(),
        output: "y".to_string(),

        body: Command::Seq(
            Box::new(Command::Assign(
                "a".to_string(),
                AExpr::Add(Box::new(AExpr::Int(2)), Box::new(AExpr::Int(3))),
            )),
            Box::new(Command::Seq(
                Box::new(Command::Assign(
                    "b".to_string(),
                    AExpr::Mul(
                        Box::new(AExpr::Var("a".to_string())),
                        Box::new(AExpr::Int(4)),
                    ),
                )),
                Box::new(Command::Seq(
                    Box::new(Command::Assign("unused".to_string(), AExpr::Int(100))),
                    Box::new(Command::Assign(
                        "y".to_string(),
                        AExpr::Add(
                            Box::new(AExpr::Var("b".to_string())),
                            Box::new(AExpr::Var("x".to_string())),
                        ),
                    )),
                )),
            )),
        ),
    };

    /*
     * Create the original CFG.
     */

    let cfg = program_to_cfg(&program);

    println!("================ ORIGINAL CFG ================\n");
    println!("{}", cfg);

    /*
     * Fragment 6 analyses.
     */

    let defined = defined_variables_analysis(&cfg);
    let live = live_variables_analysis(&cfg);
    let reaching = reaching_definitions_analysis(&cfg);

    println!("\n============= DEFINED VARIABLES =============\n");
    println!("{}", defined);

    println!("\n=============== LIVE VARIABLES ==============\n");
    println!("{}", live);

    println!("\n============ REACHING DEFINITIONS ===========\n");
    println!("{}", reaching);

    /*
     * Fragment 7:
     * Check for possibly undefined variables.
     */

    println!("\n========== UNDEFINED-VARIABLE CHECK =========\n");

    match check_undefined_variables(&cfg) {
        Ok(()) => {
            println!("No possibly undefined variables were found.");
        }

        Err(errors) => {
            println!("Possibly undefined variables were found:");

            for error in errors {
                println!("- {}", error);
            }
        }
    }

    /*
     * Fragment 7:
     * Run the optimization pipeline.
     */

    let mut optimized_cfg = cfg.clone();

    let pipeline = OptimizationPipeline::default();

    let optimization_result = pipeline.run(&mut optimized_cfg);

    println!("\n=============== OPTIMIZED CFG ===============\n");
    println!("{}", optimized_cfg);

    println!("\n============= OPTIMIZATION RESULT ===========\n");

    println!("Optimization rounds: {}", optimization_result.rounds);

    println!(
        "Changed pass executions: {}",
        optimization_result.changed_passes
    );

    println!(
        "Reached fixed point: {}",
        optimization_result.reached_fixed_point
    );

    /*
     * Generate Graphviz DOT files.
     */

    std::fs::write("cfg.dot", cfg.to_dot())?;
    std::fs::write("cfg_defined.dot", defined.to_dot())?;
    std::fs::write("cfg_live.dot", live.to_dot())?;
    std::fs::write("cfg_reaching.dot", reaching.to_dot())?;
    std::fs::write("cfg_optimized.dot", optimized_cfg.to_dot())?;

    println!("\n================= DOT FILES =================\n");

    println!("- cfg.dot");
    println!("- cfg_defined.dot");
    println!("- cfg_live.dot");
    println!("- cfg_reaching.dot");
    println!("- cfg_optimized.dot");

    Ok(())
}
