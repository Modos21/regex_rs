use crate::token::{Repetition, Token, Whitespace};
use std::iter::Peekable;
use std::num::ParseIntError;
use std::str::CharIndices;

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    char_indices: Peekable<CharIndices<'a>>,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[error(transparent)]
pub enum LexingError {
    #[error("Expected a character but reached End of file")]
    EOF,

    #[error("Unexpected character: '{_0}'")]
    UnexpectedChar(char),

    ParseError(#[from] ParseIntError)
}

type LexResult<'a> = Result<Token, LexingError>;

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(regex: &'a str) -> Self {
        Self {
            char_indices: regex.char_indices().peekable(),
        }
    }

    fn read_escaped(&mut self) -> LexResult<'a> {
        match self.char_indices.peek() {
            Some((_, 's')) => {
                self.char_indices.next();
                Ok(Token::Whitespace(Whitespace::AnyWS))
            }
            Some((_, 'S')) => {
                self.char_indices.next();
                Ok(Token::Whitespace(Whitespace::NonWS))
            }

            Some((_, 'w')) => {
                self.char_indices.next();
                Ok(Token::WordChar)
            }
            Some((_, 'W')) => {
                self.char_indices.next();
                Ok(Token::NonWordChar)
            }

            Some((_, 'd')) => {
                self.char_indices.next();
                Ok(Token::Digit)
            }
            Some((_, 'D')) => {
                self.char_indices.next();
                Ok(Token::NonDigit)
            }

            Some((_, c)) => Ok(Token::Char(*c)),

            None => Err(LexingError::EOF),
        }
    }

    fn read_set(&mut self) -> LexResult<'a> {
        match self.char_indices.peek() {
            Some((_, '^')) => {
                // consume ^ to prevent it from being lexed as SOL
                self.char_indices.next();
                Ok(Token::ExclSet)
            }
            Some((_, _)) => Ok(Token::InclSet),
            None => Err(LexingError::EOF),
        }
    }

    fn read_whitespace(&mut self) -> LexResult<'a> {
        let mut count = 1;
        while let Some((_, ' ')) = self.char_indices.peek() {
            count += 1;
            self.char_indices.next();
        }
        Ok(Token::Whitespace(Whitespace::Repeated(count)))
    }

    fn read_repetition(&mut self) -> LexResult<'a> {
        let mut digits = vec![];
        let mut res = Ok(Token::EOF);

        while let Some((_, chr)) = self.char_indices.peek() {
            match chr {
                '}' => {
                    self.char_indices.next();
                    let num = digits.iter().collect::<String>().parse::<u32>().expect("");
                    res = Ok(Token::Repeat(Repetition::ExactlyN(num)));
                    break
                },

                c if c.is_digit(10) => {
                    digits.push(*c);
                    self.char_indices.next();
                },

                c => {
                    res = Err(LexingError::UnexpectedChar(*c));
                    break
                },
            }
        }

        res
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = LexResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (_, char) = self.char_indices.next()?;

        let token_result = match char {
            '\\' => self.read_escaped(),

            '.' => Ok(Token::Any),

            '|' => Ok(Token::Or),

            '[' => self.read_set(),

            ']' => Ok(Token::SetEnd),
            
            '-' => Ok(Token::RangeIndicator),

            '(' => Ok(Token::GroupStart),

            ')' => Ok(Token::GroupEnd),

            '{' => self.read_repetition(),

            '*' => Ok(Token::Repeat(Repetition::ZeroOrMore)),

            '+' => Ok(Token::Repeat(Repetition::OneOrMore)),

            '?' => Ok(Token::Repeat(Repetition::ZeroOrOne)),

            '\n' => Ok(Token::Newline),

            '\r' => match self.char_indices.peek() {
                Some((_, '\n')) => {
                    self.char_indices.next();
                    Ok(Token::Newline)
                }
                Some((_, c)) => Err(LexingError::UnexpectedChar(*c)),
                None => Ok(Token::Newline),
            },

            '\t' => Ok(Token::Tab),

            '\0' => Ok(Token::NullChar),

            '^' => Ok(Token::SOL),

            '$' => Ok(Token::EOL),

            ' ' => self.read_whitespace(),

            c => Ok(Token::Char(c)),
        };

        Some(token_result)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Lexer, LexingError};
    use crate::token::Repetition::{ExactlyN, OneOrMore, ZeroOrMore, ZeroOrOne};
    use crate::token::Token;
    use crate::token::Whitespace::Repeated;

    #[test]
    fn test_whitespace_tokens() -> Result<(), LexingError> {
        let src = "\r\n\t\r";
        let tokens = Lexer::new(src).collect::<Result<Vec<_>, LexingError>>()?;

        assert_eq!(tokens, vec![Token::Newline, Token::Tab, Token::Newline,]);

        Ok(())
    }

    #[test]
    fn test_repetition() -> Result<(), LexingError> {
        let src = ".*+?";
        let tokens = Lexer::new(src).collect::<Result<Vec<_>, LexingError>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::Any,
                Token::Repeat(ZeroOrMore),
                Token::Repeat(OneOrMore),
                Token::Repeat(ZeroOrOne),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_normal_chars() -> Result<(), LexingError> {
        let src = "abc";
        let tokens = Lexer::new(src).collect::<Result<Vec<_>, LexingError>>()?;

        assert_eq!(
            tokens,
            vec![Token::Char('a'), Token::Char('b'), Token::Char('c'),]
        );

        Ok(())
    }

    #[test]
    fn test_spaces() -> Result<(), LexingError> {
        let src = "a b  c   d";
        let tokens = Lexer::new(src).collect::<Result<Vec<_>, LexingError>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::Char('a'),
                Token::Whitespace(Repeated(1)),
                Token::Char('b'),
                Token::Whitespace(Repeated(2)),
                Token::Char('c'),
                Token::Whitespace(Repeated(3)),
                Token::Char('d'),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_groups() -> Result<(), LexingError> {
        let src = "[ab] [^cd]";
        let tokens = Lexer::new(src).collect::<Result<Vec<_>, LexingError>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::InclSet,
                Token::Char('a'),
                Token::Char('b'),
                Token::SetEnd,
                Token::Whitespace(Repeated(1)),
                Token::ExclSet,
                Token::Char('c'),
                Token::Char('d'),
                Token::SetEnd,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_advanced_input() -> Result<(), LexingError> {
        let src = "^[^a]+  ..\\d{3}";
        let tokens = Lexer::new(src).collect::<Result<Vec<_>, LexingError>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::SOL,
                Token::ExclSet,
                Token::Char('a'),
                Token::SetEnd,
                Token::Repeat(OneOrMore),
                Token::Whitespace(Repeated(2)),
                Token::Any,
                Token::Any,
                Token::Digit,
                Token::Repeat(ExactlyN(3)),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_ranges() -> Result<(), LexingError> {
        let src = "[a-z0-9]";
        let tokens = Lexer::new(src).collect::<Result<Vec<_>, LexingError>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::InclSet,
                Token::Char('a'),
                Token::RangeIndicator,
                Token::Char('z'),
                Token::Char('0'),
                Token::RangeIndicator,
                Token::Char('9'),
                Token::SetEnd,
            ]
        );

        Ok(())
    }
}
