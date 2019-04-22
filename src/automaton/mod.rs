pub mod atom;

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
    pub finals: Vec<usize>,
}

impl Automaton {
    pub fn nb_states(&self) -> usize {
        self.nb_states
    }

    pub fn nb_transitions(&self) -> usize {
        self.transitions.len()
    }
}
