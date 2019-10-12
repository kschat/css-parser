use crate::lexer::{Token, CssLexer};
use crate::error::{ErrorHandler, ParserError};
use crate::selector_parser::{SelectorParser, is_token_selector};
use crate::property_parser::PropertyParser;
use crate::style_sheet::{StyleSheet, Rule};

use std::io::{BufRead};
use std::result;

pub type Result<T> = result::Result<T, ParserError>;

#[derive(Debug)]
pub struct CssParser<T: BufRead> {
    lexer: CssLexer<T>,
    pub(crate) error_handler: ErrorHandler,
}

impl<T: BufRead> CssParser<T> {
    pub fn new(lexer: CssLexer<T>) -> CssParser<T> {
        CssParser {
            lexer,
            error_handler: ErrorHandler::new(),
        }
    }

    pub fn parse(&mut self) -> StyleSheet {
        let (rules, errors): (Vec<_>, Vec<_>) = self.parse_rules()
            .into_iter()
            .partition(Result::is_ok);

        StyleSheet {
            rules: rules.into_iter().map(Result::unwrap).collect(),
            errors: errors.into_iter().map(Result::unwrap_err).collect(),
        }
    }

    fn parse_rules(&mut self) -> Vec<Result<Rule>> {
        let mut rules: Vec<Result<Rule>> = vec![];

        loop {
            match self.current_token(true) {
                Token::EOF => break,
                _ => rules.push(self.parse_rule()),
            };
        }

        rules
    }

    fn parse_rule(&mut self) -> Result<Rule> {
        let selectors = match (SelectorParser::new(self).parse(), self.current_token(true)) {
            (_, Token::EOF) => return Err(ParserError::UnexpectedEOF),
            (selectors, _) => selectors,
        };

        Ok(Rule {
            selectors,
            properties: PropertyParser::new(self).parse(),
        })
    }

    pub(crate) fn try_next_token(&mut self, skip_whitespace: bool) -> Token {
        let token = self.lexer.next_token();
        match token {
            Token::Error(error) => {
                self.error_handler.flag(error);
                token
            },
            Token::Whitespace(..) if skip_whitespace => {
                self.skip_whitespace_tokens();
                self.lexer.current_token()
            },
            _ => token,
        }.clone()
    }

    pub(crate) fn current_token(&mut self, skip_whitespace: bool) -> Token {
        if skip_whitespace {
            self.skip_whitespace_tokens();
        }
        self.lexer.current_token().clone()
    }

    fn skip_whitespace_tokens(&mut self) -> () {
        loop {
            match self.lexer.current_token() {
                Token::Whitespace(..) => {
                    self.lexer.next_token();
                    continue
                },
                _ => return,
            }
        }
    }
}
