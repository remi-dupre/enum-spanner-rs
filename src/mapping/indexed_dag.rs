// use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter;

use super::super::automaton::Automaton;
use super::super::mapping::{Mapping, Marker};
use super::super::progress::Progress;
use super::jump::Jump;

//  ___           _                   _ ____
// |_ _|_ __   __| | _____  _____  __| |  _ \  __ _  __ _
//  | || '_ \ / _` |/ _ \ \/ / _ \/ _` | | | |/ _` |/ _` |
//  | || | | | (_| |  __/>  <  __/ (_| | |_| | (_| | (_| |
// |___|_| |_|\__,_|\___/_/\_\___|\__,_|____/ \__,_|\__, |
//                                                  |___/

/// DAG built from the product automaton of a variable automaton and a text.
///
/// The structure allows to enumerate efficiently all the distinct matches of
/// the input automata over the input text.
pub struct IndexedDag<'t> {
    automaton: Automaton,
    text:      &'t str,
    jump:      Jump,
}

impl<'t> IndexedDag<'t> {
    pub fn compile(mut automaton: Automaton, text: &str) -> IndexedDag {
        let mut jump = Jump::new(
            iter::once(automaton.get_initial()),
            automaton.get_closure_for_assignations(),
        );

        // NOTE: this copy could be avoided by changing the cached getter's behaviour
        let closure_for_assignations = automaton.get_closure_for_assignations().clone();

        let chars: Vec<_> = text.chars().collect();
        let progress = Progress::from_iter(chars.into_iter());

        for (curr_level, curr_char) in progress.enumerate() {
            let adj_for_char = automaton.get_adj_for_char(curr_char);
            jump.init_next_level(adj_for_char, &closure_for_assignations);

            // Clean levels at exponential depth
            if curr_level > 0 {
                // TODO: this is not as "fancy" as a `x & -x`
                let depth = (2 as usize).pow(curr_level.trailing_zeros());

                for level in ((curr_level - depth + 1)..=curr_level).rev() {
                    jump.clean_level(level, &closure_for_assignations);
                }
            }

            if jump.is_disconnected() {
                break;
            }
        }

        IndexedDag {
            automaton,
            text,
            jump,
        }
    }

