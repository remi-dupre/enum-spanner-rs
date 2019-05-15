// use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryInto;
use std::iter;
use std::time::{Duration, Instant};

use super::super::automaton::Automaton;
use super::super::mapping::{Mapping, Marker};
use super::super::progress::Progress;
use super::super::settings;
use super::jump::Jump;

//  ___           _                   _ ____
// |_ _|_ __   __| | _____  _____  __| |  _ \  __ _  __ _
//  | || '_ \ / _` |/ _ \ \/ / _ \/ _` | | | |/ _` |/ _` |
//  | || | | | (_| |  __/>  <  __/ (_| | |_| | (_| | (_| |
// |___|_| |_|\__,_|\___/_/\_\___|\__,_|____/ \__,_|\__, |
//                                                  |___/

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

        let chars: Vec<_> = text.chars().collect();
        let progress = Progress::from_iter(chars.into_iter());

        for curr_char in progress {
            // Add a layer
            let adj_for_char = automaton.get_adj_for_char(curr_char);
            jump.init_next_level(adj_for_char, &closure_for_assignations);
        }

        IndexedDag {
            automaton,
            text,
            jump,
        }
    }

    /// TODO: implement this as a real iterable.
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
            for (s_p, new_gamma) in self.next_level(&gamma) {
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

    pub fn get_nb_levels(&self) -> usize {
        self.jump.get_nb_levels()
    }

    /// TODO: implement this as an iterable.
    fn next_level(
        &'a self,
        gamma: &Vec<usize>,
    ) -> impl Iterator<Item = (HashSet<&'a Marker>, Vec<usize>)> {
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

        NextLevelIterator::explore(self, k, gamma.clone())
    }

    fn follow_sp_sm(
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
}

//  _   _           _   _                   _ ___ _                 _
// | \ | | _____  _| |_| |    _____   _____| |_ _| |_ ___ _ __ __ _| |_ ___  _ __
// |  \| |/ _ \ \/ / __| |   / _ \ \ / / _ \ || || __/ _ \ '__/ _` | __/ _ \| '__|
// | |\  |  __/>  <| |_| |__|  __/\ V /  __/ || || ||  __/ | | (_| | || (_) | |
// |_| \_|\___/_/\_\\__|_____\___| \_/ \___|_|___|\__\___|_|  \__,_|\__\___/|_|
//

/// Explore all feasible variable associations in a level from a set of states
/// and resulting possible states reached for theses associations.
struct NextLevelIterator<'a> {
    /// TODO: only keep the automata (and reimplement follow_sp_sm here?)
    indexed_dag: &'a IndexedDag,

    /// Set of markers that can be reached in this level.
    expected_markers: Vec<&'a Marker>,

    /// Set of states we start the run from.
    gamma: Vec<usize>,

    /// Lol
    stack: Vec<(HashSet<&'a Marker>, HashSet<&'a Marker>)>,
}

impl<'a> NextLevelIterator<'a> {
    /// Start the exporation from the input set of states `gamma`.
    fn explore(
        indexed_dag: &'a IndexedDag,
        expected_markers: HashSet<&'a Marker>,
        gamma: Vec<usize>,
    ) -> NextLevelIterator<'a> {
        NextLevelIterator {
            indexed_dag,
            expected_markers: expected_markers.into_iter().collect(),
            gamma: gamma,
            stack: vec![(HashSet::new(), HashSet::new())],
        }
    }
}

impl<'a> Iterator for NextLevelIterator<'a> {
    type Item = (HashSet<&'a Marker>, Vec<usize>);

    fn next(&mut self) -> Option<(HashSet<&'a Marker>, Vec<usize>)> {
        while let Some((mut s_p, mut s_m)) = self.stack.pop() {
            let mut gamma2 = Some(self.indexed_dag.follow_sp_sm(&self.gamma, &s_p, &s_m));

            if gamma2.as_ref().unwrap().is_empty() {
                continue;
            }

            while s_p.len() + s_m.len() < self.expected_markers.len() {
                let depth = s_p.len() + s_m.len();
                s_p.insert(self.expected_markers[depth]);
                gamma2 = Some(self.indexed_dag.follow_sp_sm(&self.gamma, &s_p, &s_m));

                if !gamma2.as_ref().unwrap().is_empty() {
                    // If current pair Sp/Sm is feasible, add the other branch
                    // to the stack.
                    let mut new_s_p = s_p.clone();
                    let mut new_s_m = s_m.clone();
                    new_s_m.insert(self.expected_markers[depth]);
                    new_s_p.remove(self.expected_markers[depth]);
                    self.stack.push((new_s_p, new_s_m));
                } else {
                    // Overwise, the other branch has to be feasible.
                    s_p.remove(self.expected_markers[depth]);
                    s_m.insert(self.expected_markers[depth]);
                    gamma2 = None;
                }
            }

            let gamma2 = match gamma2 {
                None => self.indexed_dag.follow_sp_sm(&self.gamma, &s_p, &s_m),
                Some(val) => val,
            };

            return Some((s_p, gamma2));
        }

        None
    }
}
