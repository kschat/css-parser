use crate::lexer::{Token, CssLexer};
use crate::error::{ErrorHandler, ParserError};
use crate::selector_parser::{SelectorParser, is_token_selector};
use crate::property_parser::PropertyParser;
use crate::style_sheet::{StyleSheet, Rule};

#[derive(Debug)]
pub struct CssParser {
    lexer: CssLexer,
    pub(crate) error_handler: ErrorHandler,
}

impl CssParser {
    pub fn new(lexer: CssLexer) -> CssParser {
        CssParser {
            lexer,
            error_handler: ErrorHandler::new(),
        }
    }

    pub fn parse(&mut self) -> StyleSheet {
        let mut rules: Vec<Rule> = vec![];

        loop {
            let token = self.current_token(true);
            if is_token_selector(&token) {
                rules.push(self.parse_rule());
                if Token::EOF == self.current_token(true) {
                    break;
                }
            } else {
                self.error_handler.flag(&ParserError::UnexpectedToken {
                    found: token.to_string(),
                    expected: None,
                    context: None,
                });
            }
        };

        StyleSheet { rules }
    }

    fn parse_rule(&mut self) -> Rule {
        let selectors = SelectorParser::new(self).parse();
        Rule {
            selectors,
            properties: PropertyParser::new(self).parse(),
        }
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
