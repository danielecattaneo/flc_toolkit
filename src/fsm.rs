mod dot_formatter;
mod validation;

use std::collections::VecDeque;
use std::collections::HashSet;

use crate::reg_lang::*;

pub trait DotFormat {
    fn to_dot(&self, _: bool) -> String;
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct BaseTransition<L> {
    pub label: L,
    pub dest_id: i32
}

pub type Transition = BaseTransition<char>;

impl Transition {
    pub fn is_nonterminal(&self) -> bool {
        self.label.is_ascii_uppercase()
    }

    pub fn is_epsilon(&self) -> bool {
        self.label == '_'
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StateLabel {
    pub id: i32,
    pub m_name: char
}

#[derive(Debug)]
pub struct BaseState<SL, TL> {
    pub id: i32,
    pub label: SL,
    pub transitions: Vec<BaseTransition<TL>>,
    pub is_initial: bool,
    pub is_final: bool,
}

pub type State = BaseState<StateLabel, char>;

#[derive(Debug)]
pub struct BaseMachine<ML, SL, TL> {
    pub label: ML,
    pub states: Vec<BaseState<SL, TL>>
}

pub type Machine = BaseMachine<char, StateLabel, char>;

impl<ML, SL, TL> BaseMachine<ML, SL, TL> {
    pub fn new(name: ML) -> BaseMachine<ML, SL, TL> {
        BaseMachine::<ML, SL, TL>{ label: name, states: Vec::new() }
    }

    pub fn try_lookup_state(&self, id: i32) -> Option<&BaseState<SL, TL>> {
        self.states.iter().find(|s| s.id == id)
    }

    pub fn lookup_state(&self, id: i32) -> &BaseState<SL, TL> {
        self.try_lookup_state(id).expect("state does not exist")
    }

    pub fn lookup_state_mut(&mut self, id: i32) -> &mut BaseState<SL, TL> {
        self.states.iter_mut().find(|s| s.id == id).expect("state does not exist")
    }

    pub fn initial_states_ids(&self) -> Vec<i32> {
        self.states.iter().filter(|s| s.is_initial).map(|s| s.id).collect()
    }

    pub fn final_states_ids(&self) -> Vec<i32> {
        self.states.iter().filter(|s| s.is_final).map(|s| s.id).collect()
    }

    pub fn transition_start(&self, t: &BaseTransition<TL>) -> i32 {
        self.states.iter().find(|s| {
            s.transitions.iter().any(|t2| std::ptr::eq(t2, t))
        }).unwrap().id
    }

    pub fn transitions_to(&self, sid: i32) -> Vec<(i32, &BaseTransition<TL>)> {
        self.states.iter().flat_map(|s| {
            s.transitions.iter().filter_map(|t2| {
                if t2.dest_id == sid { Some((s.id, t2)) } else { None }
            })
        }).collect()
    }
}

pub type NumTransition = BaseTransition<NumTerm>;
pub type NumMachine = BaseMachine<char, StateLabel, NumTerm>;

impl NumTransition {
    pub fn is_epsilon(&self) -> bool {
        self.label.c == '_'
    }
}

impl NumMachine {
    pub fn from_machine(old_m: Machine) -> NumMachine {
        let mut i = 0;
        let states = old_m.states.iter().map(|old_state| {
            let mut new_ts: Vec<BaseTransition<NumTerm>> = Vec::new();
            for old_t in &old_state.transitions {
                let dest_id = old_t.dest_id;
                let label = if old_t.label == '_' {
                    NumTerm::new('_', 0)
                } else {
                    i += 1;
                    NumTerm::new(old_t.label, i)
                };
                new_ts.push(BaseTransition::<NumTerm>{ label, dest_id });
            }
            BaseState::<StateLabel, NumTerm>{
                id: old_state.id,
                label: old_state.label,
                transitions: new_ts,
                is_final: old_state.is_final,
                is_initial: old_state.is_initial}
        });
        NumMachine{ label: old_m.label, states: states.collect() }
    }

    fn nullable_from(&self, active: &Vec<i32>) -> bool {
        let mut cur_active: VecDeque<_> = active.iter().cloned().collect();
        let mut visited: HashSet<i32> = HashSet::new();
        while let Some(sid) = cur_active.pop_front() {
            if visited.contains(&sid) {
                continue;
            }
            visited.insert(sid);
            let s = self.lookup_state(sid);
            if s.is_final {
                return true;
            }
            for t in &s.transitions {
                if t.is_epsilon() {
                    cur_active.push_back(t.dest_id);
                }
            }
        }
        false
    }

    fn numbered_initials_from(&self, active: &Vec<i32>) -> NumTermSet {
        let mut cur_active: VecDeque<_> = active.iter().cloned().collect();
        let mut visited: HashSet<i32> = HashSet::new();
        let mut res = NumTermSet::new();
        while let Some(sid) = cur_active.pop_front() {
            if visited.contains(&sid) {
                continue;
            }
            visited.insert(sid);
            let s = self.lookup_state(sid);
            for t in &s.transitions {
                if t.is_epsilon() {
                    cur_active.push_back(t.dest_id);
                } else {
                    res.insert(t.label);
                }
            }
        }
        res
    }

    fn numbered_finals_to(&self, active: &Vec<i32>) -> NumTermSet {
        let mut cur_active: VecDeque<_> = active.iter().cloned().collect();
        let mut visited: HashSet<i32> = HashSet::new();
        let mut res = NumTermSet::new();
        while let Some(sid) = cur_active.pop_front() {
            if visited.contains(&sid) {
                continue;
            }
            visited.insert(sid);
            for (s0, t) in self.transitions_to(sid) {
                if t.is_epsilon() {
                    cur_active.push_back(s0);
                } else {
                    res.insert(t.label);
                }
            }
        }
        res
    }
}

impl NumLocalSets for NumMachine {
    fn all_numbered(&self) -> NumTermSet {
        self.states.iter().flat_map(|s| {
            s.transitions.iter().filter_map(|t| if t.label.c != '_' { Some(t.label) } else { None })
        }).collect()
    }

    fn nullable(&self) -> bool {
        let a: Vec<_> = self.initial_states_ids();
        self.nullable_from(&a)
    }

    fn numbered_initials(&self) -> NumTermSet {
        let a: Vec<_> = self.initial_states_ids();
        self.numbered_initials_from(&a)
    }

    fn numbered_finals(&self) -> NumTermSet {
        let a: Vec<_> = self.final_states_ids();
        self.numbered_finals_to(&a)
    }

    fn numbered_digrams(&self) -> NumDigramsSet {
        self.states.iter().flat_map(|s| {
            let prev = self.numbered_finals_to(&vec![s.id]);
            let next = self.numbered_initials_from(&vec![s.id]);
            set_prod(&prev, &next)
        }).collect()
    }
}
