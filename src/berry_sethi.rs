use std::collections::*;

use crate::regex::*;
use crate::fsm::*;

#[derive(Debug, PartialEq, Eq)]
pub struct BSStateLabel {
    terminals: HashSet<(char, usize)>,
    is_final: bool
}

pub type BSState = BaseState<BSStateLabel, char>;
pub type BSMachine = BaseMachine<BSStateLabel, char>;

impl DotFormat for BSStateLabel {
    fn to_dot(&self, _: bool) -> String {
        let mut l: Vec<_> = self.terminals.iter().collect();
        l.sort();
        let mut ll: Vec<_> = l.into_iter().map(| &(c, i) | {
            format!("{}<sub>{}</sub>", c, i)
        }).collect();
        if self.is_final {
            ll.push("⊣".to_string());
        }
        ll.join(",")
    }
}

impl Regex {
    pub fn numbered_followers(&self) -> HashMap<(char, usize), HashSet<(char, usize)>> {
        let mut res: HashMap<(char, usize), HashSet<(char, usize)>> = HashMap::new();
        for (c, i) in self.all_numbered() {
            res.insert((c, i), HashSet::new());
        }
        for ((c, i), (d, j)) in self.numbered_digrams() {
            let set = res.get_mut(&(c, i)).unwrap();
            set.insert((d, j));
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
        let mut res: Vec<char> = self.label.terminals.iter().map(| (c, _) | *c).collect();
        res.sort();
        res.dedup();
        res
    }

    fn shift(&self, c: char, dig: &HashSet<((char, usize), (char, usize))>, fin: &HashSet<(char, usize)>) -> BSStateLabel {
        let my_terminals: HashSet<_> = self.label.terminals.iter().filter(| (a, _) | *a == c).collect();
        let terminals = dig.iter().filter_map(| ((a, i), (d, j)) | {
            if my_terminals.contains(&(*a, *i)) { Some((*d, *j)) } else { None }
        });
        let is_final = my_terminals.iter().any(| (d, i) | fin.contains(&(*d, *i)) );
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

    let mut res = BSMachine::new('M');
    let init_label = BSStateLabel{ terminals: ini, is_final: re.nullable() };

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
