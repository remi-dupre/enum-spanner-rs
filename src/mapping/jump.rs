use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter;

use super::super::matrix::Matrix;
use super::levelset::LevelSet;

/// Generic Jump function inside a product DAG.
///
/// The DAG will be built layer by layer by specifying the adjacency matrix from
/// one level to the next one, an adjancency matrix can specify the structure
/// inside of a level, made of 'assignation edges'. The goal of the structure is
/// to be able to be able to navigate quickly from the last to the first layer
/// by being able to skip any path that do not contain any assignation edges.
pub struct Jump {
    /// Represent levels in the levelset, which will be built on top of one
    /// another.
    levelset: LevelSet,
    /// Last level that was built.
    last_level: usize,

    /// Set of vertices that can't be jumped since it has an ingoing
    /// non-jumpable edge. NOTE: it may only be required to store it for the
    /// last level.
    nonjump_vertices: HashSet<(usize, usize)>,
    /// Keep track of number of jumps to a given vertex.
    count_ingoing_jumps: HashMap<(usize, usize), usize>,

    /// Closest level where an assignation is done accessible from any node.
    jl: HashMap<(usize, usize), usize>,
    /// Set of levels accessible from any level using `jl`.
    rlevel: HashMap<usize, HashSet<usize>>,
    /// Reverse of `rlevel`.
    rev_rlevel: HashMap<usize, HashSet<usize>>,
    /// For any pair of level `(i, j)` such that i is in the level `rlevel[j]`,
    /// `reach[i, j]` is the accessibility matrix of vertices from level i
    /// to level j
    reach: HashMap<(usize, usize), Matrix<bool>>,
}

impl Jump {
    pub fn new<T>(initial_level: T, nonjump_adj: &Vec<Vec<usize>>) -> Jump
    where
        T: Iterator<Item = usize>,
    {
        let mut jump = Jump {
            levelset:            LevelSet::new(),
            last_level:          0,
            nonjump_vertices:    HashSet::new(),
            count_ingoing_jumps: HashMap::new(),
            jl:                  HashMap::new(),
            rlevel:              HashMap::new(),
            rev_rlevel:          HashMap::new(),
            reach:               HashMap::new(),
        };

        jump.rlevel.insert(0, HashSet::new());
        jump.rev_rlevel.insert(0, HashSet::new());

        for state in initial_level {
            jump.levelset.register(state, 0);
            jump.jl.insert((0, state), 0);
            jump.count_ingoing_jumps.insert((0, state), 0);
        }

        // Init first level
        jump.extend_level(0, nonjump_adj);
        for &vertex in jump.levelset.get_level(0).unwrap() {
            jump.count_ingoing_jumps.insert((0, vertex), 0);
        }

        jump
    }

    /// Compute next level given the adjacency list of jumpable edges from
    /// current level to the next one and adjacency list of non-jumpable
    /// edges inside the next level.
    pub fn init_next_level(&mut self, jump_adj: &Vec<Vec<usize>>, nonjump_adj: &Vec<Vec<usize>>) {
        let nonjump_vertices = &self.nonjump_vertices;
        let levelset = &mut self.levelset;
        let jl = &mut self.jl;

        let last_level = self.last_level;
        let next_level = self.last_level + 1;

        // NOTE: this clone is only necessary for the borrow checker.
        let last_level_vertices = levelset.get_level(last_level).unwrap().clone();

        // Register jumpable transitions from this level to the next one
        for source in last_level_vertices {
            // Notice that `source_jl` can be None, however, if it is not in
            // nonjump_vertices it is sure that it is not None since it was
            // necessary added by following an atomic transition.
            let source_jl = jl.get(&(last_level, source)).cloned();

            for &target in &jump_adj[source] {
                // Compute the level target will jump to, depending if there is already an
                // assigned jump level for target or not.
                let cmpt_jump_level = |previous_jl| {
                    if nonjump_vertices.contains(&(last_level, source)) {
                        last_level
                    } else {
                        match previous_jl {
                            None => source_jl.unwrap(),
                            Some(previous_jl) => max(source_jl.unwrap(), previous_jl),
                        }
                    }
                };

                // Update the jump level in a single hashmap access.
                jl.entry((next_level, target))
                    .and_modify(|target_jl| {
                        *target_jl = cmpt_jump_level(Some(*target_jl));
                    })
                    .or_insert_with(|| {
                        levelset.register(next_level, target);
                        cmpt_jump_level(None)
                    });
            }
        }

        // If at some point the next level is not reached, the output will be empty
        // anyway.
        if levelset.get_level(next_level) == None {
            return;
        }

        // NOTE: isn't there a better way of organizing this?
        self.extend_level(next_level, nonjump_adj);
        self.init_reach(next_level, jump_adj);
        self.last_level = next_level;
    }

