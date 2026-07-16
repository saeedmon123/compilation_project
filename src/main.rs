mod minifun;
mod miniimp;

use miniimp::ast::{AExpr, BExpr, Command, Program};

use miniimp::cfg::program_to_cfg;
use miniimp::llvm::write_llvm_ir;
use miniimp::optimizations::check_undefined_variables;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
        MiniImp program used to demonstrate Fragment 8:

        y := x;

        if y < 10 then
            y := y + 8
        else
            y := y - 2

        x is the program input.
        y is the program output.

        Example:
        input  = 6
        output = 14

        The conditional assigns y in both branches.
        After applying mem2reg, LLVM inserts a phi
        node where the two control-flow paths merge.
    */

    let program = Program {
        input: "x".to_string(),
        output: "y".to_string(),

        body: Command::Seq(
            Box::new(Command::Assign(
                "y".to_string(),
                AExpr::Var("x".to_string()),
            )),
            Box::new(Command::If(
                BExpr::Less(
                    Box::new(AExpr::Var("y".to_string())),
                    Box::new(AExpr::Int(10)),
                ),
                Box::new(Command::Assign(
                    "y".to_string(),
                    AExpr::Add(
                        Box::new(AExpr::Var("y".to_string())),
                        Box::new(AExpr::Int(8)),
                    ),
                )),
                Box::new(Command::Assign(
                    "y".to_string(),
                    AExpr::Sub(
                        Box::new(AExpr::Var("y".to_string())),
                        Box::new(AExpr::Int(2)),
                    ),
                )),
            )),
        ),
    };

    let cfg = program_to_cfg(&program);

    println!("================ FRAGMENT 8 ================\n");

    println!("MiniImp program CFG:\n");
    println!("{}", cfg);

    match check_undefined_variables(&cfg) {
        Ok(()) => {
            println!("Undefined-variable check: passed");
        }

        Err(errors) => {
            eprintln!("Undefined-variable check failed:");

            for error in errors {
                eprintln!("- {}", error);
            }

            return Err("LLVM IR was not generated because the \
                 program may use undefined variables"
                .into());
        }
    }

    write_llvm_ir(&program, "program.ll")?;

    println!("LLVM IR generated successfully: program.ll\n");

    println!("Next commands:");

    println!(
        "  opt -passes=\"mem2reg\" \
         program.ll -S -o program_opt.ll"
    );

    println!(
        "  llc -filetype=obj \
         program_opt.ll -o program.o"
    );

    println!("  clang wrapper.c program.o -o program");

    println!("  ./program 6");

    println!("\nExpected result: 14");

    Ok(())
}
