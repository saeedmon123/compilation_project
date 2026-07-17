use std::mem::discriminant;

use super::ast::{BinOp, Term};
use super::lexer::{Token, lex};
use super::types::Type;

pub fn parse_term(source: &str) -> Result<Term, String> {
    let tokens = lex(source)?;
    let mut parser = Parser::new(tokens);

    let term = parser.parse_term()?;

    parser.expect_simple(Token::Eof, "end of MiniFun expression")?;

    Ok(term)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn parse_term(&mut self) -> Result<Term, String> {
        match self.current() {
            Token::Let => self.parse_let(),
            Token::LetFun => self.parse_letfun(),
            Token::If => self.parse_if(),
            Token::Fun => self.parse_function(),
            _ => self.parse_and_expression(),
        }
    }

    fn parse_let(&mut self) -> Result<Term, String> {
        self.expect_simple(Token::Let, "'let'")?;

        let name = self.expect_identifier()?;

        self.expect_simple(Token::Equal, "'=' after the let-bound name")?;

        let value = self.parse_term()?;

        self.expect_simple(Token::In, "'in' after the let-bound expression")?;

        let body = self.parse_term()?;

        Ok(Term::Let(name, Box::new(value), Box::new(body)))
    }

    fn parse_letfun(&mut self) -> Result<Term, String> {
        self.expect_simple(Token::LetFun, "'letfun'")?;

        let name = self.expect_identifier()?;

        self.expect_simple(Token::LeftParen, "'(' after the recursive function name")?;

        let param = self.expect_identifier()?;

        self.expect_simple(Token::Colon, "':' after the recursive function parameter")?;

        let param_type = self.parse_type()?;

        self.expect_simple(
            Token::RightParen,
            "')' after the recursive function parameter",
        )?;

        self.expect_simple(
            Token::Colon,
            "':' before the recursive function return type",
        )?;

        let return_type = self.parse_type()?;

        self.expect_simple(Token::Equal, "'=' before the recursive function body")?;

        let body = self.parse_term()?;

        self.expect_simple(Token::In, "'in' after the recursive function body")?;

        let in_term = self.parse_term()?;

        Ok(Term::LetFun {
            name,
            param,
            param_type,
            return_type,
            body: Box::new(body),
            in_term: Box::new(in_term),
        })
    }

    fn parse_if(&mut self) -> Result<Term, String> {
        self.expect_simple(Token::If, "'if'")?;

        let condition = self.parse_term()?;

        self.expect_simple(Token::Then, "'then' after the if condition")?;

        let then_branch = self.parse_term()?;

        self.expect_simple(Token::Else, "'else' after the then branch")?;

        let else_branch = self.parse_term()?;

        Ok(Term::If(
            Box::new(condition),
            Box::new(then_branch),
            Box::new(else_branch),
        ))
    }

    fn parse_function(&mut self) -> Result<Term, String> {
        self.expect_simple(Token::Fun, "'fun'")?;

        self.expect_simple(Token::LeftParen, "'(' after 'fun'")?;

        let param = self.expect_identifier()?;

        self.expect_simple(Token::Colon, "':' after the function parameter")?;

        let param_type = self.parse_type()?;

        self.expect_simple(Token::RightParen, "')' after the function parameter")?;

        self.expect_simple(Token::Arrow, "'->' before the function body")?;

        let body = self.parse_term()?;

        Ok(Term::Fun {
            param,
            param_type,
            body: Box::new(body),
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let left = self.parse_type_atom()?;

        if self.consume(&Token::Arrow) {
            let right = self.parse_type()?;

            Ok(Type::Fun(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_type_atom(&mut self) -> Result<Type, String> {
        if self.consume(&Token::IntType) {
            return Ok(Type::Int);
        }

        if self.consume(&Token::BoolType) {
            return Ok(Type::Bool);
        }

        if self.consume(&Token::LeftParen) {
            let ty = self.parse_type()?;

            self.expect_simple(Token::RightParen, "')' after a type")?;

            return Ok(ty);
        }

        Err(self.error(format!("expected a type, but found {:?}", self.current())))
    }

    fn parse_and_expression(&mut self) -> Result<Term, String> {
        let mut term = self.parse_less_expression()?;

        while self.consume(&Token::AndAnd) {
            let right = self.parse_less_expression()?;

            term = Term::BinOp(Box::new(term), BinOp::And, Box::new(right));
        }

        Ok(term)
    }

    fn parse_less_expression(&mut self) -> Result<Term, String> {
        let mut term = self.parse_additive_expression()?;

        while self.consume(&Token::Less) {
            let right = self.parse_additive_expression()?;

            term = Term::BinOp(Box::new(term), BinOp::Less, Box::new(right));
        }

        Ok(term)
    }

    fn parse_additive_expression(&mut self) -> Result<Term, String> {
        let mut term = self.parse_multiplicative_expression()?;

        loop {
            if self.consume(&Token::Plus) {
                let right = self.parse_multiplicative_expression()?;

                term = Term::BinOp(Box::new(term), BinOp::Add, Box::new(right));
            } else if self.consume(&Token::Minus) {
                let right = self.parse_multiplicative_expression()?;

                term = Term::BinOp(Box::new(term), BinOp::Sub, Box::new(right));
            } else {
                break;
            }
        }

        Ok(term)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Term, String> {
        let mut term = self.parse_unary_expression()?;

        while self.consume(&Token::Star) {
            let right = self.parse_unary_expression()?;

            term = Term::BinOp(Box::new(term), BinOp::Mul, Box::new(right));
        }

        Ok(term)
    }

    fn parse_unary_expression(&mut self) -> Result<Term, String> {
        if self.consume(&Token::Not) {
            return Ok(Term::Not(Box::new(self.parse_unary_expression()?)));
        }

        if self.consume(&Token::Minus) {
            return Ok(Term::BinOp(
                Box::new(Term::Int(0)),
                BinOp::Sub,
                Box::new(self.parse_unary_expression()?),
            ));
        }

        self.parse_application()
    }

    fn parse_application(&mut self) -> Result<Term, String> {
        let mut term = self.parse_atom()?;

        while self.starts_atom() {
            let argument = self.parse_atom()?;

            term = Term::App(Box::new(term), Box::new(argument));
        }

        Ok(term)
    }

    fn parse_atom(&mut self) -> Result<Term, String> {
        match self.current().clone() {
            Token::Integer(value) => {
                self.advance();
                Ok(Term::Int(value))
            }

            Token::True => {
                self.advance();
                Ok(Term::Bool(true))
            }

            Token::False => {
                self.advance();
                Ok(Term::Bool(false))
            }

            Token::Identifier(name) => {
                self.advance();
                Ok(Term::Var(name))
            }

            Token::LeftParen => {
                self.advance();

                let term = self.parse_term()?;

                self.expect_simple(Token::RightParen, "')' after a MiniFun expression")?;

                Ok(term)
            }

            token => Err(self.error(format!(
                "expected a MiniFun expression, but found {:?}",
                token
            ))),
        }
    }

    fn starts_atom(&self) -> bool {
        matches!(
            self.current(),
            Token::Integer(_)
                | Token::True
                | Token::False
                | Token::Identifier(_)
                | Token::LeftParen
        )
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();

        if !matches!(token, Token::Eof) {
            self.position += 1;
        }

        token
    }

    fn check(&self, expected: &Token) -> bool {
        discriminant(self.current()) == discriminant(expected)
    }

    fn consume(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_simple(&mut self, expected: Token, description: &str) -> Result<(), String> {
        if self.consume(&expected) {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected {}, but found {:?}",
                description,
                self.current()
            )))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Identifier(name) => Ok(name),

            token => Err(self.error(format!("expected an identifier, but found {:?}", token))),
        }
    }

    fn error(&self, message: String) -> String {
        format!("Parser error at token {}: {}", self.position, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::minifun::eval::eval;

    use crate::minifun::inference::{MonoType, TypeEnvironment as InferenceEnvironment};

    use crate::minifun::runtime::{Value, empty_env};

    use crate::minifun::typecheck;

    use crate::minifun::types::{Type, empty_type_env};

    #[test]
    fn multiplication_has_higher_precedence_than_addition() {
        let term = parse_term("2 + 3 * 4").expect("parsing should succeed");

        let value = eval(&term, &mut empty_env()).expect("evaluation should succeed");

        assert!(matches!(value, Value::Int(14)));
    }

    #[test]
    fn parentheses_override_precedence() {
        let term = parse_term("(2 + 3) * 4").expect("parsing should succeed");

        let value = eval(&term, &mut empty_env()).expect("evaluation should succeed");

        assert!(matches!(value, Value::Int(20)));
    }

    #[test]
    fn parses_evaluates_and_typechecks_recursive_factorial() {
        let source = r#"
            letfun fact(n: Int): Int =
                if n < 1 then
                    1
                else
                    n * fact (n - 1)
            in
                fact 5
        "#;

        let term = parse_term(source).expect("parsing should succeed");

        let value = eval(&term, &mut empty_env()).expect("evaluation should succeed");

        let ty = typecheck::typecheck(&term, &mut empty_type_env())
            .expect("type checking should succeed");

        assert!(matches!(value, Value::Int(120)));
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn parsed_identity_function_is_polymorphic_during_inference() {
        let source = r#"
            let id = fun (x: Int) -> x in
            let ignored = id 5 in
            id true
        "#;

        let term = parse_term(source).expect("parsing should succeed");

        let mut env = InferenceEnvironment::new();

        let ty = crate::minifun::inference::typecheck(&term, &mut env)
            .expect("inference should succeed");

        assert_eq!(ty, MonoType::Bool);
    }
}
