use std::collections::{HashMap, HashSet, VecDeque};
use std::iter;

use super::super::automaton::Automaton;
use super::super::mapping::{Mapping, Marker};
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

impl<'a> IndexedDag {
    pub fn compile(mut automaton: Automaton, text: String) -> IndexedDag {
        let mut jump = Jump::new(
            iter::once(automaton.get_initial()),
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

    fn follow_SpSm(
        &self,
        gamma: &Vec<usize>,
        s_p: &HashSet<&Marker>,
        s_m: &HashSet<&Marker>,
    ) -> Vec<usize> {
        let adj = self.automaton.get_rev_assignations();
        let mut path_set: HashMap<usize, Option<HashSet<_>>> = HashMap::new();

        for &state in gamma {
            path_set.insert(state, Some(HashSet::new()));
        }

        // TODO: Move this to a `tools` module.
        let are_incomparable = |set1: &HashSet<_>, set2: &HashSet<_>| {
            set1.iter().any(|x| !set2.contains(x)) && set2.iter().any(|x| !set1.contains(x))
        };

        // NOTE: Consider writing this as a recursive function?
        let mut queue: VecDeque<_> = gamma.iter().map(|x| *x).collect();

        while let Some(source) = queue.pop_front() {
            for (label, target) in &adj[source] {
                let label = label.get_marker().unwrap();

                if s_m.contains(label) {
                    continue;
                }

                if !path_set.contains_key(target) {
                    queue.push_back(*target);
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

        path_set
            .iter()
            .filter_map(|(vertex, vertex_ps)| match vertex_ps {
                Some(vertex_ps) if vertex_ps.len() == s_p.len() => Some(*vertex),
                _ => None,
            })
            .collect()
    }

    /// TODO: implement this as an iterable.
    fn next_level(&self, gamma: &Vec<usize>) -> Vec<(HashSet<&Marker>, Vec<usize>)> {
        let mut res = Vec::new();
        let adj = self.automaton.get_rev_assignations();

        // Get list of variables that are part of the level
        // TODO: It might still be slower to just using the list of all variables in the automaton?
        let mut k = HashSet::new();
        let mut stack = gamma.clone();
        let mut marks = HashSet::new();

        for x in gamma {
            marks.insert(x);
        }

        while let Some(source) = stack.pop() {
            for (label, target) in &adj[source] {
                k.insert(label.get_marker().unwrap());

                if !marks.contains(target) {
                    marks.insert(target);
                    stack.push(*target);
                }
            }
        }

        // Run over the decision tree of variables to select
        let k: Vec<_> = k.iter().collect();
        let mut stack = vec![(HashSet::new(), HashSet::new())];

        while let Some((mut s_p, mut s_m)) = stack.pop() {
            let mut gamma2 = Some(self.follow_SpSm(gamma, &s_p, &s_m));

            if gamma2.as_ref().unwrap().is_empty() {
                continue;
            }

            while s_p.len() + s_m.len() < k.len() {
                let depth = s_p.len() + s_m.len();
                s_p.insert(*k[depth]);
                gamma2 = Some(self.follow_SpSm(gamma, &s_p, &s_m));

                if !gamma2.as_ref().unwrap().is_empty() {
                    let mut new_s_p = s_p.clone();
                    let mut new_s_m = s_m.clone();
                    new_s_m.insert(*k[depth]);
                    new_s_p.remove(*k[depth]);
                    stack.push((new_s_p, new_s_m));
                } else {
                    s_p.remove(*k[depth]);
                    s_m.insert(*k[depth]);
                    gamma2 = None;
                }
            }

            let gamma2 = match gamma2 {
                None => self.follow_SpSm(gamma, &s_p, &s_m),
                Some(val) => val,
            };

            res.push((s_p, gamma2));
        }

        res
    }

    /// TODO: implement this as an iterable.
    pub fn iter(&'a self) -> impl Iterator<Item = Mapping<'a>> {
        // Only start with accessible final states
        let start = self
            .jump
            .finals()
            .intersection(&self.automaton.finals.iter().map(|x| *x).collect())
            .map(|x| *x)
            .collect();
        let mut stack = vec![(self.text.chars().count(), start, HashSet::new())];
        let mut ret = Vec::new();

        while let Some((level, gamma, mapping)) = stack.pop() {
            for (s_p, new_gamma) in self.next_level(&gamma).into_iter() {
                if new_gamma.is_empty() {
                    continue;
                }

                let mut new_mapping = mapping.clone();
                for marker in s_p {
                    new_mapping.insert((marker, level));
                }

                if level == 0 && new_gamma.contains(&self.automaton.get_initial()) {
                    ret.push(new_mapping);
                } else if let Some((jump_level, jump_gamma)) =
                    self.jump.jump(level, new_gamma.into_iter())
                {
                    if !jump_gamma.is_empty() {
                        stack.push((jump_level, jump_gamma, new_mapping));
                    }
                }
            }
        }

        ret.into_iter().map(move |marker_assigns| {
            Mapping::from_markers(&self.text, marker_assigns.into_iter())
        })
    }
}
