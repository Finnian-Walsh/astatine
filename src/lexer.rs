use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use thiserror::Error;

use crate::syntax::{Keyword, LiteralKind};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// A reference to an entity (e.g. a variable, function or type)
    /// See [`Token`] for more types of tokens
    Identifier(String),

    /// A keyword of the language (e.g. `return`)
    /// See [`Token`] for more types of tokens
    Keyword(Keyword),

    /// A literal value (e.g. 6, 0.1 or "hello")
    /// See [`Token`] for more types of tokens
    Literal {
        kind: LiteralKind,
        value: String,
    },

    LeftParen,
    RightParen,

    LeftCurly,
    RightCurly,

    LeftSquare,
    RightSquare,

    /// An ampersand symbol (`&`)
    /// See [`Token`] for more types of tokens
    Amp,

    /// A double ampersand symbol (`&&`)
    /// See [`Token`] for more types of tokens
    AndAnd,

    /// A pipe symbol (`|`) used for bitwise or
    /// See [`Token`] for more types of tokens
    Pipe,

    /// A double pipe symbol (`||`) used for logical or
    /// See [`Token`] for more types of tokens
    OrOr,

    /// An exclamation mark (`!`)
    /// See [`Token`] for more types of tokens
    Bang,

    /// A bang and an equals symbol (`!=`) used for inequality
    /// See [`Token`] for more types of tokens
    BangEq,

    /// An equals symbol (`=`) used for assignment
    /// See [`Token`] for more types of tokens
    Eq,

    /// A double equals symbol (`==`) used for equality
    /// See [`Token`] for more types of tokens
    EqEq,

    /// A period (`.`) symbol, used to access members of modules or objects
    /// See [`Token`] for more types of tokens
    Period,

    /// A comma (`,`) symbol that separates items in a sequence (e.g. args)
    /// See [`Token`] for more types of tokens
    Comma,

    /// A colon (`:`) symbol that indicates a type
    /// See [`Token`] for more types of tokens
    Colon,

    /// A semicolon (`;`) symbol that marks the end of a statement
    /// See [`Token`] for more types of tokens
    Semicolon,

    /// An asterisk (`*`) symbol, used for multiplication and dereferencing
    /// See [`Token`] for more types of tokens
    Asterisk,

    /// A slash (`/`) symbol, used for division
    /// See [`Token`] for more types of tokens
    Slash,

    /// A percent (`%`) symbol, used for modulo
    /// See [`Token`] for more types of tokens
    Percent,

    /// A plus (`+`) symbol, used for addition and indicating positivity
    /// See [`Token`] for more types of tokens
    Plus,

    /// A minus (`-`) symbol, used for subtraction and negation
    /// See [`Token`] for more types of tokens
    Minus,
}

#[derive(Debug, Error)]
pub enum MalformedLiteralError {
    #[error("Empty character")]
    EmptyChar,

    #[error("Invalid escape sequence: `\\{0}`")]
    InvalidEscapeSequence(String),

    #[error("Invalid numeric literal")]
    InvalidNumber(String),

    #[error("Multicharacter character literal: `{0}`")]
    MulticharLiteral(String),

    #[error("Unterminated character")]
    UnterminatedCharacter,
}

#[derive(Debug, Error)]
pub enum TokenizeError {
    #[error("Invalid punctuation: {0}")]
    InvalidPunctuation(String),

