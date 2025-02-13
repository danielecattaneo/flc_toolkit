use crate::fsm::*;

impl DotFormat for char {
    fn to_dot(&self, _: bool) -> String {
        format!("{}", self)
    }
}

impl DotFormat for StateLabel {
    fn to_dot(&self, detailed: bool) -> String {
        if detailed { 
            format!("<{}<sub>{}</sub>>", self.id, self.m_name)
        } else {
            self.id.to_string()
        }
    }
}

impl<SL: DotFormat, TL: DotFormat> BaseState<SL, TL> {
    fn to_dot(&self, m_name: char, detailed: bool) -> String {
        let name = format!("{}{}", m_name, self.id);
        let label = self.label.to_dot(detailed);

        let mut res: Vec<String> = Vec::new();
        res.push(format!("  {} [label=<{}>];", name, label));
        if self.is_initial {
            res.push(format!("  init{} -> {};", m_name, name));
        }
        let transitions: Vec<_> = self.transitions.iter().map(|t| {
            let dest_name = format!("{}{}", m_name, t.dest_id);
            format!("  {} -> {} [label=\"{}\"];", name, dest_name, t.label.to_dot(detailed))
        }).collect();
        if self.is_final {
            res.push(format!("  sink{} [shape=plain,label=\" \"];", name));
            res.push(format!("  {} -> sink{};", name, name));
        }
        res.extend(transitions);
        res.join("\n")
    }
}

impl<SL: DotFormat, TL: DotFormat> BaseMachine<SL, TL> {
    pub fn to_dot_2(&self, detailed: bool, with_header: bool) -> String {
        let header = if with_header {
            "digraph {\n  rankdir=\"LR\";\n"
        } else {
            ""
        };
        let init = format!("  init{} [shape=plain,label=\"{}\"];\n", self.name, self.name);
        let states = self.states.iter().map(|s| {
            s.to_dot(self.name, detailed)
        }).collect::<Vec<_>>().join("\n");
        let trailer = if with_header { "\n}" } else { "" };
        format!("{}{}{}{}", header, init, states, trailer)
    }
}

impl<SL: DotFormat, TL: DotFormat> DotFormat for BaseMachine<SL, TL> {
    fn to_dot(&self, detailed: bool) -> String {
        self.to_dot_2(detailed, true)
    }
}
