use crate::parser::{CssParser};
use crate::lexer::Token;
use crate::error::ParserError;
use crate::style_sheet::{SelectorGroup, Selector};

use std::io::BufRead;

// TODO inline maybe
pub fn is_token_selector(token: &Token) -> bool {
    match token {
        // Token::Identifier(..) | Token::Star(..) | Token::IdSelector(..) | Token::ClassSelector(..) => true,
        Token::LeftBrace(..) | Token::RightBrace(..) => false,
        Token::LeftBracket(..) | Token::RightBracket(..) => false,
        Token::LeftParen(..) | Token::RightParen(..) => false,
        Token::EOF => false,
        _ => true,
    }
}

// Selector Grammar:
// selector_list := selector_group[,selector_group..] {
// selector_group := selector[ selector..]
// selector := *|[.#]word
pub struct SelectorParser<'a, T: BufRead> {
    // TODO had to make some fns and fields public, find alternative
    css_parser: &'a mut CssParser<T>,
}

impl<'a, T: BufRead> SelectorParser<'a, T> {
    pub fn new(css_parser: &'a mut CssParser<T>) -> SelectorParser<'a, T> {
        SelectorParser { css_parser }
    }

    pub fn parse(&mut self) -> Vec<SelectorGroup> {
        self.parse_selectors()
    }

    fn parse_selectors(&mut self) -> Vec<SelectorGroup> {
        let current = self.css_parser.current_token(true);
        if !is_token_selector(&current) {
            self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                found: current.to_string(),
                expected: Some("identifier".to_string()),
                context: Some("selector".to_string()),
            });
        }

        let selectors = self.parse_selector_list();

        match self.css_parser.current_token(true) {
            Token::LeftBrace(..) => selectors,

            t => {
                self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                    found: t.to_string(),
                    expected: Some("{".to_string()),
                    context: Some("selector".to_string()),
                });
                selectors
            },
        }
    }

    fn parse_selector_list(&mut self) -> Vec<SelectorGroup> {
        let mut selectors: Vec<SelectorGroup> = vec![self.parse_selector_group()];

        loop {
            match self.css_parser.current_token(true) {
                Token::LeftBrace(..) => return selectors,

                Token::EOF => {
                    self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                        found: "EOF".to_string(),
                        expected: None,
                        context: Some("selector".to_string()),
                    });

                    return selectors
                },

                Token::Comma(..) => {
                    self.css_parser.try_next_token(true);
                    selectors.push(self.parse_selector_group())
                },

                _ => selectors.push(self.parse_selector_group()),
            }
        }
    }

    fn parse_selector_group(&mut self) -> SelectorGroup {
        let current = self.css_parser.current_token(true);
        if !is_token_selector(&current) {
            self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                found: current.to_string(),
                expected: Some("identifier".to_string()),
                context: Some("selector".to_string()),
            });
        }

        let mut selectors: Vec<Selector> = vec![self.parse_selector()];

        SelectorGroup(loop {
            match self.css_parser.current_token(false) {
                Token::Comma(..) => break selectors,

                Token::Whitespace(..) => match self.css_parser.try_next_token(true) {
                    Token::LeftBrace(..) => break selectors,
                    _ => selectors.push(self.parse_selector()),
                },

                Token::EOF => {
                    self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                        found: "EOF".to_string(),
                        expected: None,
                        context: Some("selector".to_string()),
                    });

                    break selectors
                },

                token => {
                    self.css_parser.try_next_token(false);
                    self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                        found: token.to_string(),
                        expected: None,
                        context: Some("selector".to_string()),
                    });

                    continue
                },
            }
        })
    }

    fn parse_selector(&mut self) -> Selector {
        let mut selector = Selector {
            id: None,
            tag_name: None,
            class_names: vec![],
        };

        loop {
            match self.css_parser.current_token(false) {
                /*
                Token::IdSelector(val) => {
                    selector.id = Some(val);
                    self.css_parser.try_next_token(false);
                },

                Token::ClassSelector(val) => {
                    selector.class_names.push(val);
                    self.css_parser.try_next_token(false);
                },
                */

                Token::Identifier(val) => {
                    selector.tag_name = Some(val);
                    self.css_parser.try_next_token(false);
                },

                Token::LeftBrace(..) | Token::Comma(..) | Token::Whitespace(..) => break,

                Token::EOF => {
                    self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                        found: "EOF".to_string(),
                        expected: None,
                        context: Some("selector".to_string()),
                    });

                    break
                },

                t => {
                    self.css_parser.try_next_token(false);
                    self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                        found: t.to_string(),
                        expected: None,
                        context: Some("selector".to_string()),
                    });
                },
            }
        }

        selector
    }
}
