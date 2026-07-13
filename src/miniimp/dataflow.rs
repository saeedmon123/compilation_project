use std::collections::BTreeSet;
use std::fmt;

use super::ast::{AExpr, BExpr};
use super::cfg::{AnnotatedControlFlowGraph, BlockId, ControlFlowGraph, SimpleStatement};

pub type VariableSet = BTreeSet<String>;
pub type DefinitionSet = BTreeSet<Definition>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataflowAnnotation<T> {
    pub in_set: T,
    pub out_set: T,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Definition {
    Input(String),
    Assignment { variable: String, block: BlockId },
}

impl Definition {
    pub fn variable(&self) -> &str {
        match self {
            Definition::Input(variable) | Definition::Assignment { variable, .. } => variable,
        }
    }
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Definition::Input(variable) => write!(f, "{}@input", variable),
            Definition::Assignment { variable, block } => {
                write!(f, "{}@B{}", variable, block)
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Display for DataflowAnnotation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IN = {:?}\nOUT = {:?}", self.in_set, self.out_set)
    }
}

pub fn variables_in_aexpr(expression: &AExpr) -> VariableSet {
    match expression {
        AExpr::Var(variable) => BTreeSet::from([variable.clone()]),

        AExpr::Int(_) => BTreeSet::new(),

        AExpr::Add(left, right) | AExpr::Sub(left, right) | AExpr::Mul(left, right) => {
            union(&variables_in_aexpr(left), &variables_in_aexpr(right))
        }
    }
}

pub fn variables_in_bexpr(expression: &BExpr) -> VariableSet {
    match expression {
        BExpr::True | BExpr::False => BTreeSet::new(),

        BExpr::And(left, right) => union(&variables_in_bexpr(left), &variables_in_bexpr(right)),

        BExpr::Not(value) => variables_in_bexpr(value),

        BExpr::Less(left, right) => union(&variables_in_aexpr(left), &variables_in_aexpr(right)),
    }
}

pub fn used_variables(statement: &SimpleStatement) -> VariableSet {
    match statement {
        SimpleStatement::Skip => BTreeSet::new(),

        SimpleStatement::Assign(_, expression) => variables_in_aexpr(expression),

        SimpleStatement::Guard(condition) => variables_in_bexpr(condition),
    }
}

pub fn defined_variables(statement: &SimpleStatement) -> VariableSet {
    match statement {
        SimpleStatement::Assign(variable, _) => BTreeSet::from([variable.clone()]),

        SimpleStatement::Skip | SimpleStatement::Guard(_) => BTreeSet::new(),
    }
}

pub fn all_variables(cfg: &ControlFlowGraph) -> VariableSet {
    let mut variables = BTreeSet::from([cfg.input.clone(), cfg.output.clone()]);

    for block in cfg.blocks() {
        variables.extend(used_variables(&block.statement));
        variables.extend(defined_variables(&block.statement));
    }

    variables
}

pub fn defined_variables_analysis(
    cfg: &ControlFlowGraph,
) -> AnnotatedControlFlowGraph<DataflowAnnotation<VariableSet>> {
    let universe = all_variables(cfg);
    let block_count = cfg.blocks().len();

    // Defined variables is a forward must analysis.
    // Non-boundary values therefore start from the top element.
    let mut in_sets = vec![universe.clone(); block_count];
    let mut out_sets = vec![universe.clone(); block_count];

    let entry_boundary = BTreeSet::from([cfg.input.clone()]);

    loop {
        let mut changed = false;

        for block in cfg.blocks() {
            let mut incoming = cfg
                .predecessors(block.id)
                .into_iter()
                .map(|predecessor| out_sets[predecessor].clone())
                .collect::<Vec<_>>();

            /*
             * The entry block may also have a predecessor when the
             * entire program starts with a loop. The external program
             * input must still be considered an incoming value.
             */
            if block.id == cfg.entry {
                incoming.push(entry_boundary.clone());
            }

            let new_in = intersection_all(&incoming, &universe);

            let new_out = union(&new_in, &defined_variables(&block.statement));

            if new_in != in_sets[block.id] || new_out != out_sets[block.id] {
                in_sets[block.id] = new_in;
                out_sets[block.id] = new_out;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    cfg.annotate_with(|block| DataflowAnnotation {
        in_set: in_sets[block.id].clone(),
        out_set: out_sets[block.id].clone(),
    })
}

pub fn live_variables_analysis(
    cfg: &ControlFlowGraph,
) -> AnnotatedControlFlowGraph<DataflowAnnotation<VariableSet>> {
    let block_count = cfg.blocks().len();

    let mut in_sets = vec![BTreeSet::new(); block_count];
    let mut out_sets = vec![BTreeSet::new(); block_count];

    let exit_boundary = BTreeSet::from([cfg.output.clone()]);

    loop {
        let mut changed = false;

        /*
         * Live variables is a backward analysis.
         * Reverse block order usually reaches the fixed point faster.
         */
        for block in cfg.blocks().iter().rev() {
            let mut new_out = BTreeSet::new();

            for successor in cfg.successors(block.id) {
                new_out.extend(in_sets[successor].iter().cloned());
            }

            /*
             * The output variable must be live when the program exits.
             */
            if block.id == cfg.exit {
                new_out.extend(exit_boundary.iter().cloned());
            }

            /*
             * IN = USE union (OUT - DEF)
             */
            let mut new_in = difference(&new_out, &defined_variables(&block.statement));

            new_in.extend(used_variables(&block.statement));

            if new_in != in_sets[block.id] || new_out != out_sets[block.id] {
                in_sets[block.id] = new_in;
                out_sets[block.id] = new_out;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    cfg.annotate_with(|block| DataflowAnnotation {
        in_set: in_sets[block.id].clone(),
        out_set: out_sets[block.id].clone(),
    })
}

pub fn reaching_definitions_analysis(
    cfg: &ControlFlowGraph,
) -> AnnotatedControlFlowGraph<DataflowAnnotation<DefinitionSet>> {
    let block_count = cfg.blocks().len();

    let mut in_sets = vec![BTreeSet::new(); block_count];
    let mut out_sets = vec![BTreeSet::new(); block_count];

    let input_definition = Definition::Input(cfg.input.clone());

    loop {
        let mut changed = false;

        for block in cfg.blocks() {
            /*
             * Reaching definitions is a forward may analysis.
             * Therefore, predecessor information is combined using union.
             */
            let mut new_in = BTreeSet::new();

            for predecessor in cfg.predecessors(block.id) {
                new_in.extend(out_sets[predecessor].iter().cloned());
            }

            /*
             * The program input represents an initial definition.
             */
            if block.id == cfg.entry {
                new_in.insert(input_definition.clone());
            }

            let mut new_out = new_in.clone();

            if let SimpleStatement::Assign(variable, _) = &block.statement {
                /*
                 * Kill every previous definition of the variable.
                 */
                new_out.retain(|definition| definition.variable() != variable);

                /*
                 * Generate the new definition.
                 */
                new_out.insert(Definition::Assignment {
                    variable: variable.clone(),
                    block: block.id,
                });
            }

            if new_in != in_sets[block.id] || new_out != out_sets[block.id] {
                in_sets[block.id] = new_in;
                out_sets[block.id] = new_out;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    cfg.annotate_with(|block| DataflowAnnotation {
        in_set: in_sets[block.id].clone(),
        out_set: out_sets[block.id].clone(),
    })
}

fn union<T: Ord + Clone>(left: &BTreeSet<T>, right: &BTreeSet<T>) -> BTreeSet<T> {
    left.union(right).cloned().collect()
}

fn difference<T: Ord + Clone>(left: &BTreeSet<T>, right: &BTreeSet<T>) -> BTreeSet<T> {
    left.difference(right).cloned().collect()
}

fn intersection_all<T: Ord + Clone>(sets: &[BTreeSet<T>], default: &BTreeSet<T>) -> BTreeSet<T> {
    let Some((first, rest)) = sets.split_first() else {
        return default.clone();
    };

    rest.iter().fold(first.clone(), |current, next| {
        current.intersection(next).cloned().collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::miniimp::ast::{Command, Program};
    use crate::miniimp::cfg::program_to_cfg;

    fn assignment_block(cfg: &ControlFlowGraph, variable: &str, integer: i32) -> BlockId {
        cfg.blocks()
            .iter()
            .find_map(|block| match &block.statement {
                SimpleStatement::Assign(name, AExpr::Int(value))
                    if name == variable && *value == integer =>
                {
                    Some(block.id)
                }

                _ => None,
            })
            .expect("assignment block should exist")
    }

    #[test]
    fn support_functions_visit_nested_expressions() {
        let expression = BExpr::And(
            Box::new(BExpr::Less(
                Box::new(AExpr::Add(
                    Box::new(AExpr::Var("x".to_string())),
                    Box::new(AExpr::Var("y".to_string())),
                )),
                Box::new(AExpr::Var("z".to_string())),
            )),
            Box::new(BExpr::Not(Box::new(BExpr::Less(
                Box::new(AExpr::Var("w".to_string())),
                Box::new(AExpr::Int(0)),
            )))),
        );

        assert_eq!(
            variables_in_bexpr(&expression),
            BTreeSet::from([
                "w".to_string(),
                "x".to_string(),
                "y".to_string(),
                "z".to_string(),
            ])
        );
    }

    #[test]
    fn defined_variables_uses_intersection_at_join_points() {
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
        let analysis = defined_variables_analysis(&cfg);

        let z_block = cfg
            .blocks()
            .iter()
            .find(|block| {
                matches!(
                    &block.statement,
                    SimpleStatement::Assign(name, _)
                        if name == "z"
                )
            })
            .expect("z assignment should exist")
            .id;

        let annotation = &analysis
            .block(z_block)
            .expect("annotated block should exist")
            .annotation;

        assert!(annotation.in_set.contains("x"));
        assert!(!annotation.in_set.contains("y"));
    }

    #[test]
    fn live_variables_use_rhs_before_killing_lhs() {
        let program = Program {
            input: "y".to_string(),
            output: "x".to_string(),

            body: Command::Assign(
                "x".to_string(),
                AExpr::Add(
                    Box::new(AExpr::Var("y".to_string())),
                    Box::new(AExpr::Int(1)),
                ),
            ),
        };

        let cfg = program_to_cfg(&program);
        let analysis = live_variables_analysis(&cfg);

        let annotation = &analysis
            .block(cfg.entry)
            .expect("entry block should exist")
            .annotation;

        assert_eq!(annotation.in_set, BTreeSet::from(["y".to_string()]));

        assert_eq!(annotation.out_set, BTreeSet::from(["x".to_string()]));
    }

    #[test]
    fn reaching_definitions_kills_previous_definition_of_same_variable() {
        let program = Program {
            input: "x".to_string(),
            output: "x".to_string(),

            body: Command::Seq(
                Box::new(Command::Assign("x".to_string(), AExpr::Int(1))),
                Box::new(Command::Assign("x".to_string(), AExpr::Int(2))),
            ),
        };

        let cfg = program_to_cfg(&program);
        let analysis = reaching_definitions_analysis(&cfg);

        let first = assignment_block(&cfg, "x", 1);
        let second = assignment_block(&cfg, "x", 2);

        let second_annotation = &analysis
            .block(second)
            .expect("second assignment should exist")
            .annotation;

        assert!(second_annotation.in_set.contains(&Definition::Assignment {
            variable: "x".to_string(),
            block: first,
        }));

        assert_eq!(
            second_annotation.out_set,
            BTreeSet::from([Definition::Assignment {
                variable: "x".to_string(),
                block: second,
            }])
        );
    }

    #[test]
    fn reaching_definitions_handles_loop_back_edges() {
        let program = Program {
            input: "x".to_string(),
            output: "x".to_string(),

            body: Command::While(
                BExpr::Less(
                    Box::new(AExpr::Var("x".to_string())),
                    Box::new(AExpr::Int(3)),
                ),
                Box::new(Command::Assign(
                    "x".to_string(),
                    AExpr::Add(
                        Box::new(AExpr::Var("x".to_string())),
                        Box::new(AExpr::Int(1)),
                    ),
                )),
            ),
        };

        let cfg = program_to_cfg(&program);
        let analysis = reaching_definitions_analysis(&cfg);

        let body = cfg.successors(cfg.entry)[0];

        let guard_annotation = &analysis
            .block(cfg.entry)
            .expect("guard should exist")
            .annotation;

        assert!(
            guard_annotation
                .in_set
                .contains(&Definition::Input("x".to_string()))
        );

        assert!(guard_annotation.in_set.contains(&Definition::Assignment {
            variable: "x".to_string(),
            block: body,
        }));
    }
}
