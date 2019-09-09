use std::fmt;
use std::error;

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum ParserError {
    // Fatal(String),
    InvalidNumber(String),
    UnknownToken(String),
    UnexpectedToken {
        found: String,
        expected: Option<String>,
        context: Option<String>,
    },
}

impl error::Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::UnexpectedToken { found, context, expected } => {
                let context_string = match context {
                    None => String::new(),
                    Some(c) => format!(" found in {}", c),
                };

                let expected_string = match expected {
                    None => String::new(),
                    Some(e) => format!(", expected `{}`", e),
                };

                write!(f, "Unexpected token{} `{}`{}", context_string, found, expected_string)
            },
            ParserError::UnknownToken(message) => write!(f, "Unknown token `{}`", message),
            // ParserError::Fatal(message) => write!(f, "Fatal error {}", message),
            ParserError::InvalidNumber(message) => write!(f, "Number parse error: '{}'", message),
        }
    }
}

#[derive(Debug)]
pub struct ErrorHandler {
    // TODO make private
    pub errors: Vec<ParserError>,
}

impl ErrorHandler {
    pub fn new() -> ErrorHandler {
        ErrorHandler {
            errors: vec![],
        }
    }
    pub fn flag(&mut self, error: &ParserError) -> () {
        self.errors.push(error.clone());
    }
}
