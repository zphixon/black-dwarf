//! TOML subset parser
//!
//! Supports multi-line table literals trailing commas everywhere. Because why the hell was that
//! not a part of TOML in the first place? C'mon Tom. That's not obvious.

use crate::BlackDwarfError;
use indexmap::IndexMap;
use std::collections::VecDeque;

pub enum Value<'doc> {
    Table {
        key_values: IndexMap<&'doc str, Value<'doc>>,
        pos: Pos,
    },

    Array {
        values: Vec<Value<'doc>>,
        pos: Pos,
    },

    String {
        value: &'doc str,
        pos: Pos,
    },

    Integer {
        value: i64,
        pos: Pos,
    },

    Float {
        value: f64,
        pos: Pos,
    },

    Boolean {
        value: bool,
        pos: Pos,
    },
}

impl<'doc> Value<'doc> {
    pub fn new(pos: Pos) -> Self {
        Value::Table {
            key_values: IndexMap::new(),
            pos,
        }
    }

    pub fn type_str(&self) -> &'static str {
        match self {
            Value::Array { .. } => "array",
            Value::Table { .. } => "table",
            Value::String { .. } => "string",
            Value::Integer { .. } => "integer",
            Value::Float { .. } => "float",
            Value::Boolean { .. } => "boolean",
        }
    }

    pub fn pos(&self) -> Pos {
        match self {
            Value::Array { pos, .. } => *pos,
            Value::Table { pos, .. } => *pos,
            Value::String { pos, .. } => *pos,
            Value::Integer { pos, .. } => *pos,
            Value::Float { pos, .. } => *pos,
            Value::Boolean { pos, .. } => *pos,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value<'doc>> {
        match self {
            Value::Table { key_values, .. } => key_values.get_mut(key),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value<'doc>> {
        match self {
            Value::Table { key_values, .. } => key_values.get(key),
            _ => None,
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        match self {
            Value::Table { key_values, .. } => key_values.contains_key(key),
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        matches!(self, Value::String { .. })
    }

    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table { .. })
    }

    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::Array { values, .. } => Some(&values),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&'doc str> {
        match self {
            Value::String { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn iter_kvs(&self) -> TableIter {
        TableIter {
            inner: match self {
                Value::Table { key_values, .. } => Some(key_values.iter()),
                _ => None,
            },
        }
    }

    fn insert(&mut self, key: &'doc str, value: Value<'doc>) {
        match self {
            Value::Table { key_values, .. } => {
                key_values.insert(key, value);
            }
            _ => panic!("called create on non-table"),
        }
    }
}

impl std::fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Table { key_values, .. } => f.debug_map().entries(key_values.iter()).finish(),
            Value::Array { values, .. } => f.debug_list().entries(values.iter()).finish(),
            Value::String { value, .. } => value.fmt(f),
            Value::Integer { value, .. } => value.fmt(f),
            Value::Float { value, .. } => value.fmt(f),
            Value::Boolean { value, .. } => value.fmt(f),
        }
    }
}

// memory safety 👍
pub struct TableIter<'table, 'doc: 'table> {
    inner: Option<indexmap::map::Iter<'table, &'doc str, Value<'doc>>>,
}

impl<'table, 'doc: 'table> Iterator for TableIter<'table, 'doc> {
    type Item = (&'doc str, &'table Value<'doc>);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next().map(|(k, v)| (*k, v))
    }
}

pub fn scan(doc: &str) -> Result<Vec<Token>, BlackDwarfError> {
    Scanner::new(doc).scan_all()
}

pub fn parse(doc: &str) -> Result<Value, BlackDwarfError> {
    let mut scanner = Scanner::new(doc);
    let first = scanner.peek_token(0)?.pos;

    let mut top_level = Value::new(first);
    while scanner.peek_token(0)?.type_ != TokenType::Eof {
        let peeked = scanner.peek_token(0)?;
        if peeked.type_ == TokenType::Ident {
            parse_kv(&mut scanner, &mut top_level, 0)?;
        } else if peeked.type_ == TokenType::LeftBracket {
            parse_multiline_table(&mut scanner, &mut top_level, 0)?;
        } else {
            return Err(BlackDwarfError::ParseError {
                why: format!("expected key or table header, got '{}'", peeked.lexeme),
                where_: peeked.pos,
            });
        }
    }

    Ok(top_level)
}

