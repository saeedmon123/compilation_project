mod minifun;
mod miniimp;

use std::fs;

use minifun::inference::{MonoType, TypeEnvironment as InferenceEnvironment};

use minifun::runtime::Value;

use minifun::types::{Type, empty_type_env};

use miniimp::ast::AExpr;

use miniimp::cfg::{SimpleStatement, program_to_cfg};

use miniimp::dataflow::{
    defined_variables_analysis, live_variables_analysis, reaching_definitions_analysis,
};

use miniimp::optimizations::{OptimizationPipeline, check_undefined_variables};

const MINIIMP_SOURCE: &str = r#"
    input x;
    output y;

    y := x;

    if y < 10 then {
        y := y + 8;
    } else {
        y := y - 2;
    }
"#;

const MINIFUN_FACTORIAL_SOURCE: &str = r#"
    letfun fact(n: Int): Int =
        if n < 1 then
            1
        else
            n * fact (n - 1)
    in
        fact 5
"#;

const MINIFUN_POLYMORPHIC_SOURCE: &str = r#"
    let id = fun (x: Int) -> x in
    let ignored = id 5 in
    id true
"#;

const OPTIMIZATION_SOURCE: &str = r#"
    input x;
    output out;

    dead := 100;
    a := 30 * 2 + 4;
    b := a + 1;
    unused := 5;
    out := b * 1;
"#;