    #[error("Malformed literal: {0}")]
    MalformedLiteral(#[from] MalformedLiteralError),
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

const PUNCTUATION_CHARS: &str = "(){}[]&|!=.,;*/%+-:";

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            tokens: vec![],
            line: 1,
        }
    }

    fn match_char(&mut self, ch: char) -> Result<(), TokenizeError> {
        match ch {
            '0'..='9' | '-' => self.tokenize_number()?,
            '\'' => self.tokenize_character()?,
            '"' => self.tokenize_string()?,
            '\n' => {
                self.chars.next();
                self.line += 1;
            }
            c if c.is_whitespace() => {
                self.chars.next();
            }
            'A'..='Z' | 'a'..='z' => self.tokenize_identifier_or_keyword()?,
            ';' => {
                self.tokens.push(Token::Semicolon);
                self.chars.next();
            }
            c if PUNCTUATION_CHARS.contains(c) => self.tokenize_punctuation()?,
            _ => {
                panic!("Unknown symbol `{ch}`");
            }
        };

        Ok(())
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, QualifiedTokenizeError> {
        while let Some(&ch) = self.chars.peek() {
            self.match_char(ch).map_err(|e| QualifiedTokenizeError {
                tokenize_error: e,
                line: self.line,
            })?;
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
                _ => break,
            }
        }

        self.tokens.push(Token::Literal {
            kind: if period {
                LiteralKind::Float
            } else {
                LiteralKind::Integer
            },
            value: accumulator,
        });

        Ok(())
    }

    fn handle_escaped_seq(&mut self) -> Result<char, MalformedLiteralError> {
        let ch = self
            .chars
            .next()
            .ok_or(MalformedLiteralError::UnterminatedCharacter)?;

        Ok(match ch {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '\\' => '\\',
            '\'' => '\'',
            '\"' => '\"',
            '0' => '\0',
            'u' => {
                let bracket = self
                    .chars
                    .next()
                    .ok_or(MalformedLiteralError::UnterminatedCharacter)?;

                if bracket != '{' {
                    return Err(MalformedLiteralError::InvalidEscapeSequence(
                        bracket.to_string(),
                    ));
                }

                let mut hex = String::new();

                for ch in self.chars.by_ref() {
                    match ch {
                        'a'..='f' | 'A'..='F' | '0'..='9' => hex.push(ch),
                        '}' => break,
                        _ => {
                            return Err(MalformedLiteralError::InvalidEscapeSequence(format!(
                                "u{{{hex}{ch}"
                            )));
                        }
                    }
                }

                let num = u32::from_str_radix(&hex, 16).map_err(|_| {
                    MalformedLiteralError::InvalidEscapeSequence(format!("u{{{hex}}}"))
                })?;

                char::from_u32(num).ok_or_else(|| {
                    MalformedLiteralError::InvalidEscapeSequence(format!("u{{{hex}}}"))
                })?
            }
            _ => {
                return Err(MalformedLiteralError::InvalidEscapeSequence(ch.to_string()));
            }
        })
    }

    fn tokenize_character(&mut self) -> Result<(), MalformedLiteralError> {
        let mut escape = false;
        let mut chars = String::new();

        self.chars.next();

        while let Some(&ch) = self.chars.peek() {
            if escape {
                chars.push(self.handle_escaped_seq()?);
                escape = false;
            } else {
                match ch {
                    '\'' => break,
                    '\\' => escape = true,
                    '\n' => return Err(MalformedLiteralError::UnterminatedCharacter),
                    _ => chars.push(ch),
                }

                self.chars.next();
            }
        }

        self.chars.next();

        match chars.chars().count() {
            0 => return Err(MalformedLiteralError::EmptyChar),
            1 => self.tokens.push(Token::Literal {
                kind: LiteralKind::Char,
                value: chars,
            }),
            _ => return Err(MalformedLiteralError::MulticharLiteral(chars)),
        };

        Ok(())
    }

    fn tokenize_string(&mut self) -> Result<(), MalformedLiteralError> {
        let mut accumulator = String::new();
        let mut escape = false;

        self.chars.next();

        while let Some(&ch) = self.chars.peek() {
            match ch {
                '"' if !escape => {
                    break;
                }
                '\\' if !escape => {
                    escape = true;
                    self.chars.next();
                }
                _ if escape => {
                    accumulator.push(self.handle_escaped_seq()?);
                    escape = false;
                }
                _ => {
                    accumulator.push(ch);
                    self.chars.next();
                }
            }
        }

        self.chars.next();

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
                'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                    accumulator.push(ch);
                    self.chars.next();
                }
                _ => break,
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

    fn tokenize_punctuation(&mut self) -> Result<(), TokenizeError> {
        let first_char = self
            .chars
            .next()
            .ok_or(TokenizeError::InvalidPunctuation("".to_string()))?;

        if let Some(token) = match first_char {
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            '{' => Some(Token::LeftCurly),
            '}' => Some(Token::RightCurly),
            '[' => Some(Token::LeftSquare),
            ']' => Some(Token::RightSquare),
            _ => None,
        } {
            self.tokens.push(token);
            return Ok(());
        }

        let mut punctuation = String::from(first_char);

        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphanumeric()
                || ch.is_whitespace()
                || matches!(ch, '(' | ')' | '{' | '}' | '[' | ']')
            {
                break;
            }

            punctuation.push(ch);
            self.chars.next();
        }

        if punctuation == "//" {
            for ch in self.chars.by_ref() {
                if ch == '\n' {
                    break;
                }
            }

            return Ok(());
        }

        self.tokens.push(match punctuation.as_str() {
            "&" => Token::Amp,
            "&&" => Token::AndAnd,
            "|" => Token::Pipe,
            "||" => Token::OrOr,
            "!" => Token::Bang,
            "!=" => Token::BangEq,
            "=" => Token::Eq,
            "==" => Token::EqEq,
            "." => Token::Period,
            "," => Token::Comma,
            ":" => Token::Colon,
            ";" => Token::Semicolon,
            "*" => Token::Asterisk,
            "/" => Token::Slash,
            "%" => Token::Percent,
            "+" => Token::Plus,
            "-" => Token::Minus,
            _ => return Err(TokenizeError::InvalidPunctuation(punctuation.to_string())),
        });

        Ok(())
    }
}