    pub fn iter<'d>(&'d self) -> impl Iterator<Item = Mapping<'t>> + 'd {
        IndexedDagIterator::init(self)
    }

    pub fn get_nb_levels(&self) -> usize {
        self.jump.get_nb_levels()
    }

    /// TODO: implement this as an iterable.
    fn next_level<'d>(&'d self, gamma: Vec<usize>) -> NextLevelIterator<'d, 't> {
        let adj = self.automaton.get_rev_assignations();

        // Get list of variables that are part of the level
        // TODO: It might still be slower to just using the list of all variables in the
        // automaton?
        let mut k = HashSet::new();
        let mut stack = gamma.clone();
        let mut marks = HashSet::new();

        for &x in &gamma {
            marks.insert(x);
        }

        while let Some(source) = stack.pop() {
            for (label, target) in &adj[source] {
                k.insert(label.get_marker().unwrap());

                if !marks.contains(target) {
                    marks.insert(*target);
                    stack.push(*target);
                }
            }
        }

        NextLevelIterator::explore(self, k, gamma)
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

//  ___           _                   _
// |_ _|_ __   __| | _____  _____  __| |
//  | || '_ \ / _` |/ _ \ \/ / _ \/ _` |
//  | || | | | (_| |  __/>  <  __/ (_| |
// |___|_| |_|\__,_|\___/_/\_\___|\__,_|
//  ____
// |  _ \  __ _  __ _
// | | | |/ _` |/ _` |
// | |_| | (_| | (_| |
// |____/ \__,_|\__, |
//              |___/

struct IndexedDagIterator<'d, 't> {
    indexed_dag: &'d IndexedDag<'t>,
    stack:       Vec<(usize, Vec<usize>, HashSet<(&'d Marker, usize)>)>,

    curr_level:      usize,
    curr_mapping:    HashSet<(&'d Marker, usize)>, // TODO: HashSet required?
    curr_next_level: NextLevelIterator<'d, 't>,
}

impl<'d, 't> IndexedDagIterator<'d, 't> {
    fn init(indexed_dag: &'d IndexedDag<'t>) -> IndexedDagIterator<'d, 't> {
        let start = indexed_dag
            .jump
            .finals()
            .intersection(&indexed_dag.automaton.finals.iter().map(|x| *x).collect())
            .map(|x| *x)
            .collect();

        IndexedDagIterator {
            indexed_dag,
            stack: vec![(indexed_dag.text.chars().count(), start, HashSet::new())],

            // `curr_next_level` is initialized empty, thus theses values will
            // be replaced before the first iteration.
            curr_next_level: NextLevelIterator::empty(indexed_dag),
            curr_level: usize::default(),
            curr_mapping: HashSet::default(),
        }
    }
}

impl<'d, 't> Iterator for IndexedDagIterator<'d, 't> {
    type Item = Mapping<'t>;

    fn next(&mut self) -> Option<Mapping<'t>> {
        loop {
            // First, consume curr_next_level.
            // NOTE: Using a for loop instead would move the iterator.
            while let Some((s_p, new_gamma)) = self.curr_next_level.next() {
                if new_gamma.is_empty() {
                    continue;
                }

                let mut new_mapping = self.curr_mapping.clone();
                for marker in s_p {
                    new_mapping.insert((marker, self.curr_level));
                }

                if self.curr_level == 0
                    && new_gamma.contains(&self.indexed_dag.automaton.get_initial())
                {
                    return Some(Mapping::from_markers(
                        self.indexed_dag.text,
                        // TODO: compare performance with .iter()
                        new_mapping
                            .into_iter()
                            .map(|(marker, pos)| (marker.clone(), pos)),
                    ));
                } else if let Some((jump_level, jump_gamma)) = self
                    .indexed_dag
                    .jump
                    .jump(self.curr_level, new_gamma.into_iter())
                {
                    if !jump_gamma.is_empty() {
                        self.stack.push((jump_level, jump_gamma, new_mapping));
                    }
                }
            }

            // Overwise, read next element of the stack and init the new
            // `curr_next_level` before restarting the process.
            match self.stack.pop() {
                None => return None,
                Some((level, gamma, mapping)) => {
                    self.curr_level = level;
                    self.curr_mapping = mapping;
                    self.curr_next_level = self.indexed_dag.next_level(gamma)
                }
            }
        }
    }
}

//  _   _           _   _                   _ ___ _                 _
// | \ | | _____  _| |_| |    _____   _____| |_ _| |_ ___ _ __ __ _| |_ ___  _
// __ |  \| |/ _ \ \/ / __| |   / _ \ \ / / _ \ || || __/ _ \ '__/ _` | __/ _ \|
// '__| | |\  |  __/>  <| |_| |__|  __/\ V /  __/ || || ||  __/ | | (_| | || (_)
// | | |_| \_|\___/_/\_\\__|_____\___| \_/ \___|_|___|\__\___|_|
// \__,_|\__\___/|_|
//

/// Explore all feasible variable associations in a level from a set of states
/// and resulting possible states reached for theses associations.
struct NextLevelIterator<'d, 't> {
    /// TODO: only keep the automata (and reimplement follow_sp_sm here?)
    indexed_dag: &'d IndexedDag<'t>,

    /// Set of markers that can be reached in this level.
    expected_markers: Vec<&'d Marker>,

    /// Set of states we start the run from.
    gamma: Vec<usize>,

    /// TODO
    stack: Vec<(HashSet<&'d Marker>, HashSet<&'d Marker>)>,
}

impl<'d, 't> NextLevelIterator<'d, 't> {
    /// An empty iterator.
    fn empty(indexed_dag: &'d IndexedDag<'t>) -> NextLevelIterator<'d, 't> {
        NextLevelIterator {
            stack: Vec::new(), // Initialized with an empty stack to stop iteration instantly.
            indexed_dag,
            expected_markers: Vec::default(),
            gamma: Vec::default(),
        }
    }

    /// Start the exporation from the input set of states `gamma`.
    fn explore(
        indexed_dag: &'d IndexedDag<'t>,
        expected_markers: HashSet<&'d Marker>,
        gamma: Vec<usize>,
    ) -> NextLevelIterator<'d, 't> {
        NextLevelIterator {
            indexed_dag,
            expected_markers: expected_markers.into_iter().collect(),
            gamma,
            stack: vec![(HashSet::new(), HashSet::new())],
        }
    }
}

impl<'d, 't> Iterator for NextLevelIterator<'d, 't> {
    type Item = (HashSet<&'d Marker>, Vec<usize>);

    fn next(&mut self) -> Option<(HashSet<&'d Marker>, Vec<usize>)> {
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
