use crate::{regex::*, DotFormat};

use std::fmt;

enum RegexFmtCharClass {
    Ini,
    Literal(char, usize),
    OpenGroup(char),
    ClosedGroup(char),
    UnOp(char),
    BinOp(char)
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
}

#[derive(PartialEq, Eq)]
enum RegexFmtStyle {
    Plain,
    Numbered,
    Dot,
    DotNumbered
}

use RegexFmtStyle::*;

struct RegexFormatter {
    style: RegexFmtStyle,
    prev: RegexFmtCharClass,
    buf: String
}

impl RegexFormatter {
    fn write(&mut self, next: RegexFmtCharClass) {
        let space = if self.style == Plain || self.style == Dot {
            false
        } else if let RegexFmtCharClass::Ini = self.prev {
            false
        } else {
            match next {
                RegexFmtCharClass::Ini => true,
                RegexFmtCharClass::Literal(_, _) => true,
                RegexFmtCharClass::OpenGroup(_) => {
                    if let RegexFmtCharClass::OpenGroup(_) = self.prev { false } else { true }
                }
                RegexFmtCharClass::ClosedGroup(_) => {
                    if let RegexFmtCharClass::ClosedGroup(_) = self.prev { false } else { true }
                }
                RegexFmtCharClass::UnOp(_) => false,
                RegexFmtCharClass::BinOp(_) => true
            }
        };
        if space {
            self.buf.push(' ');
        }
        match next {
            RegexFmtCharClass::Ini =>(),
            RegexFmtCharClass::Literal(c, i) => {
                match self.style {
                    Plain => self.buf.push(c),
                    Numbered => if c == '_' { 
                        self.buf.push(c);
                    } else {
                        self.buf = format!("{}{}{}", self.buf, c, i);
                    },
                    Dot => self.buf.push(if c == '_' { 'ε' } else { c }),
                    DotNumbered => if c == '_' {
                        self.buf.push('ε')
                    } else {
                        self.buf = format!("{}{}<sub>{}</sub>", self.buf, c, i)
                    }
                }
            }
            RegexFmtCharClass::OpenGroup(c)
                | RegexFmtCharClass::ClosedGroup(c)
                | RegexFmtCharClass::UnOp(c)
                | RegexFmtCharClass::BinOp(c) => self.buf.push(c)
        };
        self.prev = next;
    }

    fn fmt_child(&mut self, mom: &Regex, daughter: &Regex) {
        if mom.precedence() > daughter.precedence() {
            self.write(RegexFmtCharClass::OpenGroup('('));
            self.fmt(daughter);
            self.write(RegexFmtCharClass::ClosedGroup(')'));
        } else {
            self.fmt(daughter);
        }
    }

    fn fmt(&mut self, re: &Regex) {
        match re {
            Regex::Null => {
                self.write(RegexFmtCharClass::Literal('_', 0))
            }
            Regex::Literal(t) => {
                self.write(RegexFmtCharClass::Literal(t.c, t.i))
            }
            Regex::Star(re2) => {
                self.fmt_child(re, re2);
                self.write(RegexFmtCharClass::UnOp('*'));
            },
            Regex::Plus(re2) => {
                self.fmt_child(re, re2);
                self.write(RegexFmtCharClass::UnOp('+'));
            },
            Regex::Concat(lhs, rhs) => {
                self.fmt_child(re, lhs);
                self.fmt_child(re, rhs);
            },
            Regex::Union(lhs, rhs) => {
                self.fmt_child(re, lhs);
                self.write(RegexFmtCharClass::BinOp('|'));
                self.fmt_child(re, rhs);
            }
        }
    }
}

impl fmt::Display for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut refmt = RegexFormatter{
            style: Plain, 
            prev: RegexFmtCharClass::Ini,
            buf: String::new()
        };
        refmt.fmt(self);
        write!(f, "{}", refmt.buf)
    }
}

impl Regex {
    pub fn to_string_numbered(&self) -> String {
        let mut refmt = RegexFormatter{
            style: Numbered, 
            prev: RegexFmtCharClass::Ini,
            buf: String::new()
        };
        refmt.fmt(self);
        refmt.buf
    }
}

impl DotFormat for Regex {
    fn to_dot(&self, detailed: bool) -> String {
        let mut refmt = RegexFormatter{
            style: if detailed { DotNumbered } else { Dot },
            prev: RegexFmtCharClass::Ini,
            buf: String::new()
        };
        refmt.fmt(self);
        format!("<{}>", refmt.buf)
    }
}