macro_rules! ensure {
    ($depth:ident, $scanner:ident) => {
        if $depth > 64 {
            return Err(BlackDwarfError::ParseError {
                why: format!("recursion limit exceeded"),
                where_: $scanner.peek_token(0)?.pos,
            });
        }

        let $depth = $depth + 1;
    };
}

fn parse_kv<'doc>(
    scanner: &mut Scanner<'doc>,
    current: &mut Value<'doc>,
    depth: usize,
) -> Result<(), BlackDwarfError> {
    ensure!(depth, scanner);

    let name = consume(scanner, TokenType::Ident)?;
    let _equals = consume(scanner, TokenType::Equals)?;
    let value = parse_value(scanner, depth)?;

    if !current.is_table() {
        return Err(BlackDwarfError::IncorrectType {
            type_: current.type_str(),
            expected: "table",
            where_: current.pos(),
        });
    }

    current.insert(name.lexeme, value);

    Ok(())
}

fn parse_value<'doc>(
    scanner: &mut Scanner<'doc>,
    depth: usize,
) -> Result<Value<'doc>, BlackDwarfError> {
    ensure!(depth, scanner);
    let next = scanner.next_token()?;

    match next.type_ {
        TokenType::LeftBracket => parse_array(scanner, depth),

        TokenType::LeftBrace => parse_table(scanner, depth),

        TokenType::String => Ok(Value::String {
            value: &next.lexeme[1..next.lexeme.len() - 1],
            pos: next.pos,
        }),

        TokenType::Integer(value) => Ok(Value::Integer {
            value,
            pos: next.pos,
        }),

        TokenType::Float(value) => Ok(Value::Float {
            value,
            pos: next.pos,
        }),

        TokenType::Boolean(value) => Ok(Value::Boolean {
            value,
            pos: next.pos,
        }),

        _ => {
            return Err(BlackDwarfError::ParseError {
                why: format!("{:?}s not yet supported", next.type_),
                where_: next.pos,
            })
        }
    }
}

fn parse_array<'doc>(
    scanner: &mut Scanner<'doc>,
    depth: usize,
) -> Result<Value<'doc>, BlackDwarfError> {
    ensure!(depth, scanner);
    let pos = scanner.peek_token(0)?.pos;
    if scanner.peek_token(0)?.type_ == TokenType::RightBracket {
        let _rb = consume(scanner, TokenType::RightBracket)?;
        return Ok(Value::Array {
            values: vec![],
            pos,
        });
    }

    let mut values = vec![parse_value(scanner, depth)?];
    while scanner.peek_token(0)?.type_ == TokenType::Comma && !scanner.is_at_end() {
        let _comma = consume(scanner, TokenType::Comma)?;
        if scanner.peek_token(0)?.type_ == TokenType::RightBracket {
            break;
        }
        values.push(parse_value(scanner, depth)?);
    }

    let _rb = consume(scanner, TokenType::RightBracket)?;
    Ok(Value::Array { values, pos })
}

fn parse_table<'doc>(
    scanner: &mut Scanner<'doc>,
    depth: usize,
) -> Result<Value<'doc>, BlackDwarfError> {
    ensure!(depth, scanner);
    let pos = scanner.peek_token(0)?.pos;
    if scanner.peek_token(0)?.type_ == TokenType::RightBrace {
        let _rb = consume(scanner, TokenType::RightBrace)?;
        return Ok(Value::Table {
            key_values: IndexMap::new(),
            pos,
        });
    }

    let mut key_values = Value::new(pos);
    parse_kv(scanner, &mut key_values, depth)?;
    while scanner.peek_token(0)?.type_ == TokenType::Comma && !scanner.is_at_end() {
        let _comma = consume(scanner, TokenType::Comma);
        if scanner.peek_token(0)?.type_ == TokenType::RightBrace {
            break;
        }
        parse_kv(scanner, &mut key_values, depth)?;
    }

    let _rb = consume(scanner, TokenType::RightBrace)?;
    Ok(key_values)
}

