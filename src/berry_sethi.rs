use std::collections::*;

use crate::regex::*;
use crate::fsm::*;

#[derive(Debug, PartialEq, Eq)]
pub struct BSStateLabel {
    terminals: NumTermSet,
    is_final: bool
}

pub type BSState = BaseState<BSStateLabel, char>;
pub type BSMachine = BaseMachine<BSStateLabel, char>;

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

pub type NumFollowersMap = HashMap<NumTerm, NumTermSet>;

impl Regex {
    pub fn numbered_followers(&self) -> NumFollowersMap {
        let mut res = NumFollowersMap::new();
        for t in self.all_numbered() {
            res.insert(t, NumTermSet::new());
        }
        for (t, f) in self.numbered_digrams() {
            res.get_mut(&t).unwrap().insert(f);
        }
        res
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

pub fn berry_sethi(re: &Regex) -> BSMachine {
    let ini = re.numbered_initials();
    let dig = re.numbered_digrams();
    let fin = re.numbered_finals();
    let null = re.nullable();

    let mut res = BSMachine::new('M');
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
