use crate::mnet::*;

impl MachineNet {
    pub fn to_dot(&self) -> String {
        let header = "digraph {\n  rankdir=\"LR\";\n  node [shape=\"circle\"];\n";
        let machines = self.machines.iter().map(|m| {
            m.to_dot_2(true, false)
        }).collect::<Vec<_>>().join("\n");
        let trailer = "\n}";
        format!("{}{}{}", header, machines, trailer)
    }
}
