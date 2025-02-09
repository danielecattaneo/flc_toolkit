mod lexer;
mod parser;
mod fsm;
mod mnet;
mod elr_pilot;

use std::path::Path;
use std::process::ExitCode;

pub use crate::elr_pilot::*;
pub use crate::lexer::*;
pub use crate::parser::*;

fn help() {
    let args: Vec<_> = std::env::args().collect();
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    let path = &args[0];
    //         12345678901234567890123456789012345678901234567890123456789012345678901234567890
    //                  11111111112222222222333333333344444444445555555555666666666677777777778
    eprintln!("FLC Toolkit {VERSION}");
    eprintln!("Usage: {} [cmd arg1 arg2 ...] [cmd arg1 arg2 ...] ...", path);
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  help");
    eprintln!("    Prints this information.");
    eprintln!("  echo_mnet <file>");
    eprintln!("    Prints the machine net in <file> to the standard output stream in graphviz");
    eprintln!("    dot format.");
    eprintln!("  pilot <file>");
    eprintln!("    Computes the ELR(1) pilot of the machine net in <file>, prints it to the");
    eprintln!("    standard output stream in graphviz dot format, and prints any conflict to");
    eprintln!("    standard error stream.");
}

fn cmd_echo_mnet(args: &[String]) -> Option<&[String]> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"echo_mnet\" command");
        return None;
    }
    let file = &args[0];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    if let Some(net) = pars.parse_mnet() {
        if net.validate() {
            println!("{}", net.to_dot());
        }
    }
    return Some(&args[1..]);
}

fn cmd_pilot(args: &[String]) -> Option<&[String]> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"pilot\" command");
        return None;
    }
    let file = &args[0];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    if let Some(net) = pars.parse_mnet() {
        if net.validate() {
            let pilot = create_pilot(&net);
            println!("{}", pilot.to_dot());
            pilot.print_conflicts();
        }
    }
    return Some(&args[1..]);
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        eprintln!("error: no commands given");
        help();
        return ExitCode::FAILURE;
    }

    let mut args_left = &args[1..];
    while !args_left.is_empty() {
        let cmd = &args_left[0];
        let new_args_left = if cmd == "pilot" {
            cmd_pilot(&args_left[1..])
        } else if cmd == "echo_mnet" {
            cmd_echo_mnet(&args_left[1..])
        } else if cmd == "help" || cmd == "-h" || cmd == "--help" {
            eprintln!("help requested");
            help();
            return ExitCode::SUCCESS;
        } else {
            eprintln!("error: invalid command \"{cmd}\"");
            None
        };
        if let Some(new_args_left) = new_args_left {
            args_left = new_args_left;
        } else {
            eprintln!("error");
            help();
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
