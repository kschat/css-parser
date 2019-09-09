use crate::source::{Source};
use crate::error::ParserError;

use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    ClassSelector(String),
    IdSelector(String),
    Identifier(String),
    String(String),
    Integer(i32),
    Float(f32),
    EOF,
    Error(ParserError),
    // Percentage(f32, String), // not supported yet

    // symbols
    Dot(String),
    Pound(String),
    Star(String),
    Comma(String),
    Colon(String),
    Semicolon(String),
    LeftBrace(String),
    RightBrace(String),
    SingleQuote(String),
    DoubleQuote(String),
    Minus(String),
    Whitespace(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct CssLexer {
    source: Source,
    current: Option<Token>,
}

// TODO return iterator
impl CssLexer {
    pub fn new(source: Source) -> CssLexer {
        CssLexer { source, current: None }
    }

    pub fn current_token(&mut self) -> &Token {
        match self.current {
            None => {
                let token = self.extract_token();
                self.current = Some(token);
                self.current.as_ref().unwrap()
            },
            _ => self.current.as_ref().unwrap(),
        }
    }

    pub fn next_token(&mut self) -> &Token {
        self.current = None;
        self.current_token()
    }

    fn extract_token(&mut self) -> Token {
        let current_char = match self.skip_comments() {
            None => return Token::EOF,
            Some(c) => c,
        };

        if current_char.is_whitespace() || current_char == '\n' {
            return self.extract_whitespace();
        }

        match current_char {
            '.' | '#' => self.extract_selector_or_symbol(current_char),
            '*' | '{' | '}' | ';' | ':' | ',' => self.extract_symbol(current_char),
            '\'' | '"' => self.extract_string(current_char),
            '0' ... '9' => self.extract_number(),
            'a' ... 'z' => self.extract_word(),
            c => {
                self.source.next_character();
                Token::Error(ParserError::UnexpectedToken {
                    found: c.to_string(),
                    context: None,
                    expected: None,
                })
            }
        }
    }

    fn extract_selector_or_symbol(&mut self, current_char: char) -> Token {
        match self.source.peek_character() {
            Some(next) if is_start_of_word(next) => {
                let selector = self.extract_while(|c| !is_start_of_selector(c) && is_word_part(c));
                match current_char {
                    '#' => Token::IdSelector(selector),
                    '.' => Token::ClassSelector(selector),
                    _ => return Token::Error(ParserError::UnexpectedToken {
                        found: current_char.to_string(),
                        expected: Some("`#` or `.`".to_string()),
                        context: None,
                    }),
                }
                /*
                Token1::Word(
                    self.extract_while(|c| !is_start_of_selector(c) && is_word_part(c)),
                    match current_char {
                        '#' => TokenKind::IdSelector,
                        '.' => TokenKind::ClassSelector,
                        _ => return Token1::Error(ParserError::UnexpectedToken {
                            found: current_char.to_string(),
                            expected: Some("`#` or `.`".to_string()),
                            context: None,
                        }),
                    },
                )
                */
            },
            _ => self.extract_symbol(current_char),
        }
    }

    fn extract_symbol(&mut self, current_char: char) -> Token {
        let value = current_char.to_string();
        self.source.next_character();

        match current_char {
            '.' => Token::Dot(value),
            '#' => Token::Pound(value),
            '*' => Token::Star(value),
            ',' => Token::Comma(value),
            ':' => Token::Colon(value),
            ';' => Token::Semicolon(value),
            '{' => Token::LeftBrace(value),
            '}' => Token::RightBrace(value),
            '-' => Token::Minus(value),
            '"' => Token::DoubleQuote(value),
            '\'' => Token::SingleQuote(value),
            _ => return Token::Error(ParserError::UnknownToken(value)),
        }
        /*
        Token1::Symbol(current_char.to_string(), match current_char {
            '.' => TokenKind::Dot,
            '#' => TokenKind::Pound,
            '*' => TokenKind::Star,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '\'' => TokenKind::SingleQuote,
            '"' => TokenKind::DoubleQuote,
            '-' => TokenKind::Minus,
            c => return Token1::Error(ParserError::UnknownToken(c.to_string())),
        })
        */
    }

    // TODO handle quotes in strings
    fn extract_string(&mut self, delimiter: char) -> Token {
        self.source.next_character();
        let value = self.extract_while(|c| c != delimiter);

        match self.source.current_character() {
            // TODO clean up repeated code
            None => return Token::Error(ParserError::UnexpectedToken {
                found: "EOF".to_string(),
                expected: Some(delimiter.to_string()),
                context: None,
            }),
            Some(found) if found != delimiter => return Token::Error(ParserError::UnexpectedToken {
                found: found.to_string(),
                expected: Some(delimiter.to_string()),
                context: None,
            }),
            _ => {
                self.source.next_character();
                Token::String(value)
            },
        }
    }

    fn extract_number(&mut self) -> Token {
        let lhs = self.extract_digits();

        match self.extract_fraction_digits() {
            Some(rhs) => match format!("{}{}", lhs, rhs).parse::<f32>() {
                Ok(value) => Token::Float(value),
                Err(e) => Token::Error(ParserError::InvalidNumber(e.to_string())),
            },
            _ => match lhs.parse::<i32>() {
                Ok(value) => Token::Integer(value),
                Err(e) => Token::Error(ParserError::InvalidNumber(e.to_string())),
            },
        }
    }

    fn extract_digits(&mut self) -> String {
        self.extract_while(|c| match c {
            '0' ... '9' => true,
            _ => false,
        })
    }

    fn extract_fraction_digits(&mut self) -> Option<String> {
        let current = self.source.current_character()?;
        let next = self.source.peek_character()?;

        if current != '.' || !next.is_digit(10) {
            return None;
        }

        Some(self.extract_digits())
    }

    fn extract_word(&mut self) -> Token {
        Token::Identifier(
            self.extract_while(is_word_part)
        )
    }

    fn extract_whitespace(&mut self) -> Token {
        Token::Whitespace(
            self.extract_while(|c| c.is_whitespace())
        )
    }

    fn skip_comments(&mut self) -> Option<char> {
        loop {
            let next = self.skip_comment()?;
            if !self.is_comment_start_next() {
                return Some(next);
            }
        }
    }

    fn skip_comment(&mut self) -> Option<char> {
        if !self.is_comment_start_next() {
            return self.source.current_character();
        }

        // consume '*'
        self.source.next_character()?;

        while let Some(_) = self.source.next_character() {
            if self.is_comment_end_next() {
                self.source.next_character(); // consume '*'
                return self.source.next_character(); // consume '/' and return next value
            }
        }

        None
    }

    fn is_comment_start_next(&mut self) -> bool {
        if let Some(current) = self.source.current_character() {
            if let Some(next) = self.source.peek_character() {
                return format!("{}{}", current, next) == "/*".to_string();
            }
        }

        false
    }

    fn is_comment_end_next(&mut self) -> bool {
        let current = match self.source.current_character() {
            None => return false,
            Some(v) => v,
        };

        let next = match self.source.peek_character() {
            Some(v) => v,
            None => return false,
        };

        format!("{}{}", current, next) == "*/".to_string()
    }

    fn extract_while<T>(&mut self, predicate: T) -> String
        where T: Fn(char) -> bool
    {
        let mut value = self.source.current_character().unwrap().to_string();
        while let Some(current) = self.source.next_character() {
            match predicate(current) {
                true => value.push(current),
                false => break,
            }
        }

        value
    }
}

fn is_word_part(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

fn is_start_of_word(c: char) -> bool {
    c.is_alphabetic() || c == '-' || c == '_'
}

fn is_start_of_selector(c: char) -> bool {
    c == '.' || c == '#'
}