    pub fn is_disconnected(&self) -> bool {
        !self.levelset.has_level(self.last_level)
    }

    /// Jump to the next relevant level from vertices in gamma at a given level.
    /// A relevent level has a node from which there is a path to gamma and
    /// that has an ingoing assignation.
    ///
    /// NOTE: It may be possible to return an iterator to refs of usize, but the
    /// autoref seems to not do the work.
    pub fn jump<T>(&self, level: usize, gamma: T) -> Option<(usize, Vec<usize>)>
    where
        T: Clone + Iterator<Item = usize>,
    {
        let jump_level = gamma
            .clone()
            .filter_map(|vertex| self.jl.get(&(level, vertex)))
            .max();

        let jump_level = match jump_level {
            None => return Some((level, Vec::new())),
            Some(&lvl) if lvl == level => return Some((level, Vec::new())),
            Some(&lvl) => lvl,
        };

        // NOTE: could convince Rust that the lifetime of this iterator is ok to return
        // a map iterator.
        let jump_level_vertices = self.levelset.get_level(jump_level).unwrap();

        let gamma2 = jump_level_vertices
            .iter()
            .enumerate()
            .filter(|&(l, _)| {
                // NOTE: Maybe it could be more efficient to compute indices `k` before the
                // filter.
                gamma.clone().any(
                    |source| match self.levelset.get_vertex_index(level, source) {
                        Some(k) => self.reach[&(jump_level, level)][(l, k)],
                        None => false,
                    },
                )
            })
            .map(|(_, target)| *target)
            .collect();

        Some((jump_level, gamma2))
    }

    /// Get the vertices that are in the final layer
    pub fn finals(&self) -> HashSet<usize> {
        if self.is_disconnected() {
            return HashSet::new();
        }

        self.levelset
            .get_level(self.last_level)
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }

    pub fn get_nb_levels(&self) -> usize {
        self.levelset.get_nb_levels()
    }

    /// Extend current level by reading non-jumpable edges inside the given
    /// level.
    fn extend_level(&mut self, level: usize, nonjump_adj: &Vec<Vec<usize>>) {
        let levelset = &mut self.levelset;
        let nonjump_vertices = &mut self.nonjump_vertices;
        let old_level = levelset.get_level(level).unwrap().clone();

        for source in old_level {
            for &target in &nonjump_adj[source] {
                levelset.register(level, target);
                nonjump_vertices.insert((level, target));
            }
        }
    }

