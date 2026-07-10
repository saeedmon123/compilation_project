use crate::minifun::types::Type;

#[derive(Debug, Clone)]
pub enum Term {
    Int(i32),
    Bool(bool),
    Var(String),

    Fun {
        param: String,
        param_type: Type,
        body: Box<Term>,
    },

    App(Box<Term>, Box<Term>),

    BinOp(Box<Term>, BinOp, Box<Term>),
    Not(Box<Term>),

    If(Box<Term>, Box<Term>, Box<Term>),

    Let(String, Box<Term>, Box<Term>),

    LetFun {
        name: String,
        param: String,
        param_type: Type,
        return_type: Type,
        body: Box<Term>,
        in_term: Box<Term>,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    And,
    Less,
}
