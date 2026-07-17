#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Input,
    Output,
    Skip,
    If,
    Then,
    Else,
    While,
    Do,
    True,
    False,

    Identifier(String),
    Integer(i32),

    Assign,
    Plus,
    Minus,
    Star,
    Less,
    AndAnd,
    Bang,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,

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
                "input" => Token::Input,
                "output" => Token::Output,
                "skip" => Token::Skip,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "while" => Token::While,
                "do" => Token::Do,
                "true" => Token::True,
                "false" => Token::False,
                "and" => Token::AndAnd,
                "not" => Token::Bang,
                _ => Token::Identifier(word),
            };

            tokens.push(token);
            continue;
        }

        match character {
            ':' => {
                if position + 1 < characters.len() && characters[position + 1] == '=' {
                    tokens.push(Token::Assign);
                    position += 2;
                } else {
                    return Err(format!(
                        "Lexer error at character {}: expected '=' after ':'",
                        position
                    ));
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

            '-' => {
                tokens.push(Token::Minus);
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

            '!' => {
                tokens.push(Token::Bang);
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

            '{' => {
                tokens.push(Token::LeftBrace);
                position += 1;
            }

            '}' => {
                tokens.push(Token::RightBrace);
                position += 1;
            }

            ';' => {
                tokens.push(Token::Semicolon);
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
    fn tokenizes_assignment_and_expression() {
        let tokens = lex("x := 2 + 3 * 4;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Integer(2),
                Token::Plus,
                Token::Integer(3),
                Token::Star,
                Token::Integer(4),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn accepts_keyword_aliases_for_boolean_operators() {
        let tokens = lex("not true and false").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::Bang,
                Token::True,
                Token::AndAnd,
                Token::False,
                Token::Eof,
            ]
        );
    }
}
