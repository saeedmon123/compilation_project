use std::collections::BTreeSet;
use std::fmt;

use super::ast::{AExpr, BExpr};
use super::cfg::{BlockId, ControlFlowGraph, SimpleStatement};
use super::dataflow::{
    Definition, DefinitionSet, defined_variables_analysis, live_variables_analysis,
    reaching_definitions_analysis, used_variables,
};

/*
 * Possible undefined-variable checking
 */

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndefinedVariable {
    pub variable: String,
    pub block: BlockId,
}

impl fmt::Display for UndefinedVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "variable '{}' may be undefined before block B{}",
            self.variable, self.block
        )
    }
}

pub fn check_undefined_variables(cfg: &ControlFlowGraph) -> Result<(), Vec<UndefinedVariable>> {
    let analysis = defined_variables_analysis(cfg);

    let mut errors = Vec::new();
    let mut already_reported = BTreeSet::new();

    for block in cfg.blocks() {
        let mut variables_used_here = used_variables(&block.statement);

        /*
         * The program output is read when the program finishes.
         * Therefore, it must be defined before the exit block.
         */
        if block.id == cfg.exit {
            variables_used_here.insert(cfg.output.clone());
        }

        let annotation = &analysis
            .block(block.id)
            .expect("every CFG block must have an annotation")
            .annotation;

        for variable in variables_used_here {
            if !annotation.in_set.contains(&variable)
                && already_reported.insert((variable.clone(), block.id))
            {
                errors.push(UndefinedVariable {
                    variable,
                    block: block.id,
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/*
 * Constant folding
 */

pub fn constant_folding(cfg: &mut ControlFlowGraph) -> bool {
    let mut changed = false;

    for block in cfg.blocks_mut() {
        let old_statement = block.statement.to_string();
        let current_statement = block.statement.clone();

        block.statement = match current_statement {
            SimpleStatement::Skip => SimpleStatement::Skip,

            SimpleStatement::Assign(variable, expression) => {
                SimpleStatement::Assign(variable, fold_aexpr(&expression))
            }

            SimpleStatement::Guard(condition) => SimpleStatement::Guard(fold_bexpr(&condition)),
        };

        if old_statement != block.statement.to_string() {
            changed = true;
        }
    }

    changed
}

fn fold_aexpr(expression: &AExpr) -> AExpr {
    match expression {
        AExpr::Var(variable) => AExpr::Var(variable.clone()),

        AExpr::Int(value) => AExpr::Int(*value),

        AExpr::Add(left, right) => {
            let left = fold_aexpr(left);
            let right = fold_aexpr(right);

            match (left, right) {
                (AExpr::Int(a), AExpr::Int(b)) => AExpr::Int(a + b),

                (left, right) => AExpr::Add(Box::new(left), Box::new(right)),
            }
        }

        AExpr::Sub(left, right) => {
            let left = fold_aexpr(left);
            let right = fold_aexpr(right);

            match (left, right) {
                (AExpr::Int(a), AExpr::Int(b)) => AExpr::Int(a - b),

                (left, right) => AExpr::Sub(Box::new(left), Box::new(right)),
            }
        }

        AExpr::Mul(left, right) => {
            let left = fold_aexpr(left);
            let right = fold_aexpr(right);

            match (left, right) {
                (AExpr::Int(a), AExpr::Int(b)) => AExpr::Int(a * b),

                (left, right) => AExpr::Mul(Box::new(left), Box::new(right)),
            }
        }
    }
}

fn fold_bexpr(expression: &BExpr) -> BExpr {
    match expression {
        BExpr::True => BExpr::True,

        BExpr::False => BExpr::False,

        BExpr::And(left, right) => {
            let left = fold_bexpr(left);
            let right = fold_bexpr(right);

            match (left, right) {
                (BExpr::False, _) => BExpr::False,

                (_, BExpr::False) => BExpr::False,

                (BExpr::True, other) => other,

                (other, BExpr::True) => other,

                (left, right) => BExpr::And(Box::new(left), Box::new(right)),
            }
        }

        BExpr::Not(value) => {
            let value = fold_bexpr(value);

            match value {
                BExpr::True => BExpr::False,

                BExpr::False => BExpr::True,

                other => BExpr::Not(Box::new(other)),
            }
        }

        BExpr::Less(left, right) => {
            let left = fold_aexpr(left);
            let right = fold_aexpr(right);

            match (left, right) {
                (AExpr::Int(a), AExpr::Int(b)) => {
                    if a < b {
                        BExpr::True
                    } else {
                        BExpr::False
                    }
                }

                (left, right) => BExpr::Less(Box::new(left), Box::new(right)),
            }
        }
    }
}

/*
 * Constant propagation
 */

pub fn constant_propagation(cfg: &mut ControlFlowGraph) -> bool {
    /*
     * Constant propagation uses reaching definitions.
     *
     * A variable can be replaced by a constant when every definition
     * of that variable reaching the block assigns the same constant.
     */
    let reaching = reaching_definitions_analysis(cfg);

    /*
     * Store a copy of the statements before modifying the CFG.
     * Reaching definitions refer to blocks using their block IDs.
     */
    let statements = cfg
        .blocks()
        .iter()
        .map(|block| block.statement.clone())
        .collect::<Vec<_>>();

    let mut changed = false;

    for block in cfg.blocks_mut() {
        let old_statement = block.statement.to_string();
        let current_statement = block.statement.clone();

        let definitions = &reaching
            .block(block.id)
            .expect("every CFG block must have an annotation")
            .annotation
            .in_set;

        block.statement = match current_statement {
            SimpleStatement::Skip => SimpleStatement::Skip,

            SimpleStatement::Assign(variable, expression) => SimpleStatement::Assign(
                variable,
                propagate_aexpr(&expression, definitions, &statements),
            ),

            SimpleStatement::Guard(condition) => {
                SimpleStatement::Guard(propagate_bexpr(&condition, definitions, &statements))
            }
        };

        if old_statement != block.statement.to_string() {
            changed = true;
        }
    }

    changed
}

fn propagate_aexpr(
    expression: &AExpr,
    definitions: &DefinitionSet,
    statements: &[SimpleStatement],
) -> AExpr {
    match expression {
        AExpr::Var(variable) => match constant_value(variable, definitions, statements) {
            Some(value) => AExpr::Int(value),

            None => AExpr::Var(variable.clone()),
        },

        AExpr::Int(value) => AExpr::Int(*value),

        AExpr::Add(left, right) => AExpr::Add(
            Box::new(propagate_aexpr(left, definitions, statements)),
            Box::new(propagate_aexpr(right, definitions, statements)),
        ),

        AExpr::Sub(left, right) => AExpr::Sub(
            Box::new(propagate_aexpr(left, definitions, statements)),
            Box::new(propagate_aexpr(right, definitions, statements)),
        ),

        AExpr::Mul(left, right) => AExpr::Mul(
            Box::new(propagate_aexpr(left, definitions, statements)),
            Box::new(propagate_aexpr(right, definitions, statements)),
        ),
    }
}

fn propagate_bexpr(
    expression: &BExpr,
    definitions: &DefinitionSet,
    statements: &[SimpleStatement],
) -> BExpr {
    match expression {
        BExpr::True => BExpr::True,

        BExpr::False => BExpr::False,

        BExpr::And(left, right) => BExpr::And(
            Box::new(propagate_bexpr(left, definitions, statements)),
            Box::new(propagate_bexpr(right, definitions, statements)),
        ),

        BExpr::Not(value) => BExpr::Not(Box::new(propagate_bexpr(value, definitions, statements))),

        BExpr::Less(left, right) => BExpr::Less(
            Box::new(propagate_aexpr(left, definitions, statements)),
            Box::new(propagate_aexpr(right, definitions, statements)),
        ),
    }
}

fn constant_value(
    variable: &str,
    definitions: &DefinitionSet,
    statements: &[SimpleStatement],
) -> Option<i32> {
    let mut result = None;
    let mut found_definition = false;

    for definition in definitions {
        if definition.variable() != variable {
            continue;
        }

        found_definition = true;

        let value = match definition {
            /*
             * Program input is not a compile-time constant.
             */
            Definition::Input(_) => {
                return None;
            }

            Definition::Assignment { block, .. } => {
                match statements.get(*block) {
                    Some(SimpleStatement::Assign(_, AExpr::Int(value))) => *value,

                    /*
                     * A non-constant assignment reaches this point.
                     */
                    _ => {
                        return None;
                    }
                }
            }
        };

        match result {
            /*
             * This is the first constant definition found.
             */
            None => {
                result = Some(value);
            }

            /*
             * More than one definition can reach a block.
             * Propagation is still safe if all definitions contain
             * the same constant.
             */
            Some(previous) if previous == value => {}

            /*
             * Different constants reach the block.
             */
            Some(_) => {
                return None;
            }
        }
    }

    if found_definition { result } else { None }
}

/*
 * Dead-store elimination
 */

pub fn dead_store_elimination(cfg: &mut ControlFlowGraph) -> bool {
    /*
     * An assignment to x is dead when x is not live after
     * the assignment.
     */
    let live = live_variables_analysis(cfg);

    let mut changed = false;

    for block in cfg.blocks_mut() {
        let variable = match &block.statement {
            SimpleStatement::Assign(variable, _) => variable.clone(),

            SimpleStatement::Skip | SimpleStatement::Guard(_) => {
                continue;
            }
        };

        let live_after = &live
            .block(block.id)
            .expect("every CFG block must have an annotation")
            .annotation
            .out_set;

        if !live_after.contains(&variable) {
            /*
             * The block is kept in the CFG, but its assignment
             * is replaced with skip.
             *
             * This preserves block IDs and control-flow edges.
             */
            block.statement = SimpleStatement::Skip;

            changed = true;
        }
    }

    changed
}

/*
 * Optimization pipeline
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationPass {
    ConstantPropagation,
    ConstantFolding,
    DeadStoreElimination,
}

impl fmt::Display for OptimizationPass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptimizationPass::ConstantPropagation => {
                write!(f, "constant propagation")
            }

            OptimizationPass::ConstantFolding => {
                write!(f, "constant folding")
            }

            OptimizationPass::DeadStoreElimination => {
                write!(f, "dead-store elimination")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationPipeline {
    /*
     * Passes can be enabled, disabled, or reordered
     * by changing this vector.
     */
    pub passes: Vec<OptimizationPass>,

    /*
     * Prevents an accidental infinite optimization loop.
     */
    pub maximum_rounds: usize,
}

impl Default for OptimizationPipeline {
    fn default() -> Self {
        Self {
            passes: vec![
                OptimizationPass::ConstantPropagation,
                OptimizationPass::ConstantFolding,
                OptimizationPass::DeadStoreElimination,
            ],

            maximum_rounds: 20,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimizationResult {
    pub rounds: usize,
    pub changed_passes: usize,
    pub reached_fixed_point: bool,
}

impl OptimizationPipeline {
    pub fn run(&self, cfg: &mut ControlFlowGraph) -> OptimizationResult {
        let mut rounds = 0;
        let mut changed_passes = 0;

        for _ in 0..self.maximum_rounds {
            rounds += 1;

            let mut changed_in_this_round = false;

            for pass in &self.passes {
                let changed = match pass {
                    OptimizationPass::ConstantPropagation => constant_propagation(cfg),

                    OptimizationPass::ConstantFolding => constant_folding(cfg),

                    OptimizationPass::DeadStoreElimination => dead_store_elimination(cfg),
                };

                if changed {
                    changed_in_this_round = true;
                    changed_passes += 1;
                }
            }

            /*
             * No pass changed the CFG.
             * A fixed point has been reached.
             */
            if !changed_in_this_round {
                return OptimizationResult {
                    rounds,
                    changed_passes,
                    reached_fixed_point: true,
                };
            }
        }

        OptimizationResult {
            rounds,
            changed_passes,
            reached_fixed_point: false,
        }
    }
}

/*
 * Tests
 */

#[cfg(test)]
mod tests {
    use super::*;

    use crate::miniimp::ast::{Command, Program};
    use crate::miniimp::cfg::program_to_cfg;

    fn find_assignment<'a>(cfg: &'a ControlFlowGraph, variable: &str) -> &'a SimpleStatement {
        &cfg.blocks()
            .iter()
            .find(|block| {
                matches!(
                    &block.statement,
                    SimpleStatement::Assign(name, _)
                        if name == variable
                )
            })
            .expect("assignment should exist")
            .statement
    }

    #[test]
    fn reports_possibly_undefined_variable() {
        let program = Program {
            input: "x".to_string(),
            output: "z".to_string(),

            body: Command::Seq(
                Box::new(Command::If(
                    BExpr::Less(
                        Box::new(AExpr::Var("x".to_string())),
                        Box::new(AExpr::Int(0)),
                    ),
                    Box::new(Command::Assign("y".to_string(), AExpr::Int(1))),
                    Box::new(Command::Skip),
                )),
                Box::new(Command::Assign(
                    "z".to_string(),
                    AExpr::Var("y".to_string()),
                )),
            ),
        };

        let cfg = program_to_cfg(&program);

        let errors = check_undefined_variables(&cfg).expect_err("y should be possibly undefined");

        assert!(errors.iter().any(|error| error.variable == "y"));
    }

    #[test]
    fn accepts_defined_variables() {
        let program = Program {
            input: "x".to_string(),
            output: "z".to_string(),

            body: Command::Seq(
                Box::new(Command::Assign(
                    "y".to_string(),
                    AExpr::Var("x".to_string()),
                )),
                Box::new(Command::Assign(
                    "z".to_string(),
                    AExpr::Var("y".to_string()),
                )),
            ),
        };

        let cfg = program_to_cfg(&program);

        assert!(check_undefined_variables(&cfg).is_ok());
    }

    #[test]
    fn folds_nested_constant_expression() {
        let program = Program {
            input: "input".to_string(),
            output: "out".to_string(),

            body: Command::Assign(
                "out".to_string(),
                AExpr::Mul(
                    Box::new(AExpr::Add(Box::new(AExpr::Int(2)), Box::new(AExpr::Int(3)))),
                    Box::new(AExpr::Int(4)),
                ),
            ),
        };

        let mut cfg = program_to_cfg(&program);

        assert!(constant_folding(&mut cfg));

        assert!(matches!(
            find_assignment(&cfg, "out"),
            SimpleStatement::Assign(_, AExpr::Int(20))
        ));
    }

    #[test]
    fn propagates_constant_to_later_assignment() {
        let program = Program {
            input: "input".to_string(),
            output: "out".to_string(),

            body: Command::Seq(
                Box::new(Command::Assign("x".to_string(), AExpr::Int(5))),
                Box::new(Command::Assign(
                    "out".to_string(),
                    AExpr::Add(
                        Box::new(AExpr::Var("x".to_string())),
                        Box::new(AExpr::Int(1)),
                    ),
                )),
            ),
        };

        let mut cfg = program_to_cfg(&program);

        assert!(constant_propagation(&mut cfg));

        assert!(matches!(
            find_assignment(&cfg, "out"),
            SimpleStatement::Assign(
                _,
                AExpr::Add(left, _)
            ) if matches!(
                left.as_ref(),
                AExpr::Int(5)
            )
        ));
    }

    #[test]
    fn does_not_propagate_different_constants_at_join() {
        let program = Program {
            input: "input".to_string(),
            output: "out".to_string(),

            body: Command::Seq(
                Box::new(Command::If(
                    BExpr::Less(
                        Box::new(AExpr::Var("input".to_string())),
                        Box::new(AExpr::Int(0)),
                    ),
                    Box::new(Command::Assign("x".to_string(), AExpr::Int(1))),
                    Box::new(Command::Assign("x".to_string(), AExpr::Int(2))),
                )),
                Box::new(Command::Assign(
                    "out".to_string(),
                    AExpr::Var("x".to_string()),
                )),
            ),
        };

        let mut cfg = program_to_cfg(&program);

        constant_propagation(&mut cfg);

        assert!(matches!(
            find_assignment(&cfg, "out"),
            SimpleStatement::Assign(
                _,
                AExpr::Var(variable)
            ) if variable == "x"
        ));
    }

    #[test]
    fn removes_assignment_whose_value_is_never_used() {
        let program = Program {
            input: "input".to_string(),
            output: "out".to_string(),

            body: Command::Seq(
                Box::new(Command::Assign("unused".to_string(), AExpr::Int(10))),
                Box::new(Command::Assign("out".to_string(), AExpr::Int(1))),
            ),
        };

        let mut cfg = program_to_cfg(&program);

        assert!(dead_store_elimination(&mut cfg));

        assert!(cfg.blocks().iter().any(|block| {
            block.id != cfg.exit && matches!(&block.statement, SimpleStatement::Skip)
        }));
    }

    #[test]
    fn keeps_assignment_to_program_output() {
        let program = Program {
            input: "input".to_string(),
            output: "out".to_string(),

            body: Command::Assign("out".to_string(), AExpr::Int(10)),
        };

        let mut cfg = program_to_cfg(&program);

        dead_store_elimination(&mut cfg);

        assert!(matches!(
            find_assignment(&cfg, "out"),
            SimpleStatement::Assign(_, AExpr::Int(10))
        ));
    }

    #[test]
    fn default_pipeline_reaches_fixed_point() {
        let program = Program {
            input: "input".to_string(),
            output: "out".to_string(),

            body: Command::Seq(
                Box::new(Command::Assign("x".to_string(), AExpr::Int(2))),
                Box::new(Command::Seq(
                    Box::new(Command::Assign(
                        "y".to_string(),
                        AExpr::Add(
                            Box::new(AExpr::Var("x".to_string())),
                            Box::new(AExpr::Int(3)),
                        ),
                    )),
                    Box::new(Command::Assign(
                        "out".to_string(),
                        AExpr::Var("y".to_string()),
                    )),
                )),
            ),
        };

        let mut cfg = program_to_cfg(&program);

        let result = OptimizationPipeline::default().run(&mut cfg);

        assert!(result.reached_fixed_point);

        assert!(matches!(
            find_assignment(&cfg, "out"),
            SimpleStatement::Assign(_, AExpr::Int(5))
        ));
    }
}
