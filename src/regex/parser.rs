use std::mem::replace;
use std::str::CharIndices;

use crate::regex::*;


enum RegexTokenValue {
    Invalid,
    Epsilon,
    Literal(char),
    Union,
    Concat,
    Star,
    Plus,
    LPar,
    RPar,
    LSquare,
    RSquare
}

struct RegexToken {
    pub location: usize,
    pub value: RegexTokenValue
}

impl RegexToken {
    pub fn from_char(location: usize, c: char) -> Option<RegexToken> {
        let value = if c.is_ascii_lowercase() {
            RegexTokenValue::Literal(c)
        } else if c == '_' {
            RegexTokenValue::Epsilon
        } else if c == '|' {
            RegexTokenValue::Union
        } else if c == '.' {
            RegexTokenValue::Concat
        } else if c == '*' {
            RegexTokenValue::Star
        } else if c == '+' {
            RegexTokenValue::Plus
        } else if c == '(' {
            RegexTokenValue::LPar
        } else if c == ')' {
            RegexTokenValue::RPar
        } else if c == '[' {
            RegexTokenValue::LSquare
        } else if c == ']' {
            RegexTokenValue::RSquare
        } else {
            RegexTokenValue::Invalid
        };
        Some(RegexToken{ location, value })
    }
}


struct RegexLexer<'a> {
    rest: CharIndices<'a>
}

impl RegexLexer<'_> {
    fn from_str(string: &str) -> RegexLexer<'_> {
        RegexLexer{ rest: string.char_indices() }
    }
}

impl Iterator for RegexLexer<'_> {
    type Item = RegexToken;

    fn next(&mut self) -> Option<RegexToken> {
        for (i, c) in self.rest.by_ref() {
            if ! c.is_ascii_whitespace() {
                return RegexToken::from_char(i, c);
            }
        }
        None
    }
}


macro_rules! token {
    ($p:pat_param) => (
        Some(RegexToken{ value:$p, .. })
    );
}

macro_rules! expect {
    ($self:expr, $p:pat_param, $err:expr, $b:block) => (
        if let token!($p) = $self.lookahead {
            let res = $b;
            $self.advance();
            res
        } else {
            $self.emit_error($err);
            return None;
        }
    );
    ($self:expr, $p:pat_param, $err:expr) => (
        expect!($self, $p, $err, {})
    );
}

macro_rules! accept {
    ($self:expr, $p:pat_param) => (
        if let token!($p) = $self.lookahead {
            $self.advance()
        } else {
            None
        }
    );
}

pub struct RegexParser<'a> {
    string: & 'a str,
    lexer: RegexLexer<'a>,
    lookahead: Option<RegexToken>,
    lit_counter: usize
}

impl RegexParser<'_> {
    pub fn new(string: &str) -> RegexParser<'_> {
        let mut lexer = RegexLexer::from_str(string);
        let lookahead = lexer.next();
        RegexParser{ string, lexer, lookahead, lit_counter: 0 }
    }

    fn emit_error(&self, s: &str) {
        let loc = if let Some(look) = &self.lookahead {
            look.location
        } else {
            self.string.len()
        };
        eprintln!("{}", self.string);
        for _ in 0..loc { eprint!("~"); }
        eprintln!("^");
        eprintln!("error: {s}");
    }

    fn advance(&mut self) -> Option<RegexToken> {
        if self.lookahead.is_some() {
            replace(&mut self.lookahead, self.lexer.next())
        } else {
            None
        }
    }

    fn parse_term(&mut self) -> Option<Regex> {
        if accept!(self, RegexTokenValue::LPar).is_some() {
            let res = self.parse_union()?;
            expect!(self, RegexTokenValue::RPar, "mismatched parenthesis");
            Some(res)
        } else if accept!(self, RegexTokenValue::LSquare).is_some() {
            let lhs = self.parse_union()?;
            expect!(self, RegexTokenValue::RSquare, "mismatched square parenthesis");
            Some(Regex::Union(Box::new(lhs), Box::new(Regex::Null)))
        } else if accept!(self, RegexTokenValue::Epsilon).is_some() {
            Some(Regex::Null)
        } else if let token!(RegexTokenValue::Literal(c)) = self.lookahead {
            self.advance();
            self.lit_counter += 1;
            Some(Regex::Literal(NumTerm::new(c, self.lit_counter)))
        } else {
            self.emit_error("expected a character or a group");
            None
        }
    }

    fn parse_star(&mut self) -> Option<Regex> {
        let mut lhs = self.parse_term()?;
        loop {
            if accept!(self, RegexTokenValue::Star).is_some() {
                lhs = Regex::Star(Box::new(lhs));
            } else if accept!(self, RegexTokenValue::Plus).is_some() {
                lhs = Regex::Plus(Box::new(lhs));
            } else {
                break Some(lhs);
            }
        }
    }

    fn parse_concat(&mut self) -> Option<Regex> {
        let mut lhs = self.parse_star()?;
        loop {
            if accept!(self, RegexTokenValue::Concat).is_some() {
                let rhs = self.parse_star()?;
                lhs = Regex::Concat(Box::new(lhs), Box::new(rhs));
            } else if let token!(RegexTokenValue::Literal(_))
                    | token!(RegexTokenValue::Epsilon)
                    | token!(RegexTokenValue::LPar)
                    | token!(RegexTokenValue::LSquare) = self.lookahead {
                let rhs = self.parse_star()?;
                lhs = Regex::Concat(Box::new(lhs), Box::new(rhs));
            } else {
                break Some(lhs);
            }
        }
    }

    fn parse_union(&mut self) -> Option<Regex> {
        let mut lhs = self.parse_concat()?;
        loop {
            if accept!(self, RegexTokenValue::Union).is_some() {
                let rhs = self.parse_concat()?;
                lhs = Regex::Union(Box::new(lhs), Box::new(rhs));
            } else {
                break Some(lhs);
            }
        }
    }

    pub fn parse_regex(&mut self) -> Option<Regex> {
        if self.lookahead.is_none() {
            Some(Regex::Null)
        } else {
            let res = self.parse_union()?;
            if self.lookahead.is_none() {
                Some(res)
            } else {
                self.emit_error("unrecognized trailing characters");
                None
            }
        }
    }
}
