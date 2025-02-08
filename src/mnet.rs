mod validation;

pub use crate::fsm::*;

use std::collections::HashSet;

#[derive(Debug)]
pub struct MachineNet {
    pub machines: Vec<Machine>
}

impl MachineNet {
    pub fn try_lookup_machine(&self, machine: char) -> Option<&Machine> {
        self.machines.iter().find(|m| m.name == machine)
    }

    pub fn lookup_machine(&self, machine: char) -> &Machine {
        self.try_lookup_machine(machine).expect("machine does not exist")
    }

    pub fn try_lookup_state(&self, machine: char, id: i32) -> Option<&State> {
        let m = self.try_lookup_machine(machine)?;
        m.try_lookup_state(id)
    }

    pub fn lookup_state(&self, machine: char, id: i32) -> &State {
        self.lookup_machine(machine).lookup_state(id)
    }

    fn followers_impl(&self, machine: char, id: i32, visited: &mut HashSet<(char, i32)>, next: &HashSet<char>) -> HashSet<char> {
        if visited.contains(&(machine, id)) {
            return HashSet::new();
        }
        visited.insert((machine, id));
        let state = self.lookup_state(machine, id);
        let mut res: HashSet<char> = HashSet::new();
        if state.is_final {
            res.extend(next);
        }
        for t in &state.transitions {
            if !t.is_nonterminal() {
                res.insert(t.character);
            } else {
                let nextnext = self.followers_impl(machine, t.dest_id, visited, next);
                let rec_fol = self.followers_impl(t.character, 0, visited, &nextnext);
                res.extend(rec_fol);
            }
        }
        return res;
    }

    pub fn followers(&self, machine: char, id: i32, next: HashSet<char>) -> HashSet<char> {
        let mut visited: HashSet<(char, i32)> = HashSet::new();
        self.followers_impl(machine, id, &mut visited, &next)
    }
}
