pub mod dot_formatter;

use std::collections::VecDeque;
use std::collections::HashSet;
pub use crate::fsm::*;

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
            self.validate_state_count(),
            self.validate_single_initial_state(),
            self.validate_any_final_state(),
            self.validate_transitions()
        ].into_iter().all(|v| v)
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

    fn followers(&self, machine: char, id: i32, next: HashSet<char>) -> HashSet<char> {
        let mut visited: HashSet<(char, i32)> = HashSet::new();
        self.followers_impl(machine, id, &mut visited, &next)
    }
}


#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Candidate {
    pub machine: char,
    pub state: i32,
    pub lookahead: char,
    pub is_final: bool
}

impl Candidate {
    fn to_string(&self) -> String {
        let state = if self.is_final {
            format!("({}{})", self.state, self.machine)
        } else {
            format!("{}{}", self.state, self.machine)
        };
        format!("<{}, {}>", state, self.lookahead)
    }

    fn is_base(&self) -> bool {
        self.state != 0
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct PilotTransition {
    pub character: char,
    pub dest_id: i32,
    pub multiplicity: i32,
    pub candidate_map: Vec<(usize, usize)>
}

pub struct ShiftReduceConflict {
    pub state_id: i32,
    pub candidate_idx: usize
}

pub struct ReduceReduceConflict {
    pub state_id: i32,
    pub candidate_1_idx: usize,
    pub candidate_2_idx: usize
}

pub struct ConvergenceConflict {
    pub state_1_id: i32,
    pub candidate_1_1_idx: usize,
    pub candidate_1_2_idx: usize,
    pub transition_char: char,
    pub state_2_id: i32,
    pub candidate_2_idx: usize
}

#[derive(Debug, Clone)]
pub struct PilotState {
    pub id: i32,
    pub candidates: Vec<Candidate>,
    pub transitions: Vec<PilotTransition>
}

impl PilotState {
    pub fn base_set(&self) -> HashSet<&Candidate> {
        self.candidates.iter().filter(|x| x.is_base()).collect::<HashSet<_>>()
    }
    
    pub fn is_equivalent(&self, other: &PilotState) -> bool {
        let my_base = self.base_set();
        let other_base = other.base_set();
        return my_base == other_base;
    }

    pub fn shift_reduce_conflicts(&self) -> Vec<ShiftReduceConflict> {
        let outgoing: HashSet<char> = self.transitions.iter().map(|trans| {
            trans.character
        }).collect();
        self.candidates.iter().enumerate().filter_map(|(i, cand)| {
            if cand.is_final && outgoing.contains(&cand.lookahead) {
                Some(ShiftReduceConflict{state_id:self.id, candidate_idx:i})
            } else {
                None
            }
        }).collect()
    }

    pub fn reduce_reduce_conflicts(&self) -> Vec<ReduceReduceConflict> {
        let mut res: Vec<ReduceReduceConflict> = Vec::new();
        for i in 0 .. self.candidates.len() {
            for j in i+1 .. self.candidates.len() {
                let ci = &self.candidates[i];
                let cj = &self.candidates[j];
                if ci.is_final && cj.is_final && ci.lookahead == cj.lookahead {
                    res.push(ReduceReduceConflict{state_id: self.id, candidate_1_idx:i, candidate_2_idx:j});
                }
            }
        }
        res
    }

    pub fn convergence_conflicts(&self) -> Vec<ConvergenceConflict> {
        let mut res: Vec<ConvergenceConflict> = Vec::new();
        for t in &self.transitions {
            for (i, (i_s, i_d)) in t.candidate_map.iter().enumerate() {
                for (j_s, j_d) in t.candidate_map[i+1 ..].iter() {
                    if i_d == j_d {
                        res.push(ConvergenceConflict{
                            state_1_id:self.id,
                            candidate_1_1_idx:*i_s,
                            candidate_1_2_idx:*j_s,
                            transition_char:t.character,
                            state_2_id:t.dest_id,
                            candidate_2_idx:*i_d});
                    }
                }
            }
        }
        res
    }
}

#[derive(Debug)]
pub struct Pilot {
    pub states: Vec<PilotState>
}

impl Pilot {
    fn lookup_state_mut(&mut self, id: i32) -> &mut PilotState {
        self.states.iter_mut().find(|s| s.id == id).expect("state does not exist")
    }

    pub fn lookup_state(&self, id: i32) -> &PilotState {
        self.states.iter().find(|s| s.id == id).expect("state does not exist")
    }

    fn insert(&mut self, mut new: PilotState, net: &MachineNet) -> i32 {
        if let Some(s) = self.states.iter().find(|s| s.is_equivalent(&new)) {
            return s.id;
        }
        let id = self.states.len() as i32;
        new.id = id;
        closure(&mut new, net);
        self.states.push(new);
        return id;
    }

    pub fn print_shift_reduce_conflict(&self, c: &ShiftReduceConflict) {
        let s = c.state_id;
        let candidate = &self.lookup_state(s).candidates[c.candidate_idx];
        let c = candidate.to_string();
        let edge = candidate.lookahead;
        eprintln!("state I{s}: shift-reduce conflict between {c} and outgoing edge '{edge}'");
    }

    pub fn print_reduce_reduce_conflict(&self, c: &ReduceReduceConflict) {
        let s = c.state_id;
        let c1 = self.lookup_state(s).candidates[c.candidate_1_idx].to_string();
        let c2 = self.lookup_state(s).candidates[c.candidate_2_idx].to_string();
        eprintln!("state I{s}: reduce-reduce conflict between {c1} and {c2}");
    }
    
    pub fn print_convergence_conflict(&self, c: &ConvergenceConflict) {
        let s1 = c.state_1_id;
        let c1 = self.lookup_state(s1).candidates[c.candidate_1_1_idx].to_string();
        let c2 = self.lookup_state(s1).candidates[c.candidate_1_2_idx].to_string();
        let ts = c.transition_char;
        let s2 = c.state_2_id;
        let c3 = self.lookup_state(s2).candidates[c.candidate_2_idx].to_string();
        eprintln!("transition I{s1} -{ts}-> I{s2}: convergence conflict as both {c1} and {c2} shift to {c3}");
    }

    pub fn print_conflicts(&self) {
        let mut n_confl = 0;
        for state in &self.states {
            let sr_confl = state.shift_reduce_conflicts();
            for confl in &sr_confl {
                self.print_shift_reduce_conflict(confl);
                n_confl += 1;
            }
            let rr_confl = state.reduce_reduce_conflicts();
            for confl in &rr_confl {
                self.print_reduce_reduce_conflict(confl);
                n_confl += 1;
            }
            let c_confl = state.convergence_conflicts();
            for confl in &c_confl {
                self.print_convergence_conflict(confl);
                n_confl += 1;
            }
        }
        if n_confl == 0 {
            eprintln!("no conflicts");
        }
    }
}


fn closure(state: &mut PilotState, net: &MachineNet) {
    let mut candidate_id: usize = 0;
    while candidate_id < state.candidates.len() {
        let c = state.candidates[candidate_id];
        let mstate = net.lookup_state(c.machine, c.state);
        for t in &mstate.transitions {
            if !t.is_nonterminal() {
                continue;
            }
            let ini = net.followers(c.machine, t.dest_id, HashSet::from([c.lookahead]));
            for ch in ini {
                let dest_state = net.lookup_state(t.character, 0);
                let c2 = Candidate{machine:t.character, state:0, lookahead:ch, is_final:dest_state.is_final};
                if !state.candidates.contains(&c2) {
                    state.candidates.push(c2);
                }
            }
        }
        candidate_id += 1;
    }
}

fn collect_transitions(state: &PilotState, net: &MachineNet) -> Vec<char> {
    let mut res: HashSet<char> = HashSet::new();
    for c in &state.candidates {
        let mstate = net.lookup_state(c.machine, c.state);
        for t in &mstate.transitions {
            res.insert(t.character);
        }
    }
    let mut vec_res = Vec::from_iter(res.into_iter());
    vec_res.sort();
    return vec_res;
}

fn shift_candidate(c: &Candidate, net: &MachineNet, next: char) -> Option<Candidate> {
    let mstate = net.lookup_state(c.machine, c.state);
    for t in &mstate.transitions {
        if t.character == next {
            let dest_state = net.lookup_state(c.machine, t.dest_id);
            return Some(Candidate{machine:c.machine, state:t.dest_id, lookahead:c.lookahead, is_final:dest_state.is_final});
        }
    }
    return None;
}

fn shift(state: &PilotState, net: &MachineNet, character: char) -> (PilotTransition, PilotState) {
    let mut orig_states: HashSet<(char, i32)> = HashSet::new();
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut candidate_map: Vec<(usize, usize)> = Vec::new();
    for (i, c) in state.candidates.iter().enumerate() {
        if let Some(new) = shift_candidate(c, net, character) {
            orig_states.insert((c.machine, c.state));
            if let Some(j) = candidates.iter().position(|other| new == *other) {
                // convergence conflicts hatch here
                candidate_map.push((i, j));
            } else {
                candidate_map.push((i, candidates.len()));
                candidates.push(new);
            }
        }
    }
    let multiplicity = orig_states.len() as i32;
    (PilotTransition{character, dest_id:-1, multiplicity, candidate_map}, PilotState{id:-1, candidates, transitions:vec![]})
}

pub fn create_pilot(net: &MachineNet) -> Pilot {
    let first_state = net.lookup_state('S', 0);
    let init_candidate = Candidate{machine:'S', state:0, lookahead:'$', is_final:first_state.is_final};
    let init_state = PilotState{id:0, candidates:vec![init_candidate], transitions:vec![]};
    let mut pilot = Pilot{states: vec![]};

    let mut worklist = VecDeque::from([pilot.insert(init_state, net)]);
    let mut visited: HashSet<i32> = HashSet::new();
    while !worklist.is_empty() {
        let state_id = worklist.pop_front().unwrap();
        if visited.contains(&state_id) {
            continue;
        }
        visited.insert(state_id);

        let state = pilot.lookup_state(state_id);
        let future_xions = collect_transitions(state, net);
        let shifts: Vec<_> = future_xions.into_iter().map(|c| {
            shift(&state, net, c)
        }).collect();
        let xions: Vec<_> = shifts.into_iter().map(|(mut trans, maybe_new_state)| {
            let id = pilot.insert(maybe_new_state, net);
            trans.dest_id = id;
            trans
        }).collect();
        worklist.extend(xions.iter().map(|xion| xion.dest_id));
        pilot.lookup_state_mut(state_id).transitions = xions;
    }

    return pilot;
}
