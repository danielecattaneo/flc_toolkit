use crate::fsm::*;

impl State {
    fn to_dot(&self, m_name: char, with_name: bool) -> String {
        let name = format!("{}{}", m_name, self.id);
        let label = if with_name { 
            format!("<{}<sub>{}</sub>>", self.id, m_name)
        } else {
            self.id.to_string()
        };

        let mut res: Vec<String> = Vec::new();
        res.push(format!("  {} [label={}];", name, label));
        if self.is_initial {
            res.push(format!("  init{} -> {};", m_name, name));
        }
        let transitions: Vec<_> = self.transitions.iter().map(|t| {
            let dest_name = format!("{}{}", m_name, t.dest_id);
            format!("  {} -> {} [label=\"{}\"];", name, dest_name, t.character)
        }).collect();
        if self.is_final {
            res.push(format!("  sink{} [shape=plain,label=\" \"];", name));
            res.push(format!("  {} -> sink{};", name, name));
        }
        res.extend(transitions);
        res.join("\n")
    }
}

impl Machine {
    pub fn to_dot_2(&self, with_name: bool, with_header: bool) -> String {
        let header = if with_header {
            "digraph {\n  rankdir=\"LR\";\n"
        } else {
            ""
        };
        let init = format!("  init{} [shape=plain,label=\"{}\"];\n", self.name, self.name);
        let states = self.states.iter().map(|s| {
            s.to_dot(self.name, with_name)
        }).collect::<Vec<_>>().join("\n");
        let trailer = if with_header { "\n}" } else { "" };
        format!("{}{}{}{}", header, init, states, trailer)
    }

    pub fn to_dot(&self) -> String {
        self.to_dot_2(false, true)
    }
}
