use std::collections::HashSet;

pub mod parser;
mod formatter;

#[derive(Debug)]
pub enum Regex {
    Null,
    Literal(char, usize),
    Union(Box<Regex>, Box<Regex>),
    Concat(Box<Regex>, Box<Regex>),
    Star(Box<Regex>),
    Plus(Box<Regex>)
}

fn set_prod(a: &HashSet<(char, usize)>, b: &HashSet<(char, usize)>) -> HashSet<((char, usize), (char, usize))> {
    let mut res: HashSet<((char, usize), (char, usize))> = HashSet::new();
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
            Regex::Literal(_, _) => false,
            Regex::Union(r1, r2) => r1.nullable() || r2.nullable(),
            Regex::Concat(r1,r2) => r1.nullable() && r2.nullable(),
            Regex::Plus(r1) => r1.nullable(),
            Regex::Star(_) => true
        }
    }

    pub fn all_numbered(&self) -> HashSet<(char, usize)> {
        match self {
            Regex::Null => HashSet::new(),
            Regex::Literal(c, i) => HashSet::from([(*c, *i)]),
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

    pub fn numbered_initials(&self) -> HashSet<(char, usize)> {
        match self {
            Regex::Null => HashSet::new(),
            Regex::Literal(c, i) => HashSet::from([(*c, *i)]),
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
            Regex::Plus(r1) | Regex::Star(r1) => r1.numbered_initials()
        }
    }

    pub fn numbered_finals(&self) -> HashSet<(char, usize)> {
        match self {
            Regex::Null => HashSet::new(),
            Regex::Literal(c, i) => HashSet::from([(*c, *i)]),
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
            Regex::Plus(r1) | Regex::Star(r1) => r1.numbered_finals()
        }
    }

    pub fn numbered_digrams(&self) -> HashSet<((char, usize), (char, usize))> {
        match self {
            Regex::Null | Regex::Literal(_, _) => HashSet::new(),
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
