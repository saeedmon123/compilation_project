use crate::miniimp::ast::{AExpr, BExpr, Command, Program};
use crate::miniimp::runtime::Memory;

pub fn eval_aexpr(expr: &AExpr, memory: &Memory) -> Result<i32, String> {
    match expr {
        AExpr::Int(n) => Ok(*n),

        AExpr::Var(name) => memory.get(name),

        AExpr::Add(left, right) => Ok(eval_aexpr(left, memory)? + eval_aexpr(right, memory)?),

        AExpr::Sub(left, right) => Ok(eval_aexpr(left, memory)? - eval_aexpr(right, memory)?),

        AExpr::Mul(left, right) => Ok(eval_aexpr(left, memory)? * eval_aexpr(right, memory)?),
    }
}

pub fn eval_bexpr(expr: &BExpr, memory: &Memory) -> Result<bool, String> {
    match expr {
        BExpr::True => Ok(true),

        BExpr::False => Ok(false),

        BExpr::And(left, right) => Ok(eval_bexpr(left, memory)? && eval_bexpr(right, memory)?),

        BExpr::Not(value) => Ok(!eval_bexpr(value, memory)?),

        BExpr::Less(left, right) => Ok(eval_aexpr(left, memory)? < eval_aexpr(right, memory)?),
    }
}

pub fn eval_command(command: &Command, memory: &mut Memory) -> Result<(), String> {
    match command {
        Command::Skip => Ok(()),

        Command::Assign(name, expr) => {
            let value = eval_aexpr(expr, memory)?;
            memory.set(name.clone(), value);
            Ok(())
        }

        Command::Seq(first, second) => {
            eval_command(first, memory)?;
            eval_command(second, memory)?;
            Ok(())
        }

        Command::If(condition, then_branch, else_branch) => {
            if eval_bexpr(condition, memory)? {
                eval_command(then_branch, memory)
            } else {
                eval_command(else_branch, memory)
            }
        }

        Command::While(condition, body) => {
            while eval_bexpr(condition, memory)? {
                eval_command(body, memory)?;
            }
            Ok(())
        }
    }
}

pub fn eval_program(program: &Program, input_value: i32) -> Result<i32, String> {
    let mut memory = Memory::new();

    memory.set(program.input.clone(), input_value);

    eval_command(&program.body, &mut memory)?;

    memory.get(&program.output)
}
