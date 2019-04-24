use super::super::automaton::Automaton;

use super::jump::Jump;

/// DAG built from the product automaton of a variable automaton and a text.
///
/// The structure allows to enumerate efficiently all the distinct matches of the input automata
/// over the input text.
pub struct IndexedDag {
    automaton: Automaton,
    text: String,
    jump: Jump,
}

impl IndexedDag {
    pub fn compile(mut automaton: Automaton, text: String) -> IndexedDag {
        let mut jump = Jump::new(
            automaton.get_initials(),
            automaton.get_closure_for_assignations(),
        );

        // NOTE: this copy could be avoided by changing the cached getter's behaviour
        let closure_for_assignations = automaton.get_closure_for_assignations().clone();

        for (curr_level, curr_char) in text.chars().enumerate() {
            let adj_for_char = automaton.get_adj_for_char(curr_char);

            jump.init_next_level(adj_for_char, &closure_for_assignations);
        }

        IndexedDag {
            automaton,
            text,
            jump,
        }
    }
}