fn main() -> Result<(), String> {
    println!("========== COMPLETE PROJECT TEST ==========\n");

    /*
     * Fragment 1:
     * MiniImp lexer, parser, and evaluator.
     */
    let miniimp_tokens = miniimp::lexer::lex(MINIIMP_SOURCE)?;

    let miniimp_program = miniimp::parser::parse_program(MINIIMP_SOURCE)?;

    let miniimp_result = miniimp::eval::eval_program(&miniimp_program, 6)?;

    let precedence_expression = miniimp::parser::parse_aexpr("2 + 3 * 4")?;

    let precedence_result =
        miniimp::eval::eval_aexpr(&precedence_expression, &miniimp::runtime::Memory::new())?;

    let parenthesized_expression = miniimp::parser::parse_aexpr("(2 + 3) * 4")?;

    let parenthesized_result =
        miniimp::eval::eval_aexpr(&parenthesized_expression, &miniimp::runtime::Memory::new())?;

    let boolean_expression = miniimp::parser::parse_bexpr("(2 + 3) < 6 && not false")?;

    let boolean_result =
        miniimp::eval::eval_bexpr(&boolean_expression, &miniimp::runtime::Memory::new())?;

    ensure_equal("MiniImp result", miniimp_result, 14)?;

    ensure_equal("MiniImp precedence result", precedence_result, 14)?;

    ensure_equal("MiniImp parenthesized result", parenthesized_result, 20)?;

    ensure_equal("MiniImp Boolean result", boolean_result, true)?;

    println!("FRAGMENT 1 - MiniImp lexer, parser, evaluator");

    println!("Tokens produced: {}", miniimp_tokens.len() - 1);

    println!(
        "Precedence test 2 + 3 * 4: {} (expected 14)",
        precedence_result
    );

    println!(
        "Parentheses test (2 + 3) * 4: {} (expected 20)",
        parenthesized_result
    );

    println!("Evaluation with input 6: {} (expected 14)", miniimp_result);

    println!("Status: PASS\n");

    /*
     * Fragment 2:
     * MiniFun lexer, parser, and evaluator.
     */
    let minifun_tokens = minifun::lexer::lex(MINIFUN_FACTORIAL_SOURCE)?;

    let factorial_term = minifun::parser::parse_term(MINIFUN_FACTORIAL_SOURCE)?;

    let factorial_value = minifun::eval::eval(&factorial_term, &mut minifun::runtime::empty_env())?;

    match factorial_value {
        Value::Int(120) => {}

        other => {
            return Err(format!(
                "MiniFun evaluator produced {:?}, but Int(120) was expected",
                other
            ));
        }
    }

    println!("FRAGMENT 2 - MiniFun lexer, parser, evaluator");

    println!("Tokens produced: {}", minifun_tokens.len() - 1);

    println!("factorial(5): 120 (expected 120)");

    println!("Status: PASS\n");

    /*
     * Fragment 3:
     * Annotated static type checking.
     */
    let factorial_type = minifun::typecheck::typecheck(&factorial_term, &mut empty_type_env())?;

    ensure_equal("Annotated MiniFun type", factorial_type.clone(), Type::Int)?;

    println!("FRAGMENT 3 - MiniFun annotated type checking");

    println!(
        "Type of factorial program: {:?} (expected Int)",
        factorial_type
    );

    println!("Status: PASS\n");

    /*
     * Fragment 4:
     * Hindley-Milner style polymorphic inference.
     */
    let polymorphic_term = minifun::parser::parse_term(MINIFUN_POLYMORPHIC_SOURCE)?;

    let mut inference_environment = InferenceEnvironment::new();

    let inferred_type =
        minifun::inference::typecheck(&polymorphic_term, &mut inference_environment)?;

    ensure_equal(
        "Polymorphic MiniFun inferred type",
        inferred_type.clone(),
        MonoType::Bool,
    )?;

    println!("FRAGMENT 4 - Polymorphic type inference");

    println!("Inferred result type: {:?} (expected Bool)", inferred_type);

    println!("Status: PASS\n");

    /*
     * Fragment 5:
     * CFG construction from the parsed MiniImp AST.
     */
    let cfg = program_to_cfg(&miniimp_program);

    if cfg.blocks().len() != 5 {
        return Err(format!(
            "CFG contains {} blocks, but 5 were expected",
            cfg.blocks().len()
        ));
    }

    fs::write("cfg.dot", cfg.to_dot())
        .map_err(|error| format!("Could not write cfg.dot: {}", error))?;

    println!("FRAGMENT 5 - CFG construction");

    println!("CFG blocks: {} (expected 5)", cfg.blocks().len());

    println!("Generated: cfg.dot");
    println!("Status: PASS\n");

    /*
     * Fragment 6:
     * The three fixed-point data-flow analyses.
     */
    let defined = defined_variables_analysis(&cfg);

    let live = live_variables_analysis(&cfg);

    let reaching = reaching_definitions_analysis(&cfg);

    fs::write("cfg_defined.dot", defined.to_dot())
        .map_err(|error| format!("Could not write cfg_defined.dot: {}", error))?;

    fs::write("cfg_live.dot", live.to_dot())
        .map_err(|error| format!("Could not write cfg_live.dot: {}", error))?;

    fs::write("cfg_reaching.dot", reaching.to_dot())
        .map_err(|error| format!("Could not write cfg_reaching.dot: {}", error))?;

    println!("FRAGMENT 6 - Data-flow analyses");

    println!("Defined variables analysis: completed");

    println!("Live variables analysis: completed");

    println!("Reaching definitions analysis: completed");

    println!("Generated: cfg_defined.dot, cfg_live.dot, cfg_reaching.dot");

    println!("Status: PASS\n");

    /*
     * Fragment 7:
     * Undefined-variable checking and optimization.
     */
    check_undefined_variables(&cfg).map_err(|errors| {
        errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let optimization_program = miniimp::parser::parse_program(OPTIMIZATION_SOURCE)?;

    let mut optimized_cfg = program_to_cfg(&optimization_program);

    check_undefined_variables(&optimized_cfg).map_err(|errors| {
        errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let optimization_result = OptimizationPipeline::default().run(&mut optimized_cfg);

    if !optimization_result.reached_fixed_point {
        return Err("The optimization pipeline did not reach a fixed point".to_string());
    }

    let output_is_65 = optimized_cfg.blocks().iter().any(|block| {
        matches!(
            &block.statement,
            SimpleStatement::Assign(
                variable,
                AExpr::Int(65)
            ) if variable == "out"
        )
    });

    if !output_is_65 {
        return Err(
            "The optimized CFG does not contain the expected assignment out := 65".to_string(),
        );
    }

    fs::write("cfg_optimized.dot", optimized_cfg.to_dot())
        .map_err(|error| format!("Could not write cfg_optimized.dot: {}", error))?;

    println!("FRAGMENT 7 - Undefined-variable checking and optimizations");

    println!("Undefined-variable check: passed");

    println!("Optimization rounds: {}", optimization_result.rounds);

    println!(
        "Changed pass executions: {}",
        optimization_result.changed_passes
    );

    println!(
        "Reached fixed point: {}",
        optimization_result.reached_fixed_point
    );

    println!("Optimized output assignment: out := 65");

    println!("Generated: cfg_optimized.dot");

    println!("Status: PASS\n");

    /*
     * Fragment 8:
     * LLVM IR generation from the parsed program.
     */
    miniimp::llvm::write_llvm_ir(&miniimp_program, "program.ll")
        .map_err(|error| format!("Could not write program.ll: {}", error))?;

    println!("FRAGMENT 8 - LLVM IR generation");

    println!("Generated: program.ll");

    println!("Native pipeline:");

    println!("  opt -passes=\"mem2reg\" program.ll -S -o program_opt.ll");

    println!("  llc -filetype=obj program_opt.ll -o program.o");

    println!("  clang wrapper.c program.o -o program");

    println!("  ./program 6");

    println!("Expected native result: 14");

    println!("Status: PASS\n");

    println!("========== ALL RUST-SIDE TESTS PASSED ==========");

    Ok(())
}

fn ensure_equal<T>(label: &str, actual: T, expected: T) -> Result<(), String>
where
    T: PartialEq + std::fmt::Debug,
{
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{} was {:?}, but {:?} was expected",
            label, actual, expected
        ))
    }
}
