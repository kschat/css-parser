use std::io::BufReader;
use std::fs::File;

use crate::source::{Source};
use crate::lexer::{CssLexer};
use crate::parser::{CssParser};
use crate::style_sheet::{StyleSheet};

pub mod lexer;
mod error;
mod selector_parser;
mod property_parser;
mod parser;
mod style_sheet;
mod source;

pub fn parse_file(path: &str) -> std::io::Result<StyleSheet> {
  let file = File::open(path)?;
  let source = Source::new(BufReader::new(file));
  let lexer = CssLexer::new(source);

  Ok(CssParser::new(lexer).parse())
}

pub fn lex_file(path: &str) -> std::io::Result<CssLexer<BufReader<File>>> {
  let file = File::open(path)?;
  let source = Source::new(BufReader::new(file));
  let lexer = CssLexer::new(source);

  Ok(lexer)
}