    // Compute reach and rlevel, that is the effective jump points to all levels
    // reachable from the current level.
    fn init_reach(&mut self, level: usize, jump_adj: &Vec<Vec<usize>>) {
        let reach = &mut self.reach;
        let rlevel = &mut self.rlevel;
        let rev_rlevel = &mut self.rev_rlevel;
        let jl = &self.jl;
        let count_ingoing_jumps = &mut self.count_ingoing_jumps;

        let curr_level = self.levelset.get_level(level).unwrap();

        // Build rlevel as the image of current level by jl
        rlevel.insert(
            level,
            curr_level
                .iter()
                .filter_map(|&source| jl.get(&(level, source)).map(|&target| target))
                .collect(),
        );

        // Update rev_rlevel for sublevels
        rev_rlevel.insert(level, HashSet::new());

        for sublevel in &rlevel[&level] {
            rev_rlevel.get_mut(sublevel).unwrap().insert(level);
        }

        // Compute the adjacency between current level and the previous one.
        let prev_level = self.levelset.get_level(level - 1).unwrap();
        let mut new_reach = Matrix::new(prev_level.len(), curr_level.len(), false);

        for &source in prev_level {
            let id_source = self.levelset.get_vertex_index(level - 1, source).unwrap();

            for &target in &jump_adj[source] {
                let id_target = self.levelset.get_vertex_index(level, target).unwrap();
                *new_reach.at(id_source, id_target) = true;
            }
        }

        reach.insert((level - 1, level), new_reach);

        // Compute by a dynamic algorithm the adjacency of current level with all its
        // sublevels.
        for &sublevel in &rlevel[&level] {
            // This eliminates the stupid cast of level 0.
            // TODO: fix this hardcoded behaviour.
            if sublevel >= level - 1 {
                continue;
            }

            reach.insert(
                (sublevel, level),
                &reach[&(sublevel, level - 1)] * &reach[&(level - 1, level)],
            );
        }

        if !rlevel[&level].contains(&(level - 1)) {
            reach.remove(&(level - 1, level));
        }

        // Init Jump counters for current level
        for &vertex in curr_level {
            count_ingoing_jumps.insert((level, vertex), 0);
        }

        // Update Jump counters previous level
        for &sublevel in &rlevel[&level] {
            let adjacency = &reach[&(sublevel, level)];

            for (vertex, vertex_index) in self.levelset.iter_level(sublevel) {
                let nb_pointers: usize = adjacency
                    .iter_row(vertex_index)
                    .map(|&x| if x { 1 } else { 0 })
                    .sum();

                if nb_pointers != 0 {
                    *count_ingoing_jumps.get_mut(&(sublevel, vertex)).unwrap() += nb_pointers;
                }
            }
        }
    }

