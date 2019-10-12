use std::io::{BufRead};

pub const EOF: &str = "";

#[derive(Debug)]
enum State {
    Init,
    Reading {
        line: String,
        position: usize,
    },
    Done,
}

impl State {
    pub fn eol(&self) -> bool {
        match self {
            State::Init => false,
            State::Done => true,
            // `- 1` is a terrible hack, should be fixed
            State::Reading { line, position } => *position >= line.len() - 1,
        }
    }
}

#[derive(Debug)]
pub struct Source<T: BufRead> {
    state: State,
    reader: T,
}

impl<T: BufRead> Source<T> {
    pub fn new(reader: T) -> Source<T> {
        Source {
            reader,
            state: State::Init,
        }
    }

    pub fn current_character(&mut self) -> Option<char> {
        match &self.state {
            State::Done => None,
            State::Init => match self.read_line() {
                Some(ref mut line) => {
                    self.state = State::Reading {
                        position: 0,
                        line: line.to_string()
                    };
                    line.chars().next()
                },
                None => {
                    self.state = State::Done;
                    return None
                },
            },
            State::Reading { position, line } => line[*position..]
                .chars()
                .next(),
        }
    }

    pub fn next_character(&mut self) -> Option<char> {
        match &self.state {
            State::Init => self.current_character(),
            State::Done => None,
            State::Reading { line, position } => {
                if self.state.eol() {
                    return self.next_line_character();
                }

                let chars = line[*position..].char_indices();
                let (offset, current) = chars.skip(1).next().unwrap();

                self.state = State::Reading {
                    position: position + offset,
                    line: line.to_string(),
                };

                Some(current)
            },
        }
    }

    pub fn peek_character(&mut self) -> Option<char> {
        self.peek_n_character(1)
    }

    // TODO `peek_n_character` is not safe and could fail with unicode. Replace
    // once this is updated to support unicode correctly.
    pub fn peek_n_character(&mut self, n: usize) -> Option<char> {
        match &self.state {
            State::Done => None,
            State::Init => {
                self.current_character();
                self.peek_character()
            },
            State::Reading { line, position } => {
                if self.state.eol() {
                    return None;
                }

                let chars = line[*position..].char_indices();
                let (_, current) = chars.skip(n).next()?;

                Some(current)
            },
        }
    }

    fn next_line_character(&mut self) -> Option<char> {
        match self.read_line() {
            Some(line) => {
                self.state = State::Reading { line, position: 0 };
                self.current_character()
            },
            None => {
                self.state = State::Done;
                None
            },
        }
    }

    fn read_line(&mut self) -> Option<String> {
        match self.state {
            State::Done => Some(EOF.to_string()),
            State::Init | State::Reading { .. } => read_line_from_buffer(&mut self.reader),
        }
    }
}

fn read_line_from_buffer<T: BufRead>(reader: &mut T) -> Option<String> {
    let mut line = String::new();
    let len = reader.read_line(&mut line).expect("Failed to read line");
    if len == 0 {
        return None
    }

    Some(line)
}
