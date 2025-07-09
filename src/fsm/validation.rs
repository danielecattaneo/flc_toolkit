use core::fmt;

use crate::fsm::*;
use crate::validation::*;

impl<ML: fmt::Display, SL, TL: fmt::Display> BaseMachine<ML, SL, TL> {
    pub fn validate_state_count(&self) -> bool {
        // All machines must have > 0 states
        if self.states.is_empty() {
            eprintln!("error: machine {} has zero states", self.label);
            false
        } else {
            true
        }
    }

    pub fn validate_any_initial_state(&self) -> bool {
        if !self.states.iter().any(|s| s.is_initial) {
            eprintln!("error: no initial state in machine {}", self.label);
            false
        } else {
            true
        }
    }

    pub fn validate_any_final_state(&self) -> bool {
        if !self.states.iter().any(|s| s.is_final) {
            eprintln!("error: no final state in machine {}", self.label);
            false
        } else {
            true
        }
    }

    pub fn validate_transitions(&self) -> bool {
        let mut res = true;
        for s in &self.states {
            for t in s.transitions.iter() {
                if self.try_lookup_state(t.dest_id).is_none() {
                    eprintln!("error: transition {}{} -{}-> {}{} goes to a non-existent state", s.id, self.label, t.label, t.dest_id, self.label);
                    res = false;
                }
            }
        }
        res
    }
}

impl<ML: fmt::Display, SL, TL: fmt::Display> Validation for BaseMachine<ML, SL, TL> {
    fn validate(&self) -> bool {
        [
            self.validate_state_count(),
            self.validate_any_initial_state(),
            self.validate_any_final_state(),
            self.validate_transitions()
        ].into_iter().all(|v| v)
    }
}
