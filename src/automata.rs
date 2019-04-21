use super::glushkov;
use super::mapping;

use regex_syntax::hir;

#[derive(Clone, Copy, Debug)]
pub enum Atom<'a> {
    Literal(&'a hir::Literal),
    Class(&'a hir::Class),
    Marker(mapping::Marker<'a>),
}

#[derive(Debug)]
pub struct Automata<'a> {
    nb_states: usize,
    transitions: Vec<(usize, Atom<'a>, usize)>,
    finals: Vec<usize>,
}

impl<'a> Automata<'a> {
    pub fn from_hir(hir: &'a hir::Hir) -> Automata {
        let locallang = glushkov::LocalLang::from_hir(hir);

        let iner_transitions = locallang
            .f
            .iter()
            .map(|(source, target)| (source + 1, locallang.atoms[*target].clone(), target + 1));

        let pref_transitions = locallang
            .p
            .iter()
            .map(|target| (0, locallang.atoms[*target].clone(), target + 1));

        let transitions = iner_transitions.chain(pref_transitions).collect();
        let finals = locallang.d.iter().map(|x| x + 1).collect();

        Automata {
            nb_states: locallang.atoms.len() + 1,
            transitions,
            finals,
        }
    }
}
