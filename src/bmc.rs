use std::fmt;

use crate::reg_lang::*;
use crate::regex::*;
use crate::fsm::*;

pub struct BMCMachineLabel {
    name: char,
    gen_id: usize
}

impl DotFormat for BMCMachineLabel {
    fn to_dot(&self, _: bool) -> String {
        format!("<{}<sub>{}</sub>>", self.name, self.gen_id)
    }
}

impl fmt::Display for BMCMachineLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.name, self.gen_id)
    }
}

pub type BMCTransition = BaseTransition<Regex>;
pub type BMCState = BaseState<StateLabel, Regex>;
pub type BMCMachine = BaseMachine<BMCMachineLabel, StateLabel, Regex>;

impl BMCMachine {
    pub fn from_machine(old_m: Machine) -> BMCMachine {
        let ini = BMCState{
            id: -1,
            label: StateLabel{ id: -1, m_name: old_m.label },
            transitions: old_m.states.iter().filter_map(|old_state| {
                if old_state.is_initial {
                    Some(BMCTransition{
                        dest_id: old_state.id,
                        label: Regex::Null
                    })
                } else {
                    None
                }
            }).collect(),
            is_initial: true,
            is_final: false
        };
        let fin = BMCState{
            id: -2,
            label: StateLabel{ id: -2, m_name: old_m.label },
            transitions: vec![],
            is_initial: false,
            is_final: true
        };
        let mut i = 0;
        let mut states: Vec<_> = old_m.states.iter().map(|old_state| {
            let mut new_ts: Vec<BMCTransition> = Vec::new();
            for old_t in &old_state.transitions {
                let dest_id = old_t.dest_id;
                let label = if old_t.label == '_' {
                    Regex::Null
                } else {
                    i += 1;
                    Regex::Literal(NumTerm { c: old_t.label, i })
                };
                new_ts.push(BMCTransition{ label, dest_id });
            }
            if old_state.is_final {
                new_ts.push(BMCTransition{ label: Regex::Null, dest_id: -2 });
            }
            BMCState{
                id: old_state.id,
                label: old_state.label,
                transitions: new_ts,
                is_final: false,
                is_initial: false}
        }).collect();
        states.push(ini);
        states.push(fin);
        BMCMachine{ label: BMCMachineLabel{ name: old_m.label, gen_id: 1 }, states }
    }

    pub fn merge_parallel_transitions(&mut self) {
        for s in &mut self.states {
            let mut i = 0;
            while i < s.transitions.len() {
                let mut j = i + 1;
                while j < s.transitions.len() {
                    if s.transitions[i].dest_id == s.transitions[j].dest_id {
                        let re1 = Box::new(s.transitions[i].label.clone());
                        let re2 = Box::new(s.transitions[j].label.clone());
                        s.transitions[i].label = Regex::Union(re1, re2);
                        s.transitions.remove(j);
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }
    }

    pub fn choose_best_state(&self) -> Option<i32> {
        self.states.iter().filter(|s| {
            !(s.is_final || s.is_initial)
        }).min_by_key(|s| {
            self.transitions_to(s.id).iter().filter(|(ss, _)| *ss != s.id).count()
                * s.transitions.iter().filter(|t| t.dest_id != s.id).count()
        }).map(|s| s.id)
    }

    pub fn eliminate(&mut self, s_id: i32) {
        let t_in = self.transitions_to(s_id);
        let s = self.lookup_state(s_id);
        let t_out: Vec<_> = s.transitions.iter().filter(|t| t.dest_id != s_id).collect();
        let t_loop: Vec<_> = s.transitions.iter().filter(|t| t.dest_id == s_id).collect();
        assert!(t_loop.len() < 2);
        let mut all_new_ts: Vec<(i32, Vec<BMCTransition>)> = Vec::new();
        for (src_id, t1) in t_in {
            let lhs = if let Some(tl) = t_loop.get(0) {
                let st = Regex::Star(Box::new(tl.label.clone()));
                Regex::Concat(Box::new(t1.label.clone()), Box::new(st))
            } else {
                t1.label.clone()
            };
            let mut new_ts: Vec<BMCTransition> = Vec::new();
            for t2 in &t_out {
                let re = Regex::Concat(Box::new(lhs.clone()), Box::new(t2.label.clone()));
                new_ts.push(BMCTransition{
                    dest_id: t2.dest_id,
                    label: re
                });
            }
            all_new_ts.push((src_id, new_ts));
        }
        for (src_id, ts) in &mut all_new_ts {
            let s = &mut self.lookup_state_mut(*src_id).transitions;
            s.retain(|s| s.dest_id != s_id);
            s.append(ts);
        }
        self.states.retain(|s| s.id != s_id);
        self.label.gen_id += 1;
    }
}
