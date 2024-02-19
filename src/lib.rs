#[cfg(test)]
mod tests;

use core::fmt;
use std::{
    env::Args, fs::File, io::{self, Read}, path::PathBuf
};

pub fn read_file_to_bytes(filepath: PathBuf) -> io::Result<Vec<u8>> {
    let mut file = File::open(filepath)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    Ok(data)
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum TokenKind {
    Whitespace,
    Invalid,
    Null,
    Int,
    Float,
    Identifier,
    Ponct,
    Op,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub loc: (usize, usize),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} {:?} -> {}",
            self.loc.0, self.loc.1, self.kind, self.value
        )
    }
}

impl Token {
    fn new(kind: TokenKind, value: String, loc: (usize, usize)) -> Self {
        Self { kind, value, loc }
    }

    fn invalid(value: String, loc: (usize, usize)) -> Self {
        Self::new(TokenKind::Invalid, value, loc)
    }

    fn null(loc: (usize, usize)) -> Self {
        Self::new(TokenKind::Null, String::new(), loc)
    }

    pub fn empty() -> Self {
        Self::new(TokenKind::Null, String::new(), (0, 0))
    }

    pub fn fmt_loc(&self) -> String {
        format!("{}:{}", self.loc.0, self.loc.1)
    }

    pub fn fmt_kind(&self) -> String {
        format!("{:?}", self.kind)
    }

    pub fn fmt_value(&self) -> String {
        format!("{}", self.value)
    }
}

pub struct Lexer {
    input: Vec<u8>,

    max_position: usize,

    position: usize,
    col: usize,
    row: usize,
}

impl Lexer {
    pub fn new(input: Vec<u8>) -> Self {
        let max = input.len();
        Self {
            input,
            max_position: max,
            position: 0,
            col: 1,
            row: 1,
        }
    }
    pub fn from_args(args: Args) -> Self {
        if let Some(s) = args.reduce(|acc, a| format!("{} {}", acc, a)) {
            Self::new(s.into_bytes())
        } else {
            Self::new(vec![])
        }
    }

    fn current_byte(&self) -> u8 {
        if self.has_next() {
            self.input[self.position]
        } else {
            0
        }
    }

    fn next_byte(&self) -> u8 {
        self.peek(1)
    }

    fn peek(&self, offset: usize) -> u8 {
        let index = self.position + offset;

        if index < self.max_position {
            self.input[index]
        } else {
            0
        }
    }

    fn advance_char(&mut self) {
        self.position += 1;
        self.col += 1;
    }

    fn has_next(&self) -> bool {
        self.position < self.max_position
    }

    pub fn next(&mut self) -> Token {
        match self.current_byte() {
            ch if ch.is_ascii_alphabetic() || ch == b'_' => {
                self.identifier(self.position)
            }
            ch if ch.is_ascii_digit() => self.number(),
            ch if ch.is_ascii_punctuation() => {
                self.position += 1;
                Token::new(
                    TokenKind::Ponct,
                    format!("{}", ch as char),
                    (self.row, self.col),
                )
            }
            ch if ch.is_ascii_whitespace() => self.whitespace(),
            _ => {
                if self.has_next() {
                    self.invalid(self.position, self.position + 1)
                } else {
                    self.null()
                }
            }
        }
    }

    fn whitespace(&mut self) -> Token {
        let start = self.position;

        while self.has_next() {
            match self.current_byte() {
                b' ' | b'\t' | b'\r' => self.advance_char(),
                b'\n' => {
                    self.row += 1;
                    self.advance_char();
                }
                _ => break,
            }
        }

        let value = self.slice_string(start, self.position);

        Token::new(TokenKind::Whitespace, value, (self.row, self.col))
    }

    fn number(&mut self) -> Token {
        let start = self.position;

        let mut kind = TokenKind::Int;

        loop {
            match self.current_byte() {
                b'0'..=b'9' => {}
                b'.' if (b'0'..=b'9').contains(&self.next_byte()) => {
                    kind = TokenKind::Float;
                }
                _ => break,
            }

            self.position += 1;
        }

        self.token(kind, start)
    }

    fn identifier(&mut self, start: usize) -> Token {
        loop {
            let ch = self.current_byte();
            if !(ch.is_ascii_alphabetic() || ch == b'_') {
                break;
            }
            self.position += 1
        }

        self.token(TokenKind::Identifier, start)
    }

    fn token(&mut self, kind: TokenKind, start: usize) -> Token {
        let value = self.slice_string(start, self.position);
        Token::new(kind, value, (self.row, self.col))
    }

    fn slice_string(&mut self, start: usize, stop: usize) -> String {
        String::from_utf8_lossy(&self.input[start..stop]).into_owned()
    }

    fn invalid(&mut self, start: usize, stop: usize) -> Token {
        let value = self.slice_string(start, stop);

        self.position = self.max_position;

        Token::invalid(value, (self.row, self.col))
    }

    fn null(&self) -> Token {
        Token::null((self.row, self.col))
    }
}
