use std::{collections::HashSet, collections::HashMap};

use itertools::Itertools;
use crate::fsm::*;

#[derive(Eq, Debug)]
pub struct DistinguishablePair {
    state_1: i32,
    state_2: i32,
}

impl PartialEq for DistinguishablePair {
    fn eq(&self, other: &Self) -> bool {
        self.state_1 == other.state_1 && self.state_2 == other.state_2
    }
}

impl std::hash::Hash for DistinguishablePair {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state_1.hash(state);
        self.state_2.hash(state);
    }
}

pub type DistTable = HashSet<DistinguishablePair>;

trait DistTableComparison {
    fn insert_state_pair(&mut self, s: i32, t: i32);
    fn find_state_pair(&self, s: i32, t: i32) -> bool;
}

impl DistTableComparison for DistTable {
    fn insert_state_pair(&mut self, s: i32, t: i32) {
        if s < t {
            self.insert(DistinguishablePair{state_1: s, state_2: t});
        } else {
            self.insert(DistinguishablePair{state_1: t, state_2: s});
        }
    }

    fn find_state_pair(&self, s: i32, t: i32) -> bool {
        if s < t {
            self.iter().find(|e| **e == DistinguishablePair{state_1: s, state_2: t}).is_some() 
        } else {
            self.iter().find(|e| **e == DistinguishablePair{state_1: t, state_2: s}).is_some() 
        }
    }
}

fn bron_kerbosch(res: &mut Vec<HashSet<i32>>, edges: &HashSet<(i32, i32)>, r: HashSet<i32>, mut p: HashSet<i32>, mut x: HashSet<i32>) {
    if p.is_empty() && x.is_empty() {
        res.push(r.clone());
    }
    for v in p.clone() {
        let v_set = HashSet::from([v]);
        let v_neigh: HashSet<i32> = edges.iter().filter_map(|(s, t)| {
            if *s == v {
                Some(*t)
            } else if *t == v {
                Some(*s)
            } else {
                None
            }
        }).collect();
        bron_kerbosch(res, edges,
            r.union(&v_set).cloned().collect(),
            p.intersection(&v_neigh).cloned().collect(),
            x.intersection(&v_neigh).cloned().collect());
        p.remove(&v);
        x.insert(v);
    }
}

impl Machine {
    pub fn dist_table_len_0(&self) -> DistTable {
        let mut res: DistTable = DistTable::new();
        for (s, t) in self.states.iter().tuple_combinations() {
            if s.is_final != t.is_final {
                res.insert_state_pair(s.id, t.id);
            }
        }
        res
    }

    pub fn dist_table_update(&self, res: &mut DistTable) -> usize {
        let mut count: usize = 0;
        for (s, t) in self.states.iter().tuple_combinations() {
            if res.find_state_pair(s.id, t.id) {
                continue;
            }
            let distinguishable = s.transitions.iter().any(|sts| {
                let maybe_tts = t.transitions.iter().find(|t| t.label == sts.label);
                if let Some(tts) = maybe_tts {
                    res.find_state_pair(sts.dest_id, tts.dest_id)
                } else {
                    true
                }
            }) || t.transitions.iter().any(|tts| {
                let maybe_sts = s.transitions.iter().find(|s| s.label == tts.label);
                if let Some(sts) = maybe_sts {
                    res.find_state_pair(tts.dest_id, sts.dest_id)
                } else {
                    true
                }
            });
            if distinguishable {
                res.insert_state_pair(s.id, t.id);
                count += 1;
            }
        }
        count
    }

    pub fn cliques(&self, dist: &DistTable) -> Vec<HashSet<i32>> {
        let vertices: HashSet<i32> = self.states.iter().map(|s| s.id).collect();
        let edges: HashSet<(i32, i32)> = self.states.iter().tuple_combinations().filter_map(|(s, t)| {
            if dist.find_state_pair(s.id, t.id) {
                None
            } else {
                Some((s.id, t.id))
            }
        }).collect();
        let mut res: Vec<HashSet<i32>> = vec![];
        bron_kerbosch(&mut res, &edges, HashSet::new(), vertices, HashSet::new());
        res
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MinimizedStateLabel {
    original_ids: Vec<i32>
}

pub type MinimizedState = BaseState<MinimizedStateLabel, char>;
pub type MinimizedMachine = BaseMachine<char, MinimizedStateLabel, char>;

impl DotFormat for MinimizedStateLabel {
    fn to_dot(&self, _: bool) -> String {
        let body = self.original_ids.iter().map(|id| format!("{id}")).join(" ");
        format!("\"{body}\"")
    }
}

impl MinimizedMachine {
    pub fn from_machine_and_equiv_sets(m: &Machine, sets: &Vec<HashSet<i32>>) -> MinimizedMachine {
        let sorted_sets: Vec<_> = sets.iter().sorted_by_key(|s| s.iter().min()).collect();
        let old_to_new: HashMap<i32, i32> = sorted_sets.iter().enumerate().flat_map(|(i, s)| {
            s.iter().map(|j| (*j, i as i32)).collect::<Vec<_>>()
        }).collect();
        let new_states: Vec<_> = sorted_sets.iter().enumerate().map(|(id, set)| {
            let is_initial = set.iter().fold(false, |acc, sid| acc || m.lookup_state(*sid).is_initial);
            let random_state = m.lookup_state(*set.iter().next().unwrap()); // any will do
            let is_final = random_state.is_final;
            let transitions: Vec<_> = random_state.transitions.iter().map(|ts| {
                let new_dest = old_to_new[&ts.dest_id];
                Transition{label: ts.label, dest_id: new_dest}
            }).collect();
            let original_ids: Vec<_> = set.iter().cloned().sorted().collect();
            let label = MinimizedStateLabel{original_ids};
            MinimizedState{id: id as i32, label, transitions, is_initial, is_final}
        }).collect();
        MinimizedMachine{label: m.label, states: new_states}
    }
}
