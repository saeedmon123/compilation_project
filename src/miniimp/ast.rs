#[derive(Debug, Clone)]
pub struct Program {
    pub input: String,
    pub output: String,
    pub body: Command,
}

#[derive(Debug, Clone)]
pub enum Command {
    Skip,
    Assign(String, AExpr),
    Seq(Box<Command>, Box<Command>),
    If(BExpr, Box<Command>, Box<Command>),
    While(BExpr, Box<Command>),
}

#[derive(Debug, Clone)]
pub enum AExpr {
    Var(String),
    Int(i32),
    Add(Box<AExpr>, Box<AExpr>),
    Sub(Box<AExpr>, Box<AExpr>),
    Mul(Box<AExpr>, Box<AExpr>),
}

#[derive(Debug, Clone)]
pub enum BExpr {
    True,
    False,
    And(Box<BExpr>, Box<BExpr>),
    Not(Box<BExpr>),
    Less(Box<AExpr>, Box<AExpr>),
}