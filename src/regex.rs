pub mod parser;
mod formatter;

use std::collections::HashSet;
use std::fmt;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NumTerm {
    pub c: char,
    pub i: usize
}

impl NumTerm {
    pub fn new(c: char, i: usize) -> NumTerm {
        NumTerm{c, i}
    }
}

impl fmt::Display for NumTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.c, self.i)
    }
}

pub type NumTermSet = HashSet<NumTerm>;
pub type NumDigramsSet = HashSet<(NumTerm, NumTerm)>;

#[derive(Debug)]
pub enum Regex {
    Null,
    Literal(NumTerm),
    Union(Box<Regex>, Box<Regex>),
    Concat(Box<Regex>, Box<Regex>),
    Star(Box<Regex>),
    Plus(Box<Regex>)
}

fn set_prod(a: &NumTermSet, b: &NumTermSet) -> NumDigramsSet {
    let mut res = NumDigramsSet::new();
    for &ia in a {
        for &ib in b {
            res.insert((ia, ib));
        }
    }
    res
}

impl Regex {
    pub fn nullable(&self) -> bool {
        match self {
            Regex::Null => true,
            Regex::Literal(_) => false,
            Regex::Union(r1, r2) => r1.nullable() || r2.nullable(),
            Regex::Concat(r1,r2) => r1.nullable() && r2.nullable(),
            Regex::Plus(r1) => r1.nullable(),
            Regex::Star(_) => true
        }
    }

    pub fn all_numbered(&self) -> NumTermSet {
        match self {
            Regex::Null => HashSet::new(),
            Regex::Literal(t) => NumTermSet::from([*t]),
            Regex::Union(r1, r2)
            | Regex::Concat(r1, r2) => {
                let mut set = r1.all_numbered();
                set.extend(r2.all_numbered());
                set
            }
            Regex::Plus(r1)
            | Regex::Star(r1) => r1.all_numbered()
        }
    }

    pub fn numbered_initials(&self) -> NumTermSet {
        match self {
            Regex::Null => HashSet::new(),
            Regex::Literal(t) => NumTermSet::from([*t]),
            Regex::Union(r1, r2) => {
                let mut set = r1.numbered_initials();
                set.extend(r2.numbered_initials());
                set
            },
            Regex::Concat(r1, r2) => {
                let mut set = r1.numbered_initials();
                if r1.nullable() {
                    set.extend(r2.numbered_initials());
                }
                set
            }
            Regex::Plus(r1)
            | Regex::Star(r1) => r1.numbered_initials()
        }
    }

    pub fn numbered_finals(&self) -> NumTermSet {
        match self {
            Regex::Null => HashSet::new(),
            Regex::Literal(t) => NumTermSet::from([*t]),
            Regex::Union(r1, r2) => {
                let mut set = r1.numbered_finals();
                set.extend(r2.numbered_finals());
                set
            },
            Regex::Concat(r1, r2) => {
                let mut set = r2.numbered_finals();
                if r2.nullable() {
                    set.extend(r1.numbered_finals());
                }
                set
            }
            Regex::Plus(r1)
            | Regex::Star(r1) => r1.numbered_finals()
        }
    }

    pub fn numbered_digrams(&self) -> NumDigramsSet {
        match self {
            Regex::Null | Regex::Literal(_) => NumDigramsSet::new(),
            Regex::Union(r1, r2) => {
                let mut set = r1.numbered_digrams();
                set.extend(r2.numbered_digrams());
                set
            },
            Regex::Concat(r1, r2) => {
                let mut res = r1.numbered_digrams();
                res.extend(r2.numbered_digrams());
                res.extend(set_prod(&r1.numbered_finals(), &r2.numbered_initials()));
                res
            }
            Regex::Plus(r1) | Regex::Star(r1) => {
                let mut res = r1.numbered_digrams();
                res.extend(set_prod(&r1.numbered_finals(), &r1.numbered_initials()));
                res
            }
        }
    }
}
