pub mod parser;

use std::fmt;

#[derive(Debug)]
pub enum Regex {
    Null,
    Literal(char),
    Union(Box<Regex>, Box<Regex>),
    Concat(Box<Regex>, Box<Regex>),
    Star(Box<Regex>),
    Plus(Box<Regex>)
}

impl Regex {
    fn precedence(&self) -> i32 {
        match self {
            Regex::Null | Regex::Literal(_) => 0,
            Regex::Star(_) | Regex::Plus(_) => -1,
            Regex::Concat(_, _) => -2,
            Regex::Union(_, _) => -3
        }
    }

    fn fmt_child(&self, child: &Regex, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.precedence() > child.precedence() {
            write!(f, "({})", child)
        } else {
            write!(f, "{}", child)
        }
    }
}

impl fmt::Display for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Regex::Null => write!(f, "_"),
            Regex::Literal(c) => write!(f, "{}", c),
            Regex::Star(re) => {
                self.fmt_child(re, f)?;
                write!(f, "*")
            },
            Regex::Plus(re) => {
                self.fmt_child(re, f)?;
                write!(f, "+")
            },
            Regex::Concat(lhs, rhs) => {
                self.fmt_child(lhs, f)?;
                self.fmt_child(rhs, f)
            },
            Regex::Union(lhs, rhs) => {
                self.fmt_child(lhs, f)?;
                write!(f, "|")?;
                self.fmt_child(rhs, f)
            }
        }
    }
}