    /// Remove all useless nodes inside current level. A useless node is a node
    /// from which there is no path of assignation to a node which can be jumped
    /// to.
    pub fn clean_level(&mut self, level: usize, jump_adj: &Vec<Vec<usize>>) -> bool {
        if level == 0 {
            // TODO: fix the reach[(0, 0)] exception
            return false;
        }

        let curr_level = match self.levelset.get_level(level) {
            None => return false,
            Some(vertices) => vertices,
        };

        // Run over the level and eliminate all path that are not usefull ie. paths that
        // don't access to a jumpable vertex
        let mut seen = HashSet::new();
        let mut lvl_vertices: HashSet<_> = curr_level.iter().cloned().collect();
        let mut del_vertices: HashSet<_> = curr_level.iter().cloned().collect();

        for &start in curr_level {
            if seen.contains(&start) {
                continue;
            }

            let mut heap = vec![(start, vec![start])];

            while let Some((source, mut path)) = heap.pop() {
                seen.insert(source);

                // If the path can be identified as usefull, remove it from the set of vertices
                // to delete.
                let usefull_path = self.count_ingoing_jumps[&(level, source)] > 0
                    || jump_adj[source].iter().any(|&vertex| {
                        lvl_vertices.contains(&vertex) && !del_vertices.contains(&vertex)
                    });

                if usefull_path {
                    for vertex in &path {
                        del_vertices.remove(vertex);
                    }

                    path.clear();
                }

                for target in &jump_adj[source] {
                    if lvl_vertices.contains(target) && !seen.contains(target) {
                        debug_assert!(del_vertices.contains(target));
                        let mut target_path = path.to_vec();
                        target_path.push(*target);
                        heap.push((*target, target_path));
                    }
                }
            }
        }

        if del_vertices.is_empty() {
            return false;
        }

        let removed_columns: Vec<_> = del_vertices
            .iter()
            .map(|&vertex| self.levelset.get_vertex_index(level, vertex).unwrap())
            .collect();

        for &col in &removed_columns {
            for &sublevel in &self.rlevel[&level] {
                assert!(
                    col < self.reach[&(sublevel, level)].get_width(),
                    "Index {} inconsistant for {} {}",
                    col,
                    sublevel,
                    level
                );
            }
        }

        // Update the levelset and update borrowed value
        self.levelset.remove_from_level(level, &del_vertices);
        let default_level = Vec::new();
        let curr_level = self.levelset.get_level(level).unwrap_or(&default_level);

        for &vertex in &del_vertices {
            self.jl.remove(&(level, vertex));
            self.count_ingoing_jumps.remove(&(level, vertex));
        }

        if !self.levelset.has_level(level) {
            for &sublevel in &self.rlevel[&level] {
                let adjacency = &self.reach[&(sublevel, level)];

                // FIXME: this should be placed before branchement?
                for (vertex, vertex_index) in self.levelset.iter_level(sublevel) {
                    let nb_pointers: usize = adjacency
                        .iter_row(vertex_index)
                        .map(|&x| if x { 1 } else { 0 })
                        .sum();

                    if nb_pointers != 0 {
                        *self
                            .count_ingoing_jumps
                            .get_mut(&(sublevel, vertex))
                            .unwrap() += nb_pointers;
                    }
                }
            }

            for &uplevel in &self.rev_rlevel[&level] {
                self.reach.remove(&(level, uplevel));
                self.rlevel.get_mut(&uplevel).unwrap().remove(&level);
            }

            for &sublevel in &self.rlevel[&level] {
                self.rev_rlevel.get_mut(&sublevel).unwrap().remove(&level);
            }

            self.rlevel.remove(&level);
            self.rev_rlevel.remove(&level);
        } else {
            // Update rlevel
            let new_rlevel: HashSet<_> = curr_level
                .iter()
                .filter_map(|&vertex| self.jl.get(&(level, vertex)).cloned())
                .collect();

            // Update jump counters to sublevels, if a sublevel is removed from rlevel, then
            // we need to remove jump pointers from any vertex of the level, overwise only
            // from removed vertices.
            // TODO: I think it's covered by next case (removed sublevels have all
            // subvertices removed)
            for &sublevel in self.rlevel[&level].difference(&new_rlevel) {
                for &vertex in self.levelset.get_level(sublevel).unwrap() {
                    let adjacency = &self.reach[&(sublevel, level)];
                    let vertex_index = self.levelset.get_vertex_index(sublevel, vertex).unwrap();
                    let nb_removed: usize = adjacency
                        .iter_row(vertex_index)
                        .map(|&x| if x { 1 } else { 0 })
                        .sum();

                    if nb_removed > 0 {
                        *self
                            .count_ingoing_jumps
                            .get_mut(&(sublevel, vertex))
                            .unwrap() -= nb_removed;
                    }
                }
            }

            for &sublevel in &new_rlevel {
                for &vertex in self.levelset.get_level(sublevel).unwrap() {
                    let adjacency = &self.reach[&(sublevel, level)];
                    let vertex_index = self.levelset.get_vertex_index(sublevel, vertex).unwrap();
                    let nb_removed: usize = removed_columns
                        .iter()
                        .map(|&col| if adjacency[(vertex_index, col)] { 1 } else { 0 })
                        .sum();

                    if nb_removed > 0 {
                        *self
                            .count_ingoing_jumps
                            .get_mut(&(sublevel, vertex))
                            .unwrap() -= nb_removed;
                    }
                }
            }

            // Remove deprecated links in reach and rlevel
            for &sublevel in self.rlevel[&level].difference(&new_rlevel) {
                self.rev_rlevel.get_mut(&sublevel).unwrap().remove(&level);
                self.reach.remove(&(sublevel, level));
            }

            self.rlevel.insert(level, new_rlevel);

            // Update reach
            for &vertex in &del_vertices {
                self.count_ingoing_jumps.remove(&(level, vertex));
            }

            for &uplevel in &self.rev_rlevel[&level] {
                self.reach.insert(
                    (level, uplevel),
                    self.reach[&(level, uplevel)]
                        .truncate(removed_columns.iter().cloned(), iter::empty()),
                );
            }

            for &sublevel in &self.rlevel[&level] {
                self.reach.insert(
                    (sublevel, level),
                    self.reach[&(sublevel, level)]
                        .truncate(iter::empty(), removed_columns.iter().cloned()),
                );
            }
        }

        true
    }
}

impl fmt::Debug for Jump {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ((sublevel, level), adj) in &self.reach {
            write!(
                f,
                "-----\n{} <- {}:\n{}: {:?}\n{}: {:?}\n{:?}",
                sublevel,
                level,
                sublevel,
                self.levelset.get_level(*sublevel),
                level,
                self.levelset.get_level(*level),
                adj
            )?;
        }

        Ok(())
    }
}
