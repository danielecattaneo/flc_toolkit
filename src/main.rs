mod reg_lang;
mod lexer;
mod regex;
mod parser;
mod fsm;
mod mnet;
mod validation;
mod elr_pilot;
mod berry_sethi;
mod bmc;
mod epsilon_elim;

use std::path::Path;
use std::process::ExitCode;

pub use crate::lexer::*;
pub use crate::parser::*;
pub use crate::reg_lang::*;
pub use crate::regex::parser::*;
pub use crate::validation::*;
pub use crate::elr_pilot::*;
pub use crate::berry_sethi::*;
pub use crate::bmc::*;

enum CmdError {
    BadArgs,
    ExecError
}

fn banner() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    eprintln!("FLC Toolkit {VERSION}");
}

fn help() {
    let args: Vec<_> = std::env::args().collect();
    let path = &args[0];
    //         12345678901234567890123456789012345678901234567890123456789012345678901234567890
    //                  11111111112222222222333333333344444444445555555555666666666677777777778
    eprintln!("Usage: {path} [cmd arg1 arg2 ...] [cmd arg1 arg2 ...] ...");
    eprintln!();
    eprintln!("Commands:");
    eprintln!();
    eprintln!("  help, -h, --help");
    eprintln!("    Prints this information.");
    eprintln!();
    eprintln!("  echo_mnet <file>");
    eprintln!("    Prints the machine net in <file> to the standard output stream in graphviz");
    eprintln!("    dot format.");
    eprintln!();
    eprintln!("  pilot <file>");
    eprintln!("    Computes the ELR(1) pilot of the machine net in <file>, prints it to the");
    eprintln!("    standard output stream in graphviz dot format, and prints any conflict to");
    eprintln!("    standard error stream.");
    eprintln!();
    eprintln!("  echo_regex <regex>");
    eprintln!("    Reprints <regex> to the standard output stream with a minimal set of");
    eprintln!("    parenthesis.");
    eprintln!();
    eprintln!("  echo_fsm <file>");
    eprintln!("    Prints the FSM in <file> to the standard output stream in graphviz dot");
    eprintln!("    format.");
    eprintln!();
    eprintln!("  berry_sethi <regex>");
    eprintln!("    Converts the given <regex> to a finite state machine using the Berry-Sethi");
    eprintln!("    algorithm, and then prints it to the standard output stream in graphviz dot");
    eprintln!("    format. The sets of initials and followers are also printed to the standard");
    eprintln!("    error stream.");
    eprintln!();
    eprintln!("  berry_sethi_fsm <file>");
    eprintln!("    Determinizes the FSM in <file> using the Berry-Sethi algorithm, and then");
    eprintln!("    prints it to the standard output stream in graphviz dot format. The sets of");
    eprintln!("    initials and followers are also printed to the standard error stream.");
    eprintln!();
    eprintln!("  bmc <file> [-o|--order <list>]");
    eprintln!("    Converts the FSM in <file> to a regular expression using the Brzozowski-");
    eprintln!("    McCluskey (node elimination) method. The reduction steps are logged to the");
    eprintln!("    standard error stream, and the intermediate and final steps of the reduction");
    eprintln!("    are printed in graphviz dot format to the standard output.");
    eprintln!("    Options:");
    eprintln!("      -o|--order <list>   Specifies the order of reduction of the states as a");
    eprintln!("                          comma-separated list. For example '3,2,1' forces the");
    eprintln!("                          elimination of state 3 first, followed by states 2 and");
    eprintln!("                          1. Any other state left is not eliminated.");
    eprintln!();
    eprintln!("  backprop <file>");
    eprintln!("  forwardprop <file>");
    eprintln!("    Eliminates spontaneous moves (epsilon-transitions) from the FSM in <file>,");
    eprintln!("    either by backward propagation (backprop) or by forward propagation");
    eprintln!("    (forwardprop). The result is printed to the standard output in graphviz dot");
    eprintln!("    format.");
    eprintln!();
}

