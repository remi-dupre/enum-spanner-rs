pub mod atom;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::mapping::Marker;

#[derive(Debug)]
pub enum Label {
    Atom(atom::Atom),
    Assignation(Marker),
}

#[derive(Debug)]
pub struct Automaton {
    pub nb_states: usize,
    pub transitions: Vec<(usize, Rc<Label>, usize)>,
    pub finals: HashSet<usize>,

    // Redundant caching structures
    adj: Vec<Vec<(Rc<Label>, usize)>>,
    adj_for_char: HashMap<char, Vec<Vec<usize>>>,
    assignations: Vec<Vec<(Rc<Label>, usize)>>,
    closure_for_assignations: Vec<Vec<usize>>,
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
            adj_for_char: HashMap::new(),
            assignations: Vec::new(),
            closure_for_assignations: Vec::new(),
        };

        automaton.adj = automaton.init_adj();
        automaton.assignations = automaton.init_assignations();
        automaton.closure_for_assignations = automaton.init_closure_for_assignations();

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

    /// Get the adjacency list representing transitions of the automaton that can be used when
    /// reading a given char.
    pub fn get_adj_for_char(&mut self, x: char) -> &Vec<Vec<usize>> {
        let nb_states = self.nb_states();
        let adj_for_char = &mut self.adj_for_char;
        let transitions = &self.transitions;

        adj_for_char.entry(x).or_insert_with(|| {
            let mut res = vec![Vec::new(); nb_states];

            for (source, _, target) in transitions {
                res[*source].push(*target);
            }

            res
        })
    }

    /// Get adjacency lists labeled with the corresponding marker for transitions labeled with an
    /// assignation.
    pub fn get_assignations(&self) -> &Vec<Vec<(Rc<Label>, usize)>> {
        &self.assignations
    }

    /// Get the closure as adjacency lists for transitions labeled with an assignation.
    pub fn get_closure_for_assignations(&self) -> &Vec<Vec<usize>> {
        &self.closure_for_assignations
    }

    fn init_adj(&self) -> Vec<Vec<(Rc<Label>, usize)>> {
        let mut ret = vec![Vec::new(); self.nb_states];

        for (source, label, target) in &self.transitions {
            ret[*source].push((label.clone(), *target));
        }

        ret
    }

    fn init_assignations(&self) -> Vec<Vec<(Rc<Label>, usize)>> {
        // Compute adjacency list
        let mut adj = vec![Vec::new(); self.nb_states()];

        for (source, label, target) in &self.transitions {
            if let Label::Assignation(_) = **label {
                adj[*source].push((label.clone(), *target))
            }
        }

        adj
    }

    fn init_closure_for_assignations(&self) -> Vec<Vec<usize>> {
        // Compute adjacency list
        let assignations = self.get_assignations();
        let adj: Vec<Vec<usize>> = (0..self.nb_states())
            .map(|i| assignations[i].iter().map(|(_, j)| *j).collect())
            .collect();

        // Compute closure
        let mut closure = vec![Vec::new(); self.nb_states()];

        for state in 0..self.nb_states() {
            let mut heap = vec![state];
            let mut seen = HashSet::new();
            seen.insert(state);

            while let Some(source) = heap.pop() {
                for target in &adj[source] {
                    closure[state].push(*target);

                    if !seen.contains(target) {
                        heap.push(*target);
                        seen.insert(*target);
                    }
                }
            }
        }

        closure
    }
}
