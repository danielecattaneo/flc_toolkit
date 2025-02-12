use crate::regex::*;

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
            Regex::Null | Regex::Literal(_, _) => 0,
            Regex::Star(_) | Regex::Plus(_) => -1,
            Regex::Concat(_, _) => -2,
            Regex::Union(_, _) => -3
        }
    }
}

struct RegexFormatter {
    numbered: bool,
    prev: RegexFmtCharClass,
    buf: String
}

impl RegexFormatter {
    fn write(&mut self, next: RegexFmtCharClass) {
        let space = if !self.numbered {
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
                if !self.numbered {
                    self.buf.push(c);
                } else {
                    self.buf = format!("{}{}{}", self.buf, c, i);
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
            Regex::Null => self.write(RegexFmtCharClass::Literal('_', 0)),
            Regex::Literal(c, i) => self.write(RegexFmtCharClass::Literal(*c, *i)),
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
            numbered: false, 
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
            numbered: true, 
            prev: RegexFmtCharClass::Ini,
            buf: String::new()
        };
        refmt.fmt(self);
        refmt.buf
    }
}
