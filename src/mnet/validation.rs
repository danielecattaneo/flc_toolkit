use crate::mnet::*;

impl MachineNet {
    fn validate_machine_count(&self) -> bool {
        if self.machines.len() == 0 {
            eprintln!("error: no machines in the machine net");
            false
        } else {
            true
        }
    }

    fn validate_start(&self) -> bool {
        // There must be a S-named machine
        if let None = self.machines.iter().find(|m| m.name == 'S') {
            eprintln!("error: axiom (machine named S) missing");
            false
        } else {
            true
        }
    }

    fn validate_not_reentrant(&self) -> bool {
        // There must be no incoming transitions to the initial state of a machine (from the same machine)
        let mut res = true;
        for m in &self.machines {
            for s in &m.states {
                for ts in &s.transitions {
                    if ts.dest_id == 0 {
                        eprintln!("error: machine {} re-entrant because of transition {}{} -{}-> 0{}", m.name, s.id, m.name, ts.character, m.name);
                        res = false;
                    }
                }
            }
        }
        res
    }

    fn validate_state_count(&self) -> bool {
        // All machines must have > 0 states
        let mut res = true;
        for m in &self.machines {
            if m.states.len() == 0 {
                eprintln!("error: machine {} has zero states", m.name);
                res = false;
            }
        }
        res
    }

    fn validate_single_initial_state(&self) -> bool {
        // The initial state must be state 0. All other states are not initial
        let mut res = true;
        for m in &self.machines {
            for s in &m.states {
                if s.is_initial && s.id != 0 {
                    eprintln!("error: state {}{} cannot be initial", m.name, s.id);
                    res = false;
                } else if s.id == 0 && !s.is_initial {
                    eprintln!("error: state {}{} must be initial", m.name, s.id);
                    res = false;
                }
            }
        }
        res
    }

    fn validate_any_final_state(&self) -> bool {
        let mut res = true;
        for m in &self.machines {
            if !m.states.iter().any(|s| s.is_final) {
                eprintln!("error: no final state in machine {}", m.name);
                res = false;
            }
        }
        res
    }

    fn validate_transitions(&self) -> bool {
        let mut res = true;
        for m in &self.machines {
            for s in &m.states {
                for (i, t) in s.transitions.iter().enumerate() {
                    if let None = m.try_lookup_state(t.dest_id) {
                        eprintln!("error: transition {}{} -{}-> {}{} goes to a non-existent state", m.name, s.id, t.character, m.name, t.dest_id);
                        res = false;
                    }
                    if t.is_nonterminal() {
                        if let None = self.try_lookup_machine(t.character) {
                            eprintln!("error: transition {}{} -{}-> ... has an invalid nonterminal", m.name, s.id, t.character);
                            res = false;
                        }
                    }
                    for tj in &s.transitions[i+1..] {
                        if t.character == tj.character {
                            eprintln!("error: multiple transitions {}{} -{}-> ...", m.name, s.id, t.character);
                            res = false;
                        }
                    }
                }
            }
        }
        res
    }

    pub fn validate(&self) -> bool {
        [
            self.validate_machine_count(),
            self.validate_start(),
            self.validate_not_reentrant(),
            self.validate_state_count(),
            self.validate_single_initial_state(),
            self.validate_any_final_state(),
            self.validate_transitions()
        ].into_iter().all(|v| v)
    }
}