fn parse_list_arg(arg: &str) -> Option<Vec<i32>> {
    let parts = arg.split(',');
    let mut res: Vec<i32> = vec![];
    for part in parts {
        if let Ok(v) = part.trim().parse::<i32>() {
            res.push(v);
        } else {
            eprintln!("error: cannot parse list \"{arg}\" in arguments");
            return None;
        }
    }
    Some(res)
}

fn cmd_echo_mnet(args: &[String]) -> Result<&[String], CmdError> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"echo_mnet\" command");
        return Err(CmdError::BadArgs);
    }
    let file = &args[0];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    if let Some(net) = validated(pars.parse_mnet_file()) {
        println!("{}", net.to_dot());
        Ok(&args[1..])
    } else {
        Err(CmdError::ExecError)
    }
}

fn cmd_pilot(args: &[String]) -> Result<&[String], CmdError> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"pilot\" command");
        return Err(CmdError::BadArgs);
    }
    let file = &args[0];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    if let Some(net) = validated(pars.parse_mnet_file()) {
        let pilot = create_pilot(&net);
        println!("{}", pilot.to_dot());
        pilot.print_conflicts();
        Ok(&args[1..])
    } else {
        Err(CmdError::ExecError)
    }
}

fn cmd_echo_regex(args: &[String]) -> Result<&[String], CmdError> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"echo_regex\" command");
        return Err(CmdError::BadArgs);
    }
    let re_str = &args[0];
    let mut pars = RegexParser::new(re_str);
    if let Some(re) = pars.parse_regex() {
        println!("{re}");
        Ok(&args[1..])
    } else {
        Err(CmdError::ExecError)
    }
}

fn cmd_echo_fsm(args: &[String]) -> Result<&[String], CmdError> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"echo_fsm\" command");
        return Err(CmdError::BadArgs);
    }
    let file = &args[0];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    if let Some(fsm) = validated(pars.parse_machine_file()) {
        println!("{}", fsm.to_dot(false));
        Ok(&args[1..])
    } else {
        Err(CmdError::ExecError)
    }
}

fn cmd_berry_sethi(args: &[String]) -> Result<&[String], CmdError> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"berry_sethi\" command");
        return Err(CmdError::BadArgs);
    }
    let re_str = &args[0];
    let mut pars = RegexParser::new(re_str);
    if let Some(re) = pars.parse_regex() {
        eprintln!("{}", re.to_string_numbered());
        re.dump_local_sets();
        println!("{}", berry_sethi(&re).to_dot(false));
        Ok(&args[1..])
    } else {
        Err(CmdError::ExecError)
    }
}

fn cmd_berry_sethi_fsm(args: &[String]) -> Result<&[String], CmdError> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"berry_sethi_fsm\" command");
        return Err(CmdError::BadArgs);
    }
    let file = &args[0];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    if let Some(fsm) = validated(pars.parse_machine_file()) {
        let num_fsm = NumMachine::from_machine(fsm);
        num_fsm.dump_local_sets();
        println!("digraph {{\n  rankdir=\"LR\";");
        println!("{}", num_fsm.to_dot_2(false, false));
        println!("{}", berry_sethi(&num_fsm).to_dot_2(false, false));
        println!("}}");
        Ok(&args[1..])
    } else {
        Err(CmdError::ExecError)
    }
}

