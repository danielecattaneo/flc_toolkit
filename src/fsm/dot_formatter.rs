use std::fmt::Display;

use crate::fsm::*;

impl DotFormat for char {
    fn to_dot(&self, _: bool) -> String {
        if *self == '_' {
            format!("ε")
        } else {
            format!("\"{}\"", self)
        }
    }
}

impl DotFormat for NumTerm {
    fn to_dot(&self, _: bool) -> String {
        if self.c == '_' {
            format!("ε")
        } else {
            format!("<{}<sub>{}</sub>>", self.c, self.i)
        }
    }
}

impl DotFormat for StateLabel {
    fn to_dot(&self, detailed: bool) -> String {
        let str_id = if self.id == -1 {
            "⊢".to_string()
        } else if self.id == -2 {
            "⊣".to_string()
        } else {
            self.id.to_string()
        };
        if detailed { 
            format!("<{}<sub>{}</sub>>", str_id, self.m_name)
        } else {
            format!("\"{}\"", str_id)
        }
    }
}

fn state_id_to_node_id<ML: DotFormat+Display>(m_name: &ML, s_id: i32) -> String {
    if s_id == -1 {
        format!("n{}B", m_name)
    } else if s_id == -2 {
        format!("n{}E", m_name)
    } else {
        format!("n{}{}", m_name, s_id)
    }
}

impl<SL: DotFormat, TL: DotFormat> BaseState<SL, TL> {
    fn to_dot<ML: DotFormat+Display>(&self, m_name: &ML, detailed: bool) -> String {
        let name = state_id_to_node_id(m_name, self.id);
        let label = self.label.to_dot(detailed);

        let mut res: Vec<String> = Vec::new();
        res.push(format!("  {} [label={}];", name, label));
        if self.is_initial {
            res.push(format!("  init{} -> {};", m_name, name));
        }
        let transitions: Vec<_> = self.transitions.iter().map(|t| {
            let dest_name = state_id_to_node_id(m_name, t.dest_id);
            format!("  {} -> {} [label={}];", name, dest_name, t.label.to_dot(detailed))
        }).collect();
        if self.is_final {
            res.push(format!("  sink{} [shape=plain,label=\" \"];", name));
            res.push(format!("  {} -> sink{};", name, name));
        }
        res.extend(transitions);
        res.join("\n")
    }
}

impl<ML: DotFormat+Display, SL: DotFormat, TL: DotFormat> BaseMachine<ML, SL, TL> {
    pub fn to_dot_2(&self, detailed: bool, with_header: bool) -> String {
        let header = if with_header {
            "digraph {\n  rankdir=\"LR\";\n"
        } else {
            ""
        };
        let init = format!("  init{} [shape=plain,label={}];\n", self.label, self.label.to_dot(detailed));
        let states = self.states.iter().map(|s| {
            s.to_dot(&self.label, detailed)
        }).collect::<Vec<_>>().join("\n");
        let trailer = if with_header { "\n}" } else { "" };
        format!("{}{}{}{}", header, init, states, trailer)
    }
}

impl<ML: DotFormat+Display, SL: DotFormat, TL: DotFormat> DotFormat for BaseMachine<ML, SL, TL> {
    fn to_dot(&self, detailed: bool) -> String {
        self.to_dot_2(detailed, true)
    }
}
