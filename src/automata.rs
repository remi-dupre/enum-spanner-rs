use regex_syntax::hir;
use regex_syntax::Parser;

use super::glushkov;
use super::mapping;

#[derive(Clone, Debug)]
pub enum Atom {
    Literal(hir::Literal),
    Class(hir::Class),
    Marker(mapping::Marker),
}

#[derive(Debug)]
pub struct Automata {
    nb_states: usize,
    transitions: Vec<(usize, Atom, usize)>,
    finals: Vec<usize>,
}

impl Automata {
    pub fn from_hir(hir: hir::Hir) -> Automata {
        let locallang = glushkov::LocalLang::from_hir(hir);

        let iner_transitions = locallang
            .factors
            .f
            .iter()
            .map(|(source, target)| (source + 1, locallang.atoms[*target].clone(), target + 1));
        let pref_transitions = locallang
            .factors
            .p
            .iter()
            .map(|target| (0, locallang.atoms[*target].clone(), target + 1));

        let transitions = iner_transitions.chain(pref_transitions).collect();
        let finals = locallang.factors.d.iter().map(|x| x + 1).collect();

        Automata {
            nb_states: locallang.atoms.len() + 1,
            transitions,
            finals,
        }
    }

    pub fn from_regex(regex: &str) -> Automata {
        let hir = Parser::new().parse(regex).unwrap();
        Automata::from_hir(hir)
    }
}
