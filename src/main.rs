mod lexer;
mod parser;
mod fsm;
mod mnet;
mod elr_pilot;

use std::env::args;
use std::path::Path;
use std::process::exit;

pub use crate::elr_pilot::*;
pub use crate::lexer::*;
pub use crate::parser::*;

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} [file]", args[0]);
        exit(1);
    }

    let lex = Lexer::from_path(Path::new(&args[1]));
    let mut pars = Parser::new(lex);
    if let Some(net) = pars.parse_mnet() {
        if net.validate() {
            let pilot = create_pilot(&net);
            //println!("pilot: {pilot:?}");
            println!("{}", pilot.to_dot());
            pilot.print_conflicts();
        }
    }
}
