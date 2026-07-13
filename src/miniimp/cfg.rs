use std::fmt;

use super::ast::{AExpr, BExpr, Command, Program};

pub type BlockId = usize;

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub input: String,
    pub output: String,
    pub entry: BlockId,
    pub exit: BlockId,
    blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub statement: SimpleStatement,
    pub outgoing: OutgoingEdges,
}

#[derive(Debug, Clone)]
pub struct AnnotatedControlFlowGraph<A> {
    pub input: String,
    pub output: String,
    pub entry: BlockId,
    pub exit: BlockId,
    blocks: Vec<AnnotatedBasicBlock<A>>,
}

#[derive(Debug, Clone)]
pub struct AnnotatedBasicBlock<A> {
    pub id: BlockId,
    pub statement: SimpleStatement,
    pub outgoing: OutgoingEdges,
    pub annotation: A,
}

#[derive(Debug, Clone)]
pub enum SimpleStatement {
    Skip,
    Assign(String, AExpr),
    Guard(BExpr),
}

#[derive(Debug, Clone)]
pub enum OutgoingEdges {
    End,
    Goto(BlockId),
    Branch { on_true: BlockId, on_false: BlockId },
}

impl ControlFlowGraph {
    pub fn from_program(program: &Program) -> Self {
        let mut builder = CfgBuilder::new();

        let exit = builder.add_block(SimpleStatement::Skip, OutgoingEdges::End);

        let entry = builder.build_command(&program.body, exit);

        Self {
            input: program.input.clone(),
            output: program.output.clone(),
            entry,
            exit,
            blocks: builder.blocks,
        }
    }

    pub fn blocks(&self) -> &[BasicBlock] {
        &self.blocks
    }

    pub fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(id)
    }

    pub fn successors(&self, id: BlockId) -> Vec<BlockId> {
        match self.block(id).map(|block| &block.outgoing) {
            Some(OutgoingEdges::End) | None => Vec::new(),

            Some(OutgoingEdges::Goto(next)) => {
                vec![*next]
            }

            Some(OutgoingEdges::Branch { on_true, on_false }) => {
                vec![*on_true, *on_false]
            }
        }
    }

    pub fn predecessors(&self, id: BlockId) -> Vec<BlockId> {
        self.blocks
            .iter()
            .filter(|block| self.successors(block.id).contains(&id))
            .map(|block| block.id)
            .collect()
    }

    pub fn annotate_with<A, F>(&self, mut annotation_for: F) -> AnnotatedControlFlowGraph<A>
    where
        F: FnMut(&BasicBlock) -> A,
    {
        let blocks = self
            .blocks
            .iter()
            .map(|block| AnnotatedBasicBlock {
                id: block.id,
                statement: block.statement.clone(),
                outgoing: block.outgoing.clone(),
                annotation: annotation_for(block),
            })
            .collect();

        AnnotatedControlFlowGraph {
            input: self.input.clone(),
            output: self.output.clone(),
            entry: self.entry,
            exit: self.exit,
            blocks,
        }
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph CFG {\n");

        dot.push_str("  node [shape=box];\n");

        for block in &self.blocks {
            let mut label = format!("B{}: {}", block.id, block.statement);

            if block.id == self.entry {
                label.push_str("\n(entry)");
            }

            if block.id == self.exit {
                label.push_str("\n(exit)");
            }

            dot.push_str(&format!(
                "  B{} [label=\"{}\"];\n",
                block.id,
                escape_dot_label(&label)
            ));
        }

        for block in &self.blocks {
            match block.outgoing {
                OutgoingEdges::End => {}

                OutgoingEdges::Goto(next) => {
                    dot.push_str(&format!("  B{} -> B{};\n", block.id, next));
                }

                OutgoingEdges::Branch { on_true, on_false } => {
                    dot.push_str(&format!(
                        "  B{} -> B{} [label=\"true\"];\n",
                        block.id, on_true
                    ));

                    dot.push_str(&format!(
                        "  B{} -> B{} [label=\"false\"];\n",
                        block.id, on_false
                    ));
                }
            }
        }

        dot.push_str("}\n");

        dot
    }
}

