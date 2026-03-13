use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use thiserror::Error;

use crate::syntax::{BinaryOp, Keyword, LiteralKind, OperatorConversionError};

#[allow(unused)]
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Keyword(Keyword),
    Literal { kind: LiteralKind, value: String },
    Semicolon,
    Comma,
    Period,

    BinaryOp(BinaryOp),
}

#[derive(Debug, Error)]
pub enum MalformedLiteralError {
    #[error("Empty character")]
    EmptyChar,

    #[error("Invalid escape sequence: `{0}`")]
    InvalidEscapeSequence(String),

    #[error("Invalid numeric literal")]
    InvalidNumber(String),

    #[error("Multicharacter character literal: `{0}`")]
    MulticharLiteral(String),

    #[error("Unterminated character")]
    UnterminatedCharacter,

    #[error("{0}")]
    OperatorConversionError(#[from] OperatorConversionError),
}

#[derive(Debug, Error)]
pub enum TokenizeError {
    #[error("Malformed literal: {0}")]
    MalformedLiteral(#[from] MalformedLiteralError),

    #[error("Unrecognized symbol: {0}")]
    UnrecognizedSymbol(char),
}

impl From<OperatorConversionError> for TokenizeError {
    fn from(value: OperatorConversionError) -> Self {
        Self::MalformedLiteral(MalformedLiteralError::OperatorConversionError(value))
    }
}

impl TokenizeError {
    fn qualify(self, line: usize) -> QualifiedTokenizeError {
        QualifiedTokenizeError {
            tokenize_error: self,
            line,
        }
    }
}

#[derive(Debug, Error)]
#[error("{tokenize_error} at line {line}")]
pub struct QualifiedTokenizeError {
    pub tokenize_error: TokenizeError,
    pub line: usize,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    line: usize,
}

fn qualify_result<T, E>(res: Result<T, E>, line: usize) -> Result<T, QualifiedTokenizeError>
where
    TokenizeError: From<E>,
{
    res.map_err(|e| TokenizeError::from(e).qualify(line))
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            tokens: vec![],
            line: 1,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, QualifiedTokenizeError> {
        while let Some(&ch) = self.chars.peek() {
            match ch {
                '0'..='9' | '-' => qualify_result(self.tokenize_number(), self.line)?,
                '\'' => qualify_result(self.tokenize_character(), self.line)?,
                '"' => qualify_result(self.tokenize_string(), self.line)?,
                '\n' => {
                    self.chars.next();
                    self.line += 1;
                }
                c if c.is_whitespace() => {
                    self.chars.next();
                }
                'a'..='z' => qualify_result(self.tokenize_identifier_or_keyword(), self.line)?,
                ';' => {
                    self.tokens.push(Token::Semicolon);
                    self.chars.next();
                }
                c if "*/%+-<>&|=!".contains(c) => {
                    qualify_result(self.tokenize_binary_op(), self.line)?;
                }
                _ => {
                    panic!("Unknown symbol {ch}");
                }
            }
        }

        Ok(self.tokens)
    }

    fn tokenize_number(&mut self) -> Result<(), MalformedLiteralError> {
        let mut accumulator = String::new();
        let mut period = false;

        while let Some(&ch) = self.chars.peek() {
            match ch {
                '0'..='9' => {
                    accumulator.push(ch);
                    self.chars.next();
                }
                '.' => {
                    accumulator.push('.');

                    if period {
                        return Err(MalformedLiteralError::InvalidNumber(accumulator));
                    }

                    period = true;
                    self.chars.next();
                }
                '_' => {
                    self.chars.next();
                }
                c if c.is_whitespace() || ";,(){}[]".contains(c) => break,
                _ => {
                    accumulator.push(ch);
                    return Err(MalformedLiteralError::InvalidNumber(accumulator));
                }
            }
        }

        self.tokens.push(Token::Literal {
            kind: LiteralKind::Integer,
            value: accumulator,
        });

        Ok(())
    }

    fn handle_escaped_character(&mut self) -> Result<char, MalformedLiteralError> {
        let Some(ch) = self.chars.peek() else {
            return Err(MalformedLiteralError::UnterminatedCharacter);
        };

        Ok(match ch {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '\\' => '\\',
            '\'' => '\'',
            '\"' => '\"',
            '0' => '\0',
            'u' => todo!("Add unicode escape sequencing"),
            _ => {
                return Err(MalformedLiteralError::InvalidEscapeSequence(format!(
                    "\\{}",
                    ch
                )));
            }
        })
    }

    fn tokenize_character(&mut self) -> Result<(), MalformedLiteralError> {
        let mut escape = false;
        let mut chars = String::new();

        while let Some(ch) = self.chars.next() {
            match ch {
                '\'' if !escape => {
                    self.chars.next();
                    break;
                }
                '\\' if !escape => {
                    escape = true;
                }
                '\n' => {
                    return Err(MalformedLiteralError::UnterminatedCharacter);
                }
                _ => {
                    if escape {
                        chars.push(self.handle_escaped_character()?);
                        escape = false;
                    } else {
                        chars.push(ch);
                    }
                }
            }
        }

        match chars.len() {
            0 => return Err(MalformedLiteralError::EmptyChar),
            1 => self.tokens.push(Token::Literal {
                kind: LiteralKind::Char,
                value: chars,
            }),
            _ => return Err(MalformedLiteralError::MulticharLiteral(chars)),
        };

        Ok(())
    }

    fn tokenize_string(&mut self) -> Result<(), TokenizeError> {
        let mut accumulator = String::new();
        let mut escape = false;

        while let Some(ch) = self.chars.next() {
            match ch {
                '"' if !escape => {
                    self.chars.next();
                    break;
                }
                '\\' if !escape => {
                    escape = true;
                }
                _ => {
                    if escape {
                        accumulator.push(self.handle_escaped_character()?);
                        escape = false;
                    } else {
                        accumulator.push(ch);
                    }
                }
            }
        }

        self.tokens.push(Token::Literal {
            kind: LiteralKind::String,
            value: accumulator,
        });

        Ok(())
    }

    fn tokenize_identifier_or_keyword(&mut self) -> Result<(), TokenizeError> {
        let mut accumulator = String::new();

        while let Some(&ch) = self.chars.peek() {
            match ch {
                'a'..='z' | '0'..='9' => {
                    accumulator.push(ch);
                    self.chars.next();
                }
                c if c.is_whitespace() || ";.(){}[]".contains(c) => break,
                _ => return Err(TokenizeError::UnrecognizedSymbol(ch)),
            }
        }

        self.tokens
            .push(if let Ok(keyword) = Keyword::from_str(&accumulator) {
                Token::Keyword(keyword)
            } else {
                Token::Identifier(accumulator)
            });

        Ok(())
    }

    fn tokenize_binary_op(&mut self) -> Result<(), TokenizeError> {
        let mut op_str = String::new();

        for ch in self.chars.by_ref() {
            if ch.is_whitespace() {
                break;
            }

            op_str.push(ch);
        }

        let binary_op = BinaryOp::try_from(op_str)?;
        self.tokens.push(Token::BinaryOp(binary_op));

        Ok(())
    }
}
