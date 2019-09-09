use crate::parser::{CssParser};
use crate::lexer::Token;
use crate::error::ParserError;
use crate::style_sheet::{Property, DataType};

// TODO make semicolons required except for the last property
// TODO skip property if there's a syntax error
pub struct PropertyParser<'a> {
    css_parser: &'a mut CssParser,
}

impl<'a> PropertyParser<'a> {
    pub fn new(css_parser: &'a mut CssParser) -> PropertyParser {
        PropertyParser { css_parser }
    }

    pub fn parse(&mut self) -> Vec<Property> {
        self.parse_properties()
    }

    fn parse_properties(&mut self) -> Vec<Property> {
        match self.css_parser.current_token(true) {
            Token::LeftBrace(..) => self.css_parser.try_next_token(true),
            Token::EOF => {
                return vec![]
            },
            token => {
                self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                    found: format!("{}", token),
                    expected: Some("{".to_string()),
                    context: None,
                });
                token
            },
        };

        let mut properties: Vec<Property> = vec![];
        loop {
            let name = match self.css_parser.current_token(true) {
                Token::Identifier(value) => value.to_string(),
                t => {
                    self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                        found: format!("{}", t),
                        expected: Some("identifier".to_string()),
                        context: None,
                    });
                    continue;
                },
            };

            match self.css_parser.try_next_token(true) {
                Token::Colon(..) => self.css_parser.try_next_token(true),
                token => {
                    self.css_parser.error_handler.flag(&ParserError::UnexpectedToken {
                        found: format!("{}", token),
                        expected: Some(":".to_string()),
                        context: None,
                    });
                    token
                },
            };

            let value = match self.css_parser.current_token(true) {
                Token::Identifier(val) | Token::String(val) => {
                    DataType::Keyword(val)
                },
                _ => DataType::Keyword("don't know yet!".to_string()),
            };

            properties.push(Property { name, value });

            if let Token::Semicolon(..) = self.css_parser.try_next_token(true) {
                self.css_parser.try_next_token(true);
            }

            if let Token::RightBrace(..) = self.css_parser.current_token(true) {
                self.css_parser.try_next_token(true);
                break;
            }
        }

        properties
    }
}
