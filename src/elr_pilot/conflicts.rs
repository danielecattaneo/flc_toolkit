use crate::elr_pilot::*;

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

impl PilotState {
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

impl Pilot {
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