impl<A> AnnotatedControlFlowGraph<A> {
    pub fn blocks(&self) -> &[AnnotatedBasicBlock<A>] {
        &self.blocks
    }

    pub fn block(&self, id: BlockId) -> Option<&AnnotatedBasicBlock<A>> {
        self.blocks.get(id)
    }

    pub fn successors(&self, id: BlockId) -> Vec<BlockId> {
        match self.block(id).map(|block| &block.outgoing) {
            Some(OutgoingEdges::End) | None => Vec::new(),

            Some(OutgoingEdges::Goto(next)) => {
                vec![*next]
            }

            Some(OutgoingEdges::Branch { on_true, on_false }) => {
                vec![*on_true, *on_false]
            }
        }
    }

    pub fn predecessors(&self, id: BlockId) -> Vec<BlockId> {
        self.blocks
            .iter()
            .filter(|block| self.successors(block.id).contains(&id))
            .map(|block| block.id)
            .collect()
    }
}

impl<A: fmt::Display> AnnotatedControlFlowGraph<A> {
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph CFG {\n");

        dot.push_str("  node [shape=box];\n");

        for block in &self.blocks {
            let mut label = format!("B{}: {}\n{}", block.id, block.statement, block.annotation);

            if block.id == self.entry {
                label.push_str("\n(entry)");
            }

            if block.id == self.exit {
                label.push_str("\n(exit)");
            }

            dot.push_str(&format!(
                "  B{} [label=\"{}\"];\n",
                block.id,
                escape_dot_label(&label)
            ));
        }

        for block in &self.blocks {
            match block.outgoing {
                OutgoingEdges::End => {}

                OutgoingEdges::Goto(next) => {
                    dot.push_str(&format!("  B{} -> B{};\n", block.id, next));
                }

                OutgoingEdges::Branch { on_true, on_false } => {
                    dot.push_str(&format!(
                        "  B{} -> B{} [label=\"true\"];\n",
                        block.id, on_true
                    ));

                    dot.push_str(&format!(
                        "  B{} -> B{} [label=\"false\"];\n",
                        block.id, on_false
                    ));
                }
            }
        }

        dot.push_str("}\n");

        dot
    }
}

impl<A: fmt::Display> fmt::Display for AnnotatedControlFlowGraph<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "CFG(input: {}, output: {}, entry: B{}, exit: B{})",
            self.input, self.output, self.entry, self.exit
        )?;

        for block in &self.blocks {
            writeln!(f, "B{}: {}", block.id, block.statement)?;

            for line in block.annotation.to_string().lines() {
                writeln!(f, "    {}", line)?;
            }

            match block.outgoing {
                OutgoingEdges::End => {
                    writeln!(f, "    -> end")?;
                }

                OutgoingEdges::Goto(next) => {
                    writeln!(f, "    -> B{}", next)?;
                }

                OutgoingEdges::Branch { on_true, on_false } => {
                    writeln!(f, "    -> true: B{}, false: B{}", on_true, on_false)?;
                }
            }
        }

        Ok(())
    }
}

pub fn program_to_cfg(program: &Program) -> ControlFlowGraph {
    ControlFlowGraph::from_program(program)
}

struct CfgBuilder {
    blocks: Vec<BasicBlock>,
}

impl CfgBuilder {
    fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    fn add_block(&mut self, statement: SimpleStatement, outgoing: OutgoingEdges) -> BlockId {
        let id = self.blocks.len();

        self.blocks.push(BasicBlock {
            id,
            statement,
            outgoing,
        });

        id
    }

