use std::mem::discriminant;

use super::ast::{AExpr, BExpr, Command, Program};
use super::lexer::{Token, lex};

pub fn parse_program(source: &str) -> Result<Program, String> {
    let tokens = lex(source)?;
    Parser::new(tokens).parse_program()
}

pub fn parse_aexpr(source: &str) -> Result<AExpr, String> {
    let tokens = lex(source)?;
    let mut parser = Parser::new(tokens);

    let expression = parser.parse_additive_expression()?;

    parser.expect_simple(Token::Eof, "end of arithmetic expression")?;

    Ok(expression)
}

pub fn parse_bexpr(source: &str) -> Result<BExpr, String> {
    let tokens = lex(source)?;
    let mut parser = Parser::new(tokens);

    let expression = parser.parse_boolean_expression()?;

    parser.expect_simple(Token::Eof, "end of Boolean expression")?;

    Ok(expression)
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

    fn parse_program(mut self) -> Result<Program, String> {
        self.expect_simple(Token::Input, "'input'")?;
        let input = self.expect_identifier()?;

        self.expect_simple(Token::Semicolon, "';' after the input declaration")?;

        self.expect_simple(Token::Output, "'output'")?;
        let output = self.expect_identifier()?;

        self.expect_simple(Token::Semicolon, "';' after the output declaration")?;

        let body = self.parse_command_sequence()?;

        self.expect_simple(Token::Eof, "end of MiniImp program")?;

        Ok(Program {
            input,
            output,
            body,
        })
    }

    fn parse_command_sequence(&mut self) -> Result<Command, String> {
        if self.check(&Token::RightBrace) || self.check(&Token::Eof) {
            return Ok(Command::Skip);
        }

        let mut commands = vec![self.parse_command_atom()?];

        while self.consume(&Token::Semicolon) {
            // Multiple or trailing semicolons are harmless.
            while self.consume(&Token::Semicolon) {}

            if self.check(&Token::RightBrace) || self.check(&Token::Eof) {
                break;
            }

            commands.push(self.parse_command_atom()?);
        }

        let mut iterator = commands.into_iter();

        let first = iterator
            .next()
            .expect("a non-empty command sequence must have a first command");

        Ok(iterator.fold(first, |left, right| {
            Command::Seq(Box::new(left), Box::new(right))
        }))
    }

    fn parse_command_atom(&mut self) -> Result<Command, String> {
        match self.current().clone() {
            Token::Skip => {
                self.advance();
                Ok(Command::Skip)
            }

            Token::Identifier(name) => {
                self.advance();

                self.expect_simple(Token::Assign, "':=' after an assignment target")?;

                let expression = self.parse_additive_expression()?;

                Ok(Command::Assign(name, expression))
            }

            Token::If => self.parse_if_command(),

            Token::While => self.parse_while_command(),

            Token::LeftBrace => self.parse_block(),

            token => Err(self.error(format!("expected a command, but found {:?}", token))),
        }
    }

    fn parse_if_command(&mut self) -> Result<Command, String> {
        self.expect_simple(Token::If, "'if'")?;

        let condition = self.parse_boolean_expression()?;

        self.expect_simple(Token::Then, "'then' after the if condition")?;

        let then_branch = self.parse_command_or_block()?;

        self.expect_simple(Token::Else, "'else' after the then branch")?;

        let else_branch = self.parse_command_or_block()?;

        Ok(Command::If(
            condition,
            Box::new(then_branch),
            Box::new(else_branch),
        ))
    }

    fn parse_while_command(&mut self) -> Result<Command, String> {
        self.expect_simple(Token::While, "'while'")?;

        let condition = self.parse_boolean_expression()?;

        self.expect_simple(Token::Do, "'do' after the while condition")?;

        let body = self.parse_command_or_block()?;

        Ok(Command::While(condition, Box::new(body)))
    }

    fn parse_command_or_block(&mut self) -> Result<Command, String> {
        if self.check(&Token::LeftBrace) {
            self.parse_block()
        } else {
            self.parse_command_atom()
        }
    }

    fn parse_block(&mut self) -> Result<Command, String> {
        self.expect_simple(Token::LeftBrace, "'{'")?;

        let command = self.parse_command_sequence()?;

        self.expect_simple(Token::RightBrace, "'}' after a command block")?;

        Ok(command)
    }

    fn parse_boolean_expression(&mut self) -> Result<BExpr, String> {
        self.parse_boolean_and()
    }

    fn parse_boolean_and(&mut self) -> Result<BExpr, String> {
        let mut expression = self.parse_boolean_not()?;

        while self.consume(&Token::AndAnd) {
            let right = self.parse_boolean_not()?;

            expression = BExpr::And(Box::new(expression), Box::new(right));
        }

        Ok(expression)
    }

    fn parse_boolean_not(&mut self) -> Result<BExpr, String> {
        if self.consume(&Token::Bang) {
            return Ok(BExpr::Not(Box::new(self.parse_boolean_not()?)));
        }

        self.parse_boolean_atom()
    }

    fn parse_boolean_atom(&mut self) -> Result<BExpr, String> {
        if self.consume(&Token::True) {
            return Ok(BExpr::True);
        }

        if self.consume(&Token::False) {
            return Ok(BExpr::False);
        }

        /*
         * First try a parenthesized Boolean expression.
         *
         * If that fails, restore the parser position so an
         * arithmetic expression such as (x + 1) < 10 can be
         * parsed as a comparison.
         */
        if self.check(&Token::LeftParen) {
            let saved_position = self.position;
            self.advance();

            if let Ok(expression) = self.parse_boolean_expression() {
                if self.consume(&Token::RightParen) {
                    return Ok(expression);
                }
            }

            self.position = saved_position;
        }

        let left = self.parse_additive_expression()?;

        self.expect_simple(Token::Less, "'<' in a Boolean comparison")?;

        let right = self.parse_additive_expression()?;

        Ok(BExpr::Less(Box::new(left), Box::new(right)))
    }

    fn parse_additive_expression(&mut self) -> Result<AExpr, String> {
        let mut expression = self.parse_multiplicative_expression()?;

        loop {
            if self.consume(&Token::Plus) {
                let right = self.parse_multiplicative_expression()?;

                expression = AExpr::Add(Box::new(expression), Box::new(right));
            } else if self.consume(&Token::Minus) {
                let right = self.parse_multiplicative_expression()?;

                expression = AExpr::Sub(Box::new(expression), Box::new(right));
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<AExpr, String> {
        let mut expression = self.parse_unary_arithmetic_expression()?;

        while self.consume(&Token::Star) {
            let right = self.parse_unary_arithmetic_expression()?;

            expression = AExpr::Mul(Box::new(expression), Box::new(right));
        }

        Ok(expression)
    }

    fn parse_unary_arithmetic_expression(&mut self) -> Result<AExpr, String> {
        if self.consume(&Token::Minus) {
            return Ok(AExpr::Sub(
                Box::new(AExpr::Int(0)),
                Box::new(self.parse_unary_arithmetic_expression()?),
            ));
        }

        self.parse_arithmetic_atom()
    }

    fn parse_arithmetic_atom(&mut self) -> Result<AExpr, String> {
        match self.current().clone() {
            Token::Integer(value) => {
                self.advance();
                Ok(AExpr::Int(value))
            }

            Token::Identifier(name) => {
                self.advance();
                Ok(AExpr::Var(name))
            }

            Token::LeftParen => {
                self.advance();

                let expression = self.parse_additive_expression()?;

                self.expect_simple(Token::RightParen, "')' after an arithmetic expression")?;

                Ok(expression)
            }

            token => Err(self.error(format!(
                "expected an arithmetic expression, but found {:?}",
                token
            ))),
        }
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

    use crate::miniimp::eval::{eval_aexpr, eval_program};

    use crate::miniimp::runtime::Memory;

    #[test]
    fn multiplication_has_higher_precedence_than_addition() {
        let expression = parse_aexpr("2 + 3 * 4").expect("parsing should succeed");

        let value = eval_aexpr(&expression, &Memory::new()).expect("evaluation should succeed");

        assert_eq!(value, 14);
    }

    #[test]
    fn parentheses_override_arithmetic_precedence() {
        let expression = parse_aexpr("(2 + 3) * 4").expect("parsing should succeed");

        let value = eval_aexpr(&expression, &Memory::new()).expect("evaluation should succeed");

        assert_eq!(value, 20);
    }

    #[test]
    fn parses_and_executes_if_program() {
        let source = r#"
            input x;
            output y;

            y := x;

            if y < 10 then {
                y := y + 8;
            } else {
                y := y - 2;
            }
        "#;

        let program = parse_program(source).expect("parsing should succeed");

        assert_eq!(eval_program(&program, 6).unwrap(), 14);

        assert_eq!(eval_program(&program, 12).unwrap(), 10);
    }

    #[test]
    fn parses_parenthesized_arithmetic_inside_comparison() {
        let expression = parse_bexpr("(x + 1) < 10").expect("parsing should succeed");

        let mut memory = Memory::new();

        memory.set("x".to_string(), 8);

        assert!(crate::miniimp::eval::eval_bexpr(&expression, &memory,).unwrap());
    }

    #[test]
    fn parses_and_executes_while_program() {
        let source = r#"
            input x;
            output x;

            while x < 3 do {
                x := x + 1;
            }
        "#;

        let program = parse_program(source).expect("parsing should succeed");

        assert_eq!(eval_program(&program, 0).unwrap(), 3);
    }
}
