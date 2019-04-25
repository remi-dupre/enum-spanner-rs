use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::rc::Rc;

use super::super::automaton::Automaton;

use super::super::mapping::Marker;
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

        println!("{:?}", jump);
        IndexedDag {
            automaton,
            text,
            jump,
        }
    }

    fn follow_SpSm(&self, gamma: &Vec<usize>, s_p: &HashSet<&Marker>, s_m: &HashSet<&Marker>) {
        let adj = self.automaton.get_rev_assignations();

        // NOTE: Consider using a Vec instead?
        let mut path_set: HashMap<usize, Option<HashSet<_>>> = HashMap::new();

        for &state in gamma {
            path_set.insert(state, Some(HashSet::new()));
        }

        // TODO: Move this to a `tools` module.
        let are_incomparable = |set1: &HashSet<_>, set2: &HashSet<_>| {
            set1.iter().any(|x| !set2.contains(x)) && set2.iter().any(|x| !set1.contains(x))
        };

        // NOTE: Consider writing this as a recursive function?
        let mut queue = VecDeque::new();

        for x in gamma {
            queue.push_back(*x);
        }

        while let Some(source) = queue.pop_front() {
            for (label, target) in &adj[source] {
                let label = label.get_marker().unwrap();

                if s_m.contains(label) {
                    continue;
                }

                let mut new_ps = path_set[&source].clone().unwrap();

                if s_p.contains(label) {
                    new_ps.insert(label);
                }

                path_set
                    .entry(*target)
                    .and_modify(|entry| {
                        if let Some(old_ps) = entry {
                            if are_incomparable(&new_ps, old_ps) {
                                *entry = None;
                            } else {
                                *entry = Some(new_ps.clone());
                            }
                        }
                    })
                    .or_insert(Some(new_ps));
            }
        }
    }
}
