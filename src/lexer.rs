use crate::source::{Source};
use crate::error::ParserError;

use std::io::{BufRead};
use std::fmt;

// TODO change values to char where appropriate
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    Function(String),

    // Hash(String),
    // AtKeyword(String),

    Url(String),
    BadUrl(String),

    String(String),
    // BadString(String),
    Integer(i32), // TODO replace Integer and Float with Number
    Float(f32),

    // Number { int_value: i32, float_value: f32 },
    // Percentage(String),
    // Dimension(String),

    // Cdo(String),
    // Cdc(String),

    Error(ParserError), // TODO remove

    // TODO no need to have a value for constant tokens
    // symbols
    Whitespace(String),
    Comma(String),
    Colon(String),
    Semicolon(String),

    // Delim(String),

    LeftBrace(String),
    RightBrace(String),

    LeftBracket(String),
    RightBracket(String),

    LeftParen(String),
    RightParen(String),

    SingleQuote(String),
    DoubleQuote(String),
    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct CssLexer<T: BufRead> {
    source: Source<T>,
    current: Option<Token>,
}

// TODO return iterator
impl<T: BufRead> CssLexer<T> {
    pub fn new(source: Source<T>) -> CssLexer<T> {
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

        if self.is_start_of_name() {
            return self.extract_name();
        }

        match current_char {
            '.' | '#' | '*' | '{' | '}' | '[' | ']' | '(' | ')' | ';' | ':' | ',' => self.extract_symbol(current_char),
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

    fn extract_symbol(&mut self, current_char: char) -> Token {
        let value = current_char.to_string();
        self.source.next_character();

        match current_char {
            // '.' => Token::Dot(value),
            // '#' => Token::Pound(value),
            // '*' => Token::Star(value),
            ',' => Token::Comma(value),
            ':' => Token::Colon(value),
            ';' => Token::Semicolon(value),
            '{' => Token::LeftBrace(value),
            '}' => Token::RightBrace(value),
            '[' => Token::LeftBracket(value),
            ']' => Token::RightBracket(value),
            '(' => Token::LeftParen(value),
            ')' => Token::RightParen(value),
            '"' => Token::DoubleQuote(value),
            '\'' => Token::SingleQuote(value),
            _ => return Token::Error(ParserError::UnknownToken(value)),
        }
    }

    fn is_escape(&mut self) -> bool {
        match self.source.current_character() {
            Some('\\') => match self.source.peek_character() {
                None => false,
                _ => true,
            },
            _ => false,
        }
    }

    fn is_escape_at(&mut self, start_from: usize) -> bool {
        match self.source.peek_n_character(start_from) {
            Some('\\') => match self.source.peek_n_character(start_from + 1) {
                None => false,
                _ => true,
            },
            _ => false,
        }
    }

    fn is_start_of_name(&mut self) -> bool {
        match self.source.current_character() {
            Some('-') => match self.source.peek_character() {
                Some('-') => true,
                Some(c) => c.is_alphabetic() || c == '_' || self.is_escape_at(1),
                None => false,
            },
            Some(c) => c.is_alphabetic() || c == '_',
            None => false,
        }
    }

    fn extract_name(&mut self) -> Token {
        let value = self.extract_while(is_word_part);

        match self.source.current_character() {
            Some('(') => {
                self.source.next_character();
                if !value.eq_ignore_ascii_case("url") {
                    return Token::Function(value);
                }

                // consume the next token while the next two tokens are whitespace
                let next = match self.while_next_two_whitespace() {
                    Some(c) if c.is_whitespace() => self.source.peek_character(),
                    n => n,
                };

                match next {
                    // URLs with a string parameter are parsed as functions
                    Some('\'') | Some('"') => Token::Function(value),
                    _ => self.extract_url(),
                }
            },
            _ => Token::Identifier(value),
        }
    }

    // TODO return Err for BadUrl per spec
    fn extract_url(&mut self) -> Token {
        let mut value = String::new();
        self.extract_whitespace();

        // TODO clean up
        loop {
            match self.source.current_character() {
                Some(')') | None => {
                    self.source.next_character();
                    return Token::Url(value);
                },
                Some(c) if c.is_whitespace() => {
                    self.extract_whitespace();
                    match self.source.current_character() {
                        Some(')') | None => {
                            self.source.next_character();
                            return Token::Url(value);
                        },
                        _ => return self.consume_bad_url(value),
                    }
                },
                // TODO handle non-printable code points
                Some('"') | Some('\'') | Some('(') => return self.consume_bad_url(value),
                // TODO handle escapes
                Some(c) => {
                    self.source.next_character();
                    value.push(c);
                },
            }
        }
    }

    fn consume_bad_url(&mut self, value: String) -> Token {
        while let Some(c) = self.source.current_character() {
            self.source.next_character();
            if c == ')' {
                break;
            }

            if self.is_escape() {
                // TODO consume escape
            }
        }

        Token::BadUrl(value)
    }

    fn while_next_two_whitespace(&mut self) -> Option<char> {
        loop {
            let current = self.source.current_character()?;
            // peek will return None if we're at the EOL, which is whitespace
            let next = self.source.peek_character().or(Some('\n'))?;

            if current.is_whitespace() && next.is_whitespace() {
                self.source.next_character();
                continue;
            }

            return Some(current)
        }
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

    fn extract_while<U>(&mut self, predicate: U) -> String
        where U: Fn(char) -> bool
    {
        let mut value = String::new();

        while let Some(current) = self.source.current_character() {
            match predicate(current) {
                true => {
                    self.source.next_character();
                    value.push(current);
                },
                false => break,
            }
        }

        value
    }
}

// TODO fix to work with unicode
fn is_word_part(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}
