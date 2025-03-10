mod reg_lang;
mod lexer;
mod regex;
mod parser;
mod fsm;
mod mnet;
mod elr_pilot;
mod berry_sethi;
mod bmc;

use std::path::Path;
use std::process::ExitCode;

pub use crate::elr_pilot::*;
pub use crate::berry_sethi::*;
pub use crate::bmc::*;
pub use crate::lexer::*;
pub use crate::parser::*;
pub use crate::reg_lang::*;
pub use crate::regex::parser::*;

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

fn parse_list_arg(arg: &str) -> Option<Vec<i32>> {
    let parts = arg.split(',');
    let mut res: Vec<i32> = vec![];
    for part in parts {
        if let Ok(v) = part.trim().parse::<i32>() {
            res.push(v);
        } else {
            eprintln!("error: cannot parse list \"{}\" in arguments", arg);
            return None;
        }
    }
    Some(res)
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

fn cmd_echo_regex(args: &[String]) -> Option<&[String]> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"echo_regex\" command");
        return None;
    }
    let re_str = &args[0];
    let mut pars = RegexParser::new(re_str);
    if let Some(re) = pars.parse_regex() {
        println!("{}", re);
    }
    return Some(&args[1..]);
}

fn cmd_berry_sethi(args: &[String]) -> Option<&[String]> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"berry_sethi\" command");
        return None;
    }
    let re_str = &args[0];
    let mut pars = RegexParser::new(re_str);
    if let Some(re) = pars.parse_regex() {
        eprintln!("{}", re.to_string_numbered());
        re.dump_local_sets();
        println!("{}", berry_sethi(&re).to_dot(false));
    }
    return Some(&args[1..]);
}

fn cmd_berry_sethi_fsm(args: &[String]) -> Option<&[String]> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"berry_sethi_fsm\" command");
        return None;
    }
    let file = &args[0];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    if let Some(fsm) = pars.parse_machine() {
        let num_fsm = NumMachine::from_machine(fsm);
        num_fsm.dump_local_sets();
        println!("digraph {{\n  rankdir=\"LR\";");
        println!("{}", num_fsm.to_dot_2(false, false));
        println!("{}", berry_sethi(&num_fsm).to_dot_2(false, false));
        println!("}}");
    }
    return Some(&args[1..]);
}

fn cmd_bmc(args: &[String]) -> Option<&[String]> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"bmc\" command");
        return None;
    }
    let file = &args[0];
    let args_left = &args[1..];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    let fsm = pars.parse_machine()?;

    if args_left.len() >= 1 && (args_left[0] == "--order" || args_left[0] == "-o") {
        if args_left.len() < 2 {
            eprintln!("error: missing argument to \"--order\"");
            return None;
        }
        let list = parse_list_arg(args_left[1].as_str())?;
        let mut bmc_fsm = BMCMachine::from_machine(fsm);
        println!("digraph {{\n  rankdir=\"LR\";");
        bmc_fsm.merge_parallel_transitions();
        println!("{}", bmc_fsm.to_dot_2(false, false));
        for sid in list {
            let Some(_) = bmc_fsm.try_lookup_state(sid) else {
                eprintln!("error: state {} does not exist", sid);
                return None;
            };
            eprintln!("Eliminating state {}", sid);
            bmc_fsm.eliminate(sid);
            bmc_fsm.merge_parallel_transitions();
            println!("{}", bmc_fsm.to_dot_2(false, false));
        }
        println!("}}");
        Some(&args_left[2..])
    } else {
        let mut bmc_fsm = BMCMachine::from_machine(fsm);
        println!("digraph {{\n  rankdir=\"LR\";");
        bmc_fsm.merge_parallel_transitions();
        println!("{}", bmc_fsm.to_dot_2(false, false));
        while let Some(sid) = bmc_fsm.choose_best_state() {
            eprintln!("Eliminating state {}", sid);
            bmc_fsm.eliminate(sid);
            bmc_fsm.merge_parallel_transitions();
            println!("{}", bmc_fsm.to_dot_2(false, false));
        }
        println!("}}");
        Some(&args_left)
    }
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
        } else if cmd == "echo_regex" {
            cmd_echo_regex(&args_left[1..])
        } else if cmd == "berry_sethi" {
            cmd_berry_sethi(&args_left[1..])
        } else if cmd == "berry_sethi_fsm" {
            cmd_berry_sethi_fsm(&args_left[1..])
        } else if cmd == "bmc" {
            cmd_bmc(&args_left[1..])
        } else if cmd == "help" || cmd == "-h" || cmd == "--help" {
            help();
            return ExitCode::SUCCESS;
        } else {
            eprintln!("error: invalid command \"{cmd}\"");
            None
        };
        if let Some(new_args_left) = new_args_left {
            args_left = new_args_left;
        } else {
            help();
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