fn parse_multiline_table<'doc>(
    scanner: &mut Scanner<'doc>,
    top_level: &mut Value<'doc>,
    depth: usize,
) -> Result<(), BlackDwarfError> {
    ensure!(depth, scanner);
    let path = parse_path(scanner)?;

    let mut current = &mut *top_level;
    for fragment in path {
        if !current.is_table() {
            return Err(BlackDwarfError::IncorrectType {
                type_: current.type_str(),
                expected: "table",
                where_: current.pos(),
            });
        }

        if !current.contains_key(fragment.lexeme) {
            current.insert(fragment.lexeme, Value::new(fragment.pos));
        }

        current = current.get_mut(fragment.lexeme).unwrap();
    }

    while scanner.peek_token(0)?.type_ != TokenType::LeftBracket && !scanner.is_at_end() {
        parse_kv(scanner, current, depth)?;
    }

    Ok(())
}

fn parse_path<'doc>(scanner: &mut Scanner<'doc>) -> Result<Vec<Token<'doc>>, BlackDwarfError> {
    let _lb = consume(scanner, TokenType::LeftBracket)?;
    let mut names = vec![consume(scanner, TokenType::Ident)?];
    while scanner.peek_token(0)?.type_ != TokenType::RightBracket && !scanner.is_at_end() {
        let _dot = consume(scanner, TokenType::Dot)?;
        names.push(consume(scanner, TokenType::Ident)?);
    }
    let _rb = consume(scanner, TokenType::RightBracket)?;
    Ok(names)
}

fn consume<'a>(scanner: &mut Scanner<'a>, type_: TokenType) -> Result<Token<'a>, BlackDwarfError> {
    let tok = scanner.next_token()?;
    if tok.type_ == type_ {
        Ok(tok)
    } else {
        Err(BlackDwarfError::ParseError {
            why: format!("expected {:?}, got '{}'", type_, tok.lexeme),
            where_: tok.pos,
        })
    }
}

impl From<BlackDwarfError> for Vec<BlackDwarfError> {
    fn from(value: BlackDwarfError) -> Self {
        vec![value]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    line: usize,
    col: usize,
}

impl Pos {
    fn inc_line(&mut self) {
        self.line += 1;
    }

    fn inc_col(&mut self) {
        self.col += 1;
    }

    fn reset_col(&mut self) {
        self.col = 1;
    }
}

#[derive(Debug)]
pub struct Token<'doc> {
    pub lexeme: &'doc str,
    pub type_: TokenType,
    pub pos: Pos,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Integer(i64),
    Float(f64),
    Boolean(bool),

    String,
    Ident,

    /// [
    LeftBracket,

    /// ]
    RightBracket,

    /// {
    LeftBrace,

    /// }
    RightBrace,

    Equals,
    Dot,
    Comma,
    Eof,
}

