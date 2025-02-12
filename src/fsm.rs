mod dot_formatter;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct BaseTransition<L> {
    pub label: L,
    pub dest_id: i32
}

impl BaseTransition<char> {
    pub fn is_nonterminal(&self) -> bool {
        self.label.is_ascii_uppercase()
    }
}

pub type Transition = BaseTransition<char>;

#[derive(Debug)]
pub struct BaseState<SL, TL> {
    pub id: i32,
    pub label: SL,
    pub transitions: Vec<BaseTransition<TL>>,
    pub is_initial: bool,
    pub is_final: bool,
}

pub type State = BaseState<(), char>;

#[derive(Debug)]
pub struct BaseMachine<SL, TL> {
    pub name: char,
    pub states: Vec<BaseState<SL, TL>>
}

pub type Machine = BaseMachine<(), char>;

impl Machine {
    pub fn try_lookup_state(&self, id: i32) -> Option<&State> {
        self.states.iter().find(|s| s.id == id)
    }

    pub fn lookup_state(&self, id: i32) -> &State {
        self.try_lookup_state(id).expect("state does not exist")
    }
}
