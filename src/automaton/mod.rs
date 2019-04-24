pub mod atom;

use std::collections::HashSet;
use std::rc::Rc;

use super::mapping;

#[derive(Debug)]
pub enum Label {
    Atom(atom::Atom),
    Assignation(mapping::Marker),
}

#[derive(Debug)]
pub struct Automaton {
    pub nb_states: usize,
    pub transitions: Vec<(usize, Rc<Label>, usize)>,
    pub finals: HashSet<usize>,

    /// Redundant caching structures
    adj: Vec<Vec<(Rc<Label>, usize)>>,
}

impl Automaton {
    pub fn new<T, U>(nb_states: usize, transitions: T, finals: U) -> Automaton
    where
        T: Iterator<Item = (usize, Rc<Label>, usize)>,
        U: Iterator<Item = usize>,
    {
        let mut automaton = Automaton {
            nb_states,
            transitions: transitions.collect(),
            finals: finals.collect(),

            adj: Vec::new(),
        };
        automaton.adj = automaton.init_adj();
        automaton
    }

    pub fn nb_states(&self) -> usize {
        self.nb_states
    }

    pub fn nb_transitions(&self) -> usize {
        self.transitions.len()
    }

    pub fn get_adj(&self) -> &Vec<Vec<(Rc<Label>, usize)>> {
        &self.adj
    }

    fn init_adj(&self) -> Vec<Vec<(Rc<Label>, usize)>> {
        let mut ret = vec![Vec::new(); self.nb_states];

        for (source, label, target) in &self.transitions {
            ret[*source].push((label.clone(), *target));
        }

        ret
    }
}
