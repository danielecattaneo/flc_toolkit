use std::mem::replace;

use crate::lexer::*;
use crate::mnet::*;

pub struct Parser {
    lexer: Lexer,
    lookahead: Option<Token>
}

macro_rules! token {
    ($p:pat_param) => (
        Some(Token{value:$p, ..})
    );
}

macro_rules! expect {
    ($self:expr, $p:pat_param, $err:expr, $b:block) => (
        if let token!($p) = $self.lookahead {
            let res = $b;
            $self.advance();
            res
        } else {
            $self.emit_error($err);
            return None;
        }
    );
    ($self:expr, $p:pat_param, $err:expr) => (
        expect!($self, $p, $err, {})
    );
}

macro_rules! accept {
    ($self:expr, $p:pat_param) => (
        if let token!($p) = $self.lookahead {
            $self.advance()
        } else {
            None
        }
    );
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Parser {
        let lookahead = lexer.next();
        Parser{lexer, lookahead}
    }

    fn emit_error(&self, s: &str) {
        if let Some(look) = &self.lookahead {
            look.location.emit_error(s);
        } else {
            eprintln!("error: {}", s);
        }
    }

    fn advance(&mut self) -> Option<Token> {
        if let Some(_) = self.lookahead {
            replace(&mut self.lookahead, self.lexer.next())
        } else {
            None
        }
    }

    fn parse_state(&mut self) -> Option<State> {
        expect!(self, TokenValue::KwState, "expected a state");
        let id = expect!(self, TokenValue::Number(num), "expected the state identifier", { num });
        let mut state = State{id, transitions:vec![], is_initial:false, is_final:false};
        expect!(self, TokenValue::LBrace, "expected a state body enclosed in {}");
        loop {
            if let Some(_) = accept!(self, TokenValue::KwInitial) {
                expect!(self, TokenValue::Semi, "expected semicolon");
                state.is_initial = true;
            } else if let Some(_) = accept!(self, TokenValue::KwFinal) {
                expect!(self, TokenValue::Semi, "expected semicolon");
                state.is_final = true;
            } else if let token!(TokenValue::Ident(character)) = self.lookahead {
                self.advance();
                expect!(self, TokenValue::RArrow, "expected -> after transition character");
                expect!(self, TokenValue::Number(dest_id), "expected transition destination state", {
                    let trans = Transition{character, dest_id};
                    state.transitions.push(trans);
                });
                expect!(self, TokenValue::Semi, "expected semicolon");
            } else {
                break;
            }
        }
        expect!(self, TokenValue::RBrace, "expected a transition or a state property");
        Some(state)
    }

    fn parse_machine(&mut self) -> Option<Machine> {
        expect!(self, TokenValue::KwMachine, "expected a machine");
        let name = expect!(self, TokenValue::Ident(name), "expected a machine name", {
            if !name.is_ascii_uppercase() {
                self.emit_error("machine name must be ASCII uppercase");
                return None;
            } else {
                name
            }
        });
        let mut machine = Machine{name, states: vec![]};
        expect!(self, TokenValue::LBrace, "expected a machine body enclosed by {}");
        while let token!(TokenValue::KwState) = self.lookahead {
            if let Some(state) = self.parse_state() {
                machine.states.push(state);
            } else {
                return None;
            }
        }
        expect!(self, TokenValue::RBrace, "expected a list of states");
        Some(machine)
    }

    pub fn parse_mnet(&mut self) -> Option<MachineNet> {
        let mut machines: Vec<Machine> = Vec::new();
        expect!(self, TokenValue::KwMNet, "expected a machine net");
        expect!(self, TokenValue::LBrace, "expected a machine net body enclosed by {}");
        while let token!(TokenValue::KwMachine) = self.lookahead {
            if let Some(mach) = self.parse_machine() {
                machines.push(mach);
            } else {
                return None;
            }
        }
        expect!(self, TokenValue::RBrace, "unmatched }");
        Some(MachineNet{machines})
    }
}
