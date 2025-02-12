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

struct RegexFormatter {
    numbered: bool,
    prev: RegexFmtCharClass,
}

impl RegexFormatter {
    fn write(&mut self, f: &mut fmt::Formatter<'_>, next: RegexFmtCharClass) -> fmt::Result {
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
            write!(f, " ")?;
        }
        match next {
            RegexFmtCharClass::Ini => fmt::Result::Ok(()),
            RegexFmtCharClass::Literal(c, i) => {
                if !self.numbered {
                    write!(f, "{}", c)
                } else {
                    write!(f, "{}{}", c, i)
                }
            }
            RegexFmtCharClass::OpenGroup(c) => write!(f, "{}", c),
            RegexFmtCharClass::ClosedGroup(c) => write!(f, "{}", c),
            RegexFmtCharClass::UnOp(c) => write!(f, "{}", c),
            RegexFmtCharClass::BinOp(c) => write!(f, "{}", c)
        }?;
        self.prev = next;
        fmt::Result::Ok(())
    }

    fn fmt_child(&mut self, f: &mut fmt::Formatter<'_>, mom: &Regex, daughter: &Regex) -> fmt::Result {
        if mom.precedence() > daughter.precedence() {
            self.write(f, RegexFmtCharClass::OpenGroup('('))?;
            self.fmt(f, daughter)?;
            self.write(f, RegexFmtCharClass::ClosedGroup(')'))
        } else {
            self.fmt(f, daughter)
        }
    }

    fn fmt(&mut self, f: &mut fmt::Formatter<'_>, re: &Regex) -> fmt::Result {
        match re {
            Regex::Null => self.write(f, RegexFmtCharClass::Literal('_', 0)),
            Regex::Literal(c, i) => self.write(f, RegexFmtCharClass::Literal(*c, *i)),
            Regex::Star(re2) => {
                self.fmt_child(f, re, re2)?;
                self.write(f, RegexFmtCharClass::UnOp('*'))
            },
            Regex::Plus(re2) => {
                self.fmt_child(f, re, re2)?;
                self.write(f, RegexFmtCharClass::UnOp('+'))
            },
            Regex::Concat(lhs, rhs) => {
                self.fmt_child(f, re, lhs)?;
                self.fmt_child(f, re, rhs)
            },
            Regex::Union(lhs, rhs) => {
                self.fmt_child(f, re, lhs)?;
                self.write(f, RegexFmtCharClass::BinOp('|'))?;
                self.fmt_child(f, re, rhs)
            }
        }
    }
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

impl fmt::Display for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut refmt = RegexFormatter{
            numbered: true, 
            prev: RegexFmtCharClass::Ini
        };
        refmt.fmt(f, self)
    }
}
