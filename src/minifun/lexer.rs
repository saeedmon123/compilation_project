#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Fun,
    Let,
    LetFun,
    In,
    If,
    Then,
    Else,
    True,
    False,
    Not,

    IntType,
    BoolType,

    Identifier(String),
    Integer(i32),

    Plus,
    Minus,
    Star,
    Less,
    AndAnd,
    Equal,
    Colon,
    Arrow,

    LeftParen,
    RightParen,

    Eof,
}

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let characters = source.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut position = 0;

    while position < characters.len() {
        let character = characters[position];

        if character.is_whitespace() {
            position += 1;
            continue;
        }

        // Line comments begin with // and continue to the next line.
        if character == '/' && position + 1 < characters.len() && characters[position + 1] == '/' {
            position += 2;

            while position < characters.len() && characters[position] != '\n' {
                position += 1;
            }

            continue;
        }

        if character.is_ascii_digit() {
            let start = position;

            while position < characters.len() && characters[position].is_ascii_digit() {
                position += 1;
            }

            let text = characters[start..position].iter().collect::<String>();

            let value = text.parse::<i32>().map_err(|_| {
                format!(
                    "Lexer error at character {}: integer '{}' is outside the i32 range",
                    start, text
                )
            })?;

            tokens.push(Token::Integer(value));
            continue;
        }

        if character.is_ascii_alphabetic() || character == '_' {
            let start = position;

            while position < characters.len()
                && (characters[position].is_ascii_alphanumeric() || characters[position] == '_')
            {
                position += 1;
            }

            let word = characters[start..position].iter().collect::<String>();

            let token = match word.as_str() {
                "fun" => Token::Fun,
                "let" => Token::Let,
                "letfun" => Token::LetFun,
                "in" => Token::In,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "true" => Token::True,
                "false" => Token::False,
                "not" => Token::Not,
                "and" => Token::AndAnd,
                "Int" => Token::IntType,
                "Bool" => Token::BoolType,
                _ => Token::Identifier(word),
            };

            tokens.push(token);
            continue;
        }

        match character {
            '-' => {
                if position + 1 < characters.len() && characters[position + 1] == '>' {
                    tokens.push(Token::Arrow);
                    position += 2;
                } else {
                    tokens.push(Token::Minus);
                    position += 1;
                }
            }

            '&' => {
                if position + 1 < characters.len() && characters[position + 1] == '&' {
                    tokens.push(Token::AndAnd);
                    position += 2;
                } else {
                    return Err(format!(
                        "Lexer error at character {}: expected another '&'",
                        position
                    ));
                }
            }

            '+' => {
                tokens.push(Token::Plus);
                position += 1;
            }

            '*' => {
                tokens.push(Token::Star);
                position += 1;
            }

            '<' => {
                tokens.push(Token::Less);
                position += 1;
            }

            '=' => {
                tokens.push(Token::Equal);
                position += 1;
            }

            ':' => {
                tokens.push(Token::Colon);
                position += 1;
            }

            '(' => {
                tokens.push(Token::LeftParen);
                position += 1;
            }

            ')' => {
                tokens.push(Token::RightParen);
                position += 1;
            }

            _ => {
                return Err(format!(
                    "Lexer error at character {}: unexpected character '{}'",
                    position, character
                ));
            }
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_annotated_function() {
        let tokens = lex("fun (x: Int) -> x + 1").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::Fun,
                Token::LeftParen,
                Token::Identifier("x".to_string()),
                Token::Colon,
                Token::IntType,
                Token::RightParen,
                Token::Arrow,
                Token::Identifier("x".to_string()),
                Token::Plus,
                Token::Integer(1),
                Token::Eof,
            ]
        );
    }
}
