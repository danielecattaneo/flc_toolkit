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
