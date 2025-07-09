pub mod dot_formatter;
pub mod conflicts;

use core::fmt;
use std::collections::VecDeque;
use std::collections::HashSet;
pub use crate::mnet::*;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Candidate {
    pub machine: char,
    pub state: i32,
    pub lookahead: char,
    pub is_final: bool
}

impl fmt::Display for Candidate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state = if self.is_final {
            format!("({}{})", self.state, self.machine)
        } else {
            format!("{}{}", self.state, self.machine)
        };
        write!(f, "<{}, {}>", state, self.lookahead)
    }
}

impl Candidate {
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
        my_base == other_base
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
        id
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
                let dest_state = net.lookup_state(t.label, 0);
                let c2 = Candidate{machine:t.label, state:0, lookahead:ch, is_final:dest_state.is_final};
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
            res.insert(t.label);
        }
    }
    let mut vec_res = Vec::from_iter(res);
    vec_res.sort();
    vec_res
}

fn shift_candidate(c: &Candidate, net: &MachineNet, next: char) -> Option<Candidate> {
    let mstate = net.lookup_state(c.machine, c.state);
    for t in &mstate.transitions {
        if t.label == next {
            let dest_state = net.lookup_state(c.machine, t.dest_id);
            return Some(Candidate{machine:c.machine, state:t.dest_id, lookahead:c.lookahead, is_final:dest_state.is_final});
        }
    }
    None
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
            shift(state, net, c)
        }).collect();
        let xions: Vec<_> = shifts.into_iter().map(|(mut trans, maybe_new_state)| {
            let id = pilot.insert(maybe_new_state, net);
            trans.dest_id = id;
            trans
        }).collect();
        worklist.extend(xions.iter().map(|xion| xion.dest_id));
        pilot.lookup_state_mut(state_id).transitions = xions;
    }

    pilot
}
