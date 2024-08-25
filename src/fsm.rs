#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Transition {
    pub character: char,
    pub dest_id: i32
}

impl Transition {
    pub fn is_nonterminal(&self) -> bool {
        self.character.is_ascii_uppercase()
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
    pub fn lookup_state(&self, id: i32) -> &State {
        for s in &self.states {
            if s.id == id {
                return &s;
            }
        }
        panic!("state {id} does not exist")
    }
}
