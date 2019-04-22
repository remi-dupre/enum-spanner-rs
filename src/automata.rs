use std::collections::HashMap;
use std::rc::Rc;

use regex_syntax::hir;
use regex_syntax::Parser;

use super::glushkov;
use super::mapping;

#[derive(Debug)]
pub struct Label {
    pub id: usize,
    pub kind: LabelKind,
}

#[derive(Debug)]
pub enum LabelKind {
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

        let iner_transitions = locallang.factors.f.into_iter().map(|(source, target)| {
            let src_id = source.id;
            let tgt_id = target.id;
            (src_id + 1, target, tgt_id + 1)
        });
        let pref_transitions = locallang.factors.p.into_iter().map(|target| {
            let tgt_id = target.id;
            (0, target, tgt_id + 1)
        });

        let transitions = iner_transitions.chain(pref_transitions).collect();
        let mut finals: Vec<usize> = locallang.factors.d.into_iter().map(|x| x.id + 1).collect();

        if locallang.factors.g {
            finals.push(0);
        }

        Automata {
            nb_states: locallang.nb_labels + 1,
            transitions,
            finals,
        }
    }

    pub fn from_regex(regex: &str) -> Automata {
        let hir = Parser::new().parse(regex).unwrap();
        Automata::from_hir(hir)
    }
}