#[derive(Debug)]
struct Scanner<'a> {
    source: &'a [u8],
    tokens: VecDeque<Token<'a>>,
    start: usize,
    current: usize,
    start_pos: Pos,
    current_pos: Pos,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Scanner {
            source: source.as_bytes(),
            tokens: VecDeque::new(),
            start: 0,
            current: 0,
            start_pos: Pos { line: 1, col: 1 },
            current_pos: Pos { line: 1, col: 1 },
        }
    }

    fn scan_all(mut self) -> Result<Vec<Token<'a>>, BlackDwarfError> {
        while self.next()?.type_ != TokenType::Eof {}
        Ok(self.tokens.drain(0..).collect())
    }

    fn next_token(&mut self) -> Result<Token<'a>, BlackDwarfError> {
        if self.tokens.is_empty() {
            self.next()?;
        }

        Ok(self.tokens.pop_front().unwrap())
    }

    fn peek_token<'b>(&'b mut self, index: usize) -> Result<&'b Token<'a>, BlackDwarfError> {
        if self.tokens.is_empty() {
            self.next()?;
        }

        while self.tokens.len() <= index {
            self.next()?;
        }

        Ok(&self.tokens[index])
    }

    fn slurp_whitespace(&mut self) {
        while self.peek_char() == b'#' || is_whitespace(self.peek_char()) {
            if self.peek_char() == b'#' {
                while !self.is_at_end() && self.peek_char() != b'\n' {
                    self.advance_char();
                }
            }
            while !self.is_at_end() && is_whitespace(self.peek_char()) {
                if self.advance_char() == b'\n' {
                    self.advance_line();
                }
            }
        }
    }

    fn next<'b>(&'b mut self) -> Result<&'b Token<'a>, BlackDwarfError> {
        self.slurp_whitespace();
        if self.is_at_end() {
            self.add_token(TokenType::Eof)?;
            return Ok(&self.tokens[self.tokens.len() - 1]);
        }

        self.set_start();
        let tk = match self.advance_char() {
            b'[' => TokenType::LeftBracket,
            b']' => TokenType::RightBracket,
            b'{' => TokenType::LeftBrace,
            b'}' => TokenType::RightBrace,
            b',' => TokenType::Comma,
            b'.' => TokenType::Dot,
            b'=' => TokenType::Equals,

            c @ (b'"' | b'\'') => self.scan_string(c)?,

            c => {
                if is_digit(c) {
                    self.scan_number()?
                } else if is_whitespace(c) {
                    panic!("found whitespace where there shouldn't be any");
                } else {
                    self.identifier_or_keyword()?
                }
            }
        };
        self.add_token(tk)?;

        Ok(&self.tokens[self.tokens.len() - 1])
    }

    fn identifier_or_keyword(&mut self) -> Result<TokenType, BlackDwarfError> {
        while !is_non_identifier(self.peek_char()) {
            self.advance_char();
        }

        if let Some(keyword) = into_keyword(self.lexeme()?) {
            Ok(keyword)
        } else {
            Ok(TokenType::Ident)
        }
    }

    fn scan_string(&mut self, quote: u8) -> Result<TokenType, BlackDwarfError> {
        while self.peek_char() != quote && !self.is_at_end() {
            if self.peek_char() == b'\n' {
                self.advance_line();
            }

            if self.peek_char() == b'\\' {
                self.advance_char();
                self.advance_line();
            }

            if !self.is_at_end() {
                self.advance_char();
            }
        }

        if self.is_at_end() {
            Err(BlackDwarfError::ParseError {
                why: "unterminated string".into(),
                where_: self.start_pos,
            })
        } else {
            self.advance_char();
            Ok(TokenType::String)
        }
    }

    fn scan_number(&mut self) -> Result<TokenType, BlackDwarfError> {
        while is_digit(self.peek_char()) {
            self.advance_char();
        }

        if self.peek_char() == b'.' {
            let range = self.lookahead_char(1) == b'.';

            while self.current != 0 && is_digit(self.peek_char()) {
                self.reverse_char();
            }

            if !range {
                return self.scan_float();
            }
        }

        let value = self.lexeme()?;
        if let Ok(i) = value.parse::<i64>() {
            Ok(TokenType::Integer(i))
        } else {
            Err(BlackDwarfError::ParseError {
                why: format!("invalid number literal '{}'", value),
                where_: self.current_pos,
            })
        }
    }

    fn scan_float(&mut self) -> Result<TokenType, BlackDwarfError> {
        while is_digit(self.peek_char()) {
            self.advance_char();
        }
        if self.peek_char() == b'.' {
            self.advance_char();
            while is_digit(self.peek_char()) {
                self.advance_char();
            }
        }

        let value = self.lexeme()?;
        if let Ok(f) = value.parse::<f64>() {
            Ok(TokenType::Float(f))
        } else {
            Err(BlackDwarfError::ParseError {
                why: format!("invalid number literal '{}'", value),
                where_: self.current_pos,
            })
        }
    }

    fn add_token(&mut self, type_: TokenType) -> Result<(), BlackDwarfError> {
        self.tokens.push_back(Token {
            type_,
            lexeme: self.lexeme()?,
            pos: self.start_pos,
        });

        Ok(())
    }

    fn lexeme(&self) -> Result<&'a str, BlackDwarfError> {
        core::str::from_utf8(&self.source[self.start..self.current]).map_err(|_| {
            BlackDwarfError::ParseError {
                why: "invalid utf-8".into(),
                where_: self.start_pos,
            }
        })
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn set_start(&mut self) {
        self.start = self.current;
        self.start_pos = self.current_pos;
    }

    fn advance_line(&mut self) {
        self.current_pos.inc_line();
        self.current_pos.reset_col();
    }

    fn advance_char(&mut self) -> u8 {
        self.current_pos.inc_col();
        self.current += 1;
        self.source[self.current - 1]
    }

    fn reverse_char(&mut self) -> u8 {
        self.current -= 1;
        self.source[self.current]
    }

    fn peek_char(&mut self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source[self.current]
        }
    }

    fn lookahead_char(&mut self, n: usize) -> u8 {
        if self.is_at_end() || self.current + n >= self.source.len() {
            b'\0'
        } else {
            self.source[self.current + n]
        }
    }
}