    fn build_command(&mut self, command: &Command, continuation: BlockId) -> BlockId {
        match command {
            Command::Skip => {
                self.add_block(SimpleStatement::Skip, OutgoingEdges::Goto(continuation))
            }

            Command::Assign(variable, expression) => self.add_block(
                SimpleStatement::Assign(variable.clone(), expression.clone()),
                OutgoingEdges::Goto(continuation),
            ),

            Command::Seq(first, second) => {
                let second_entry = self.build_command(second, continuation);

                self.build_command(first, second_entry)
            }

            Command::If(condition, then_branch, else_branch) => {
                let then_entry = self.build_command(then_branch, continuation);

                let else_entry = self.build_command(else_branch, continuation);

                self.add_block(
                    SimpleStatement::Guard(condition.clone()),
                    OutgoingEdges::Branch {
                        on_true: then_entry,
                        on_false: else_entry,
                    },
                )
            }

            Command::While(condition, body) => {
                /*
                 * Reserve the guard block first so that the body
                 * can create a back-edge to it.
                 */
                let guard_id = self.add_block(
                    SimpleStatement::Guard(condition.clone()),
                    OutgoingEdges::End,
                );

                let body_entry = self.build_command(body, guard_id);

                self.blocks[guard_id].outgoing = OutgoingEdges::Branch {
                    on_true: body_entry,
                    on_false: continuation,
                };

                guard_id
            }
        }
    }
}

impl fmt::Display for SimpleStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimpleStatement::Skip => {
                write!(f, "skip")
            }

            SimpleStatement::Assign(variable, expression) => {
                write!(f, "{} := {}", variable, expression)
            }

            SimpleStatement::Guard(condition) => {
                write!(f, "guard {}", condition)
            }
        }
    }
}

impl fmt::Display for AExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AExpr::Var(name) => {
                write!(f, "{}", name)
            }

            AExpr::Int(value) => {
                write!(f, "{}", value)
            }

            AExpr::Add(left, right) => {
                write!(f, "({} + {})", left, right)
            }

            AExpr::Sub(left, right) => {
                write!(f, "({} - {})", left, right)
            }

            AExpr::Mul(left, right) => {
                write!(f, "({} * {})", left, right)
            }
        }
    }
}

impl fmt::Display for BExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BExpr::True => {
                write!(f, "true")
            }

            BExpr::False => {
                write!(f, "false")
            }

            BExpr::And(left, right) => {
                write!(f, "({} && {})", left, right)
            }

            BExpr::Not(value) => {
                write!(f, "!({})", value)
            }

            BExpr::Less(left, right) => {
                write!(f, "({} < {})", left, right)
            }
        }
    }
}

impl fmt::Display for ControlFlowGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "CFG(input: {}, output: {}, entry: B{}, exit: B{})",
            self.input, self.output, self.entry, self.exit
        )?;

        for block in &self.blocks {
            write!(f, "B{}: {}", block.id, block.statement)?;

            match block.outgoing {
                OutgoingEdges::End => {
                    writeln!(f, " -> end")?;
                }

                OutgoingEdges::Goto(next) => {
                    writeln!(f, " -> B{}", next)?;
                }

                OutgoingEdges::Branch { on_true, on_false } => {
                    writeln!(f, " -> true: B{}, false: B{}", on_true, on_false)?;
                }
            }
        }

        Ok(())
    }
}

fn escape_dot_label(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_is_connected_in_order() {
        let program = Program {
            input: "x".to_string(),
            output: "z".to_string(),

            body: Command::Seq(
                Box::new(Command::Assign("y".to_string(), AExpr::Int(1))),
                Box::new(Command::Assign("z".to_string(), AExpr::Int(2))),
            ),
        };

        let cfg = program_to_cfg(&program);

        let first_successor = cfg.successors(cfg.entry);

        assert_eq!(first_successor.len(), 1);

        let second = first_successor[0];

        assert_eq!(cfg.successors(second), vec![cfg.exit]);
    }

    #[test]
    fn while_body_has_back_edge_to_guard() {
        let program = Program {
            input: "x".to_string(),
            output: "x".to_string(),

            body: Command::While(
                BExpr::Less(
                    Box::new(AExpr::Var("x".to_string())),
                    Box::new(AExpr::Int(10)),
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

        let guard = cfg.entry;
        let successors = cfg.successors(guard);
        let body = successors[0];

        assert_eq!(cfg.successors(body), vec![guard]);

        assert_eq!(successors[1], cfg.exit);
    }
}
