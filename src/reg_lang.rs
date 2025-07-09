use std::fmt;
use std::collections::HashSet;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NumTerm {
    pub c: char,
    pub i: usize
}

impl NumTerm {
    pub fn new(c: char, i: usize) -> NumTerm {
        NumTerm{c, i}
    }
}

impl fmt::Display for NumTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.c, self.i)
    }
}

pub type NumTermSet = HashSet<NumTerm>;
pub type NumDigramsSet = HashSet<(NumTerm, NumTerm)>;

pub fn set_prod(a: &NumTermSet, b: &NumTermSet) -> NumDigramsSet {
    let mut res = NumDigramsSet::new();
    for &ia in a {
        for &ib in b {
            res.insert((ia, ib));
        }
    }
    res
}

pub type NumFollowersMap = HashMap<NumTerm, NumTermSet>;

pub trait NumLocalSets {
    fn nullable(&self) -> bool;
    fn all_numbered(&self) -> NumTermSet;
    fn numbered_initials(&self) -> NumTermSet;
    fn numbered_finals(&self) -> NumTermSet;
    fn numbered_digrams(&self) -> NumDigramsSet;

    fn numbered_followers(&self) -> NumFollowersMap {
        let mut res = NumFollowersMap::new();
        for t in self.all_numbered() {
            res.insert(t, NumTermSet::new());
        }
        for (t, f) in self.numbered_digrams() {
            res.get_mut(&t).unwrap().insert(f);
        }
        res
    }

    fn dump_local_sets(&self) {
        let mut ini: Vec<_> = self.numbered_initials().into_iter().map(|t| format!("{t}")).collect();
        ini.sort();
        if self.nullable() {
            ini.push("⊣".to_string());
        }
        eprintln!("Ini = {{{}}}", ini.join(", "));

        let fin = self.numbered_finals();

        let mut fin_tmp: Vec<_> = self.numbered_followers().iter().map(|(t, fol)| {
            let mut str_fol: Vec<_> = fol.iter().map(|t| format!("{t}")).collect();
            str_fol.sort();
            if fin.contains(t) {
                str_fol.push("⊣".to_string());
            }
            (t.i, format!("Fol({}) = {{{}}}", t, str_fol.join(", ")))
        }).collect();
        fin_tmp.sort();
        let fin: Vec<_> = fin_tmp.iter().map(|(_, s)| s.clone()).collect();
        eprintln!("{}", fin.join("\n"));
    }
}
