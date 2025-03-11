use crate::fsm::*;

impl Machine {
    fn add_epsilon_transition(&mut self, src_id: i32, dest_id: i32) -> bool {
        let src = self.lookup_state_mut(src_id);
        if src.transitions.iter().any(|t| t.dest_id == dest_id && t.is_epsilon()) {
            false
        } else {
            src.transitions.push(Transition{ dest_id, label: '_' });
            true
        }
    }

    pub fn epsilon_trans_closure(&mut self) {
        loop {
            let mut to_add: Vec<(i32, i32)> = vec![];
            for s in &self.states {
                for t in &s.transitions {
                    if t.is_epsilon() {
                        let dest_s = self.lookup_state(t.dest_id);
                        for next_t in &dest_s.transitions {
                            if next_t.is_epsilon() {
                                to_add.push((s.id, next_t.dest_id));
                            }
                        }
                    }
                }
            }
            let mut changed = false;
            for (src_id, dest_id) in to_add {
                changed |= self.add_epsilon_transition(src_id, dest_id);
            }
            if !changed {
                break;
            }
        }
    }

    fn add_transition(&mut self, src_id: i32, c: char, dest_id: i32) {
        let src = self.lookup_state_mut(src_id);
        if !src.transitions.iter().any(|t| t.dest_id == dest_id && t.label == c) {
            src.transitions.push(Transition{ dest_id, label: c });
        }
    }

    pub fn mark_new_final(&mut self) {
        let mut to_add: Vec<i32> = vec![];
        for s in &self.states {
            for t in &s.transitions {
                if t.is_epsilon() && self.lookup_state(t.dest_id).is_final {
                    to_add.push(s.id);
                }
            }
        }
        for id in to_add {
            self.lookup_state_mut(id).is_final = true;
        }
    }

    pub fn backward_propagation(&mut self) {
        let mut to_add: Vec<(i32, char, i32)> = vec![];
        for s in &self.states {
            for t in &s.transitions {
                if t.is_epsilon() {
                    let dest_s = self.lookup_state(t.dest_id);
                    for next_t in &dest_s.transitions {
                        if !next_t.is_epsilon() {
                            to_add.push((s.id, next_t.label, next_t.dest_id));
                        }
                    }
                }
            }
        }
        for (src_id, c, dest_id) in to_add {
            self.add_transition(src_id, c, dest_id);
        }
        self.mark_new_final();
    }

    pub fn mark_new_initial(&mut self) {
        let mut to_add: Vec<i32> = vec![];
        for s in &self.states {
            if s.is_initial {
                for t in &s.transitions {
                    if t.is_epsilon() {
                        to_add.push(t.dest_id);
                    }
                }
            }
        }
        for id in to_add {
            self.lookup_state_mut(id).is_initial = true;
        }
    }

    pub fn forward_propagation(&mut self) {
        let mut to_add: Vec<(i32, char, i32)> = vec![];
        for s in &self.states {
            for t in &s.transitions {
                if !t.is_epsilon() {
                    let dest_s = self.lookup_state(t.dest_id);
                    for next_t in &dest_s.transitions {
                        if next_t.is_epsilon() {
                            to_add.push((s.id, t.label, next_t.dest_id));
                        }
                    }
                }
            }
        }
        for (src_id, c, dest_id) in to_add {
            self.add_transition(src_id, c, dest_id);
        }
        self.mark_new_initial();
    }

    pub fn remove_epsilon_trans(&mut self) {
        for s in &mut self.states {
            let new_ts: Vec<_> = s.transitions.iter().filter(|t| {
                !t.is_epsilon()
            }).cloned().collect();
            s.transitions = new_ts;
        }
    }
}
