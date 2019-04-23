use std::str::CharIndices;

use super::super::automaton::{Automaton, Label};
use super::{Mapping, Marker};

/// Enumerate all the matches of a variable automata over a text.
///
/// ** For this naive implementation, there is no garantee that produced matches are distincts. **
pub struct NaiveEnum<'a> {
    automaton: &'a Automaton,
    text: &'a str,

    /// Holds current positions of the runs as a stack of:
    ///  - current state on the automata
    ///  - current index on the word
    ///  - assignations that have been done so far
    curr_state: Vec<(usize, CharIndices<'a>, Vec<(&'a Marker, usize)>)>,
}

impl<'a> NaiveEnum<'a> {
    pub fn new(automaton: &'a Automaton, text: &'a str) -> NaiveEnum<'a> {
        NaiveEnum {
            automaton,
            text,
            curr_state: vec![(0, text.char_indices(), Vec::new())],
        }
    }
}

impl<'a> Iterator for NaiveEnum<'a> {
    type Item = Mapping<'a>;

    fn next(&mut self) -> Option<Mapping<'a>> {
        while let Some((state, index, assigns)) = self.curr_state.pop() {
            let curr_char = index.clone().next();

            for (label, target) in &self.automaton.get_adj()[state] {
                match **label {
                    Label::Atom(ref atom) if curr_char != None => {
                        if let Some((_, curr_char)) = curr_char {
                            if !atom.is_match(&curr_char) {
                                continue;
                            }
                        }
                        let mut new_index = index.clone();
                        new_index.next();

                        self.curr_state.push((*target, new_index, assigns.clone()));
                    }
                    Label::Assignation(ref marker) => {
                        let mut new_assigns = assigns.clone();
                        let pos = match curr_char {
                            None => self.text.len(),
                            Some((pos, _)) => pos,
                        };
                        new_assigns.push((marker, pos));
                        self.curr_state.push((*target, index.clone(), new_assigns));
                    }
                    _ => (),
                }
            }

            if curr_char == None && self.automaton.finals.contains(&state) {
                return Some(Mapping::from_markers(self.text, assigns.into_iter()));
            }
        }

        None
    }
}