fn cmd_bmc(args: &[String]) -> Result<&[String], CmdError> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"bmc\" command");
        return Err(CmdError::BadArgs);
    }
    let file = &args[0];
    let args_left = &args[1..];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    let Some(fsm) = validated(pars.parse_machine_file()) else {
        return Err(CmdError::ExecError);
    };

    if args_left.len() >= 1 && (args_left[0] == "--order" || args_left[0] == "-o") {
        if args_left.len() < 2 {
            eprintln!("error: missing argument to \"--order\"");
            return Err(CmdError::BadArgs);
        }
        let Some(list) = parse_list_arg(args_left[1].as_str()) else {
            return Err(CmdError::BadArgs);
        };
        let mut bmc_fsm = BMCMachine::from_machine(fsm);
        println!("digraph {{\n  rankdir=\"LR\";");
        bmc_fsm.merge_parallel_transitions();
        println!("{}", bmc_fsm.to_dot_2(false, false));
        for sid in list {
            let Some(_) = bmc_fsm.try_lookup_state(sid) else {
                eprintln!("error: state {sid} does not exist");
                return Err(CmdError::BadArgs);
            };
            eprintln!("Eliminating state {sid}");
            bmc_fsm.eliminate(sid);
            bmc_fsm.merge_parallel_transitions();
            println!("{}", bmc_fsm.to_dot_2(false, false));
        }
        println!("}}");
        Ok(&args_left[2..])
    } else {
        let mut bmc_fsm = BMCMachine::from_machine(fsm);
        println!("digraph {{\n  rankdir=\"LR\";");
        bmc_fsm.merge_parallel_transitions();
        println!("{}", bmc_fsm.to_dot_2(false, false));
        while let Some(sid) = bmc_fsm.choose_best_state() {
            eprintln!("Eliminating state {sid}");
            bmc_fsm.eliminate(sid);
            bmc_fsm.merge_parallel_transitions();
            println!("{}", bmc_fsm.to_dot_2(false, false));
        }
        println!("}}");
        Ok(args_left)
    }
}

fn cmd_backprop(args: &[String]) -> Result<&[String], CmdError> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"backprop\" command");
        return Err(CmdError::BadArgs);
    }
    let file = &args[0];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    if let Some(mut fsm) = validated(pars.parse_machine_file()) {
        fsm.epsilon_trans_closure();
        fsm.backward_propagation();
        fsm.remove_epsilon_trans();
        println!("{}", fsm.to_dot(false));
        Ok(&args[1..])
    } else {
        Err(CmdError::ExecError)
    }
}

fn cmd_forwardprop(args: &[String]) -> Result<&[String], CmdError> {
    if args.len() < 1 {
        eprintln!("error: missing argument to \"forwardprop\" command");
        return Err(CmdError::BadArgs);
    }
    let file = &args[0];
    let lex = Lexer::from_path(Path::new(file));
    let mut pars = Parser::new(lex);
    if let Some(mut fsm) = validated(pars.parse_machine_file()) {
        fsm.epsilon_trans_closure();
        fsm.forward_propagation();
        fsm.remove_epsilon_trans();
        println!("{}", fsm.to_dot(false));
        Ok(&args[1..])
    } else {
        Err(CmdError::ExecError)
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
        let cmd_res = if cmd == "pilot" {
            cmd_pilot(&args_left[1..])
        } else if cmd == "echo_mnet" {
            cmd_echo_mnet(&args_left[1..])
        } else if cmd == "echo_regex" {
            cmd_echo_regex(&args_left[1..])
        } else if cmd == "echo_fsm" {
            cmd_echo_fsm(&args_left[1..])
        } else if cmd == "berry_sethi" {
            cmd_berry_sethi(&args_left[1..])
        } else if cmd == "berry_sethi_fsm" {
            cmd_berry_sethi_fsm(&args_left[1..])
        } else if cmd == "bmc" {
            cmd_bmc(&args_left[1..])
        } else if cmd == "backprop" {
            cmd_backprop(&args_left[1..])
        } else if cmd == "forwardprop" {
            cmd_forwardprop(&args_left[1..])
        } else if cmd == "help" || cmd == "-h" || cmd == "--help" {
            banner();
            help();
            return ExitCode::SUCCESS;
        } else {
            eprintln!("error: invalid command \"{cmd}\"");
            Err(CmdError::BadArgs)
        };
        match cmd_res {
            Ok(new_args_left) => args_left = new_args_left,
            Err(CmdError::BadArgs) => {
                help();
                return ExitCode::FAILURE;
            }
            Err(_) => return ExitCode::FAILURE
        }
    }

    ExitCode::SUCCESS
}