fn is_digit(c: u8) -> bool {
    (b'0'..=b'9').contains(&c)
}

fn is_whitespace(c: u8) -> bool {
    c == 0x09 || c == 0x0A || c == 0x0B || c == 0x0C || c == 0x0D || c == 0x20
}

fn is_non_identifier(c: u8) -> bool {
    is_whitespace(c)
        || c == 0x00
        || c == b'#'
        || c == b'['
        || c == b']'
        || c == b'{'
        || c == b'}'
        || c == b','
        || c == b'.'
        || c == b'='
        || c == b'"'
        || c == b'\''
}

fn into_keyword(s: &str) -> Option<TokenType> {
    match s {
        "true" => Some(TokenType::Boolean(true)),
        "false" => Some(TokenType::Boolean(false)),
        _ => None,
    }
}

// non-recursive
#[cfg(test)]
pub(crate) fn for_each_toml_in_dir(
    crate_dir: &std::path::Path,
    dir: &std::path::Path,
    mut f: impl FnMut(String, String),
) {
    let toml = std::ffi::OsString::from("toml");
    for file in std::fs::read_dir(dir).unwrap() {
        let file = file.unwrap();
        let absolute = file.path();
        let path = absolute.strip_prefix(crate_dir).unwrap();

        if file.file_type().unwrap().is_dir() {
            continue;
        }
        if !file.file_type().unwrap().is_file() {
            panic!(
                "{} is not a regular file (symlink, pipe, socket?)",
                path.display()
            );
        }

        if path.extension() != Some(&toml) {
            panic!("{} is not a .toml file", path.display());
        }

        f(
            format!("{}", path.display()),
            std::fs::read_to_string(path).unwrap(),
        );
    }
}

#[cfg(test)]
pub(crate) fn check_parse(name: String, contents: String) {
    println!("parse {}", name);

    let expected_debug = contents
        .lines()
        .filter(|line| line.starts_with("#--"))
        .map(|line| &line[3..])
        .fold(String::new(), |acc, next| acc + next + "\n");

    let toml = parse(&contents).unwrap();
    let debug = format!("{:#?}\n", toml);

    if expected_debug != debug {
        for diff in diff::lines(&expected_debug, &debug) {
            match diff {
                diff::Result::Left(l) => println!("-{}", l),
                diff::Result::Both(l, _) => println!(" {}", l),
                diff::Result::Right(r) => println!("+{}", r),
            }
        }
        assert_eq!(expected_debug, debug, "different parse result")
    }
}

#[test]
fn test_parse() {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ok_bd_tests_dir = crate_dir.join("tests");
    let bad_bd_tests_dir = ok_bd_tests_dir.join("should_fail");
    let ok_parse_dir = ok_bd_tests_dir.join("toml");
    let bad_parse_dir = ok_parse_dir.join("should_fail");

    for_each_toml_in_dir(&crate_dir, &ok_bd_tests_dir, check_parse);
    for_each_toml_in_dir(&crate_dir, &bad_bd_tests_dir, check_parse);
    for_each_toml_in_dir(&crate_dir, &ok_parse_dir, check_parse);

    for_each_toml_in_dir(&crate_dir, &bad_parse_dir, |name, contents| {
        println!("parse {}, should fail", name);
        let _ = parse(&contents).unwrap_err();
    });
}
