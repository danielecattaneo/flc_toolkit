mod dot_formatter;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Transition {
    pub label: char,
    pub dest_id: i32
}

impl Transition {
    pub fn is_nonterminal(&self) -> bool {
        self.label.is_ascii_uppercase()
    }
}

#[derive(Debug)]
pub struct State {
    pub id: i32,
    pub transitions: Vec<Transition>,
    pub is_initial: bool,
    pub is_final: bool
}

#[derive(Debug)]
pub struct Machine {
    pub name: char,
    pub states: Vec<State>
}

impl Machine {
    pub fn try_lookup_state(&self, id: i32) -> Option<&State> {
        self.states.iter().find(|s| s.id == id)
    }

    pub fn lookup_state(&self, id: i32) -> &State {
        self.try_lookup_state(id).expect("state does not exist")
    }
}
