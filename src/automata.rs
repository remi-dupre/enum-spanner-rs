use std::rc::Rc;

use regex_syntax::hir;
use regex_syntax::Parser;

use super::glushkov;
use super::mapping;

#[derive(Debug)]
pub enum Label {
    Atom(Atom),
    Assignation(mapping::Marker),
}

#[derive(Debug)]
pub enum Atom {
    Literal(hir::Literal),
    Class(hir::Class),
}

#[derive(Debug)]
pub struct Automata {
    nb_states: usize,
    transitions: Vec<(usize, Rc<Label>, usize)>,
    finals: Vec<usize>,
}

impl Automata {
    pub fn from_hir(hir: hir::Hir) -> Automata {
        let locallang = glushkov::LocalLang::from_hir(hir);

        let iner_transitions = locallang
            .factors
            .f
            .into_iter()
            .map(|(source, target)| (source.id + 1, target.label, target.id + 1));
        let pref_transitions = locallang
            .factors
            .p
            .into_iter()
            .map(|target| (0, target.label, target.id + 1));

        let transitions = iner_transitions.chain(pref_transitions).collect();
        let mut finals: Vec<usize> = locallang.factors.d.into_iter().map(|x| x.id + 1).collect();

        if locallang.factors.g {
            finals.push(0);
        }

        Automata {
            nb_states: locallang.nb_terms + 1,
            transitions,
            finals,
        }
    }

    pub fn from_regex(regex: &str) -> Automata {
        let hir = Parser::new().parse(regex).unwrap();
        Automata::from_hir(hir)
    }

    pub fn nb_states(&self) -> usize {
        self.nb_states
    }

    pub fn nb_transitions(&self) -> usize {
        self.transitions.len()
    }
}
