use std::collections::*;

use crate::reg_lang::*;
use crate::fsm::*;

#[derive(Debug, PartialEq, Eq)]
pub struct BSStateLabel {
    terminals: NumTermSet,
    is_final: bool
}

pub type BSState = BaseState<BSStateLabel, char>;
pub type BSMachine = BaseMachine<char, BSStateLabel, char>;

impl DotFormat for BSStateLabel {
    fn to_dot(&self, _: bool) -> String {
        let mut l: Vec<_> = self.terminals.iter().collect();
        l.sort();
        let mut ll: Vec<_> = l.into_iter().map(| &t | {
            format!("{}<sub>{}</sub>", t.c, t.i)
        }).collect();
        if self.is_final {
            ll.push("⊣".to_string());
        }
        ll.join(",")
    }
}

impl BSState {
    fn new(label: BSStateLabel, is_initial: bool) -> BSState {
        let is_final = label.is_final;
        BSState{ id: -1, label, transitions: Vec::new(), is_initial, is_final }
    }

    fn collect_transitions(&self) -> Vec<char> {
        let mut res: Vec<char> = self.label.terminals.iter().map(|t| t.c).collect();
        res.sort();
        res.dedup();
        res
    }

    fn shift(&self, c: char, dig: &NumDigramsSet, fin: &NumTermSet) -> BSStateLabel {
        let my_terminals: HashSet<_> = self.label.terminals.iter().filter(|t| t.c == c).collect();
        let terminals = dig.iter().filter_map(|(t, f)| {
            if my_terminals.contains(t) { Some(*f) } else { None }
        });
        let is_final = my_terminals.iter().any(|t| fin.contains(t));
        BSStateLabel{ terminals: terminals.collect(), is_final }
    }
}

impl BSMachine {
    fn try_lookup_state_by_label(&self, label: &BSStateLabel) -> Option<&BSState> {
        self.states.iter().find(|s| s.label == *label)
    }

    fn insert(&mut self, mut new: BSState) -> i32 {
        if let Some(s) = self.try_lookup_state_by_label(&new.label) {
            return s.id;
        }
        let id = self.states.len() as i32;
        new.id = id;
        self.states.push(new);
        return id;
    }
}

fn berry_sethi_impl(ini: NumTermSet, dig: NumDigramsSet, fin: NumTermSet, null: bool) -> BSMachine {
    let mut res = BSMachine::new('b');
    let init_label = BSStateLabel{ terminals: ini, is_final: null };

    let mut worklist = VecDeque::from([res.insert(BSState::new(init_label, true))]);
    let mut visited: HashSet<i32> = HashSet::new();
    while !worklist.is_empty() {
        let state_id = worklist.pop_front().unwrap();
        if visited.contains(&state_id) {
            continue;
        }
        visited.insert(state_id);

        let state = res.lookup_state(state_id);
        let future_xions = state.collect_transitions();
        let shifts: Vec<_> = future_xions.into_iter().map(|c| {
            (c, state.shift(c, &dig, &fin))
        }).collect();
        let xions: Vec<_> = shifts.into_iter().map(| (c, label) | {
            let maybe_new_state = BSState::new(label, false);
            let id = res.insert(maybe_new_state);
            Transition{ label: c, dest_id: id }
        }).collect();
        worklist.extend(xions.iter().map(|xion| xion.dest_id));
        res.lookup_state_mut(state_id).transitions = xions;
    }
    res
}

pub fn berry_sethi<T: NumLocalSets>(x: &T) -> BSMachine {
    let ini = x.numbered_initials();
    let dig = x.numbered_digrams();
    let fin = x.numbered_finals();
    let null = x.nullable();
    berry_sethi_impl(ini, dig, fin, null)
}
