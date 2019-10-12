use css_parser::lex_file;
use css_parser::parse_file;
use css_parser::lexer::Token;

use std::env;

type Result = std::result::Result<(), std::io::Error>;

fn parse(path: &str) -> Result {
  let style_sheet = parse_file(path)?;

  for rule in style_sheet.rules {
    println!("Selector list: [");
    for selector_group in rule.selectors {
      println!("  Selector group: ({:?}) [", selector_group.specificity());
      for selector in selector_group.0 {
        println!("    {:?}: {:?}", selector, selector.specificity());
      }
      println!("  ]");
    }

    println!("]");
    println!("Properties: [");

    for property in rule.properties {
      println!("  {:?}", property);
    }

    println!("]");
    println!(" ");
  }

  Ok(())
}

fn lex(path: &str) -> Result {
  let mut lexer = lex_file(path)?;

  loop {
    let token = lexer.next_token();
    println!("{:?}", token);
    if *token == Token::EOF {
      break;
    }
  }

  Ok(())
}

fn main() -> Result {
  let path: &'static str = "./data/test.css";
  match env::args().nth(1).as_ref().map(|s| &s[..]) {
    Some("lex") => lex(path),
    Some("parse") | None => parse(path),
    Some(arg) => panic!("Unknown argument {}", arg),
  }
}
