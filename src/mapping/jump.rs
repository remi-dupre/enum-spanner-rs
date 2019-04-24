use std::cmp::max;
use std::collections::{HashMap, HashSet};

use super::levelset::LevelSet;

/// Generic Jump function inside a product DAG.
///
/// The DAG will be built layer by layer by specifying the adjacency matrix from one level to the
/// next one, an adjancency matrix can specify the structure inside of a level, made of
/// 'assignation edges'. The goal of the structure is to be able to be able to navigate quickly
/// from the last to the first layer by being able to skip any path that do not contain any
/// assignation edges.
pub struct Jump {
    /// Represent levels in the levelset, which will be built on top of one another.
    levelset: LevelSet,
    /// Last level that was built.
    last_level: usize,

    /// Set of vertices that can't be jumped since it has an ingoing non-jumpable edge.
    /// NOTE: it may only be required to store it for the last level.
    nonjump_vertices: HashSet<(usize, usize)>,

    /// Closest level where an assignation is done accessible from any node.
    jl: HashMap<(usize, usize), usize>,
    /// Set of levels accessible from any level using `jl`.
    rlevel: HashMap<usize, HashSet<usize>>,
    /// Reverse of `rlevel`.
    rev_rlevel: HashMap<usize, HashSet<usize>>,
    /// For any pair of level `(i, j)` such that i is in the level `rlevel[j]`, `reach[i, j]` is
    /// the accessibility matrix of vertices from level i to level j
    reach: HashMap<(usize, usize), ()>,
}

impl Jump {
    pub fn new<T>(initial_level: T, nonjump_adj: &Vec<Vec<usize>>) -> Jump
    where
        T: Iterator<Item = usize>,
    {
        let mut jump = Jump {
            levelset: LevelSet::new(),
            last_level: 0,
            nonjump_vertices: HashSet::new(),
            jl: HashMap::new(),
            rlevel: HashMap::new(),
            rev_rlevel: HashMap::new(),
            reach: HashMap::new(),
        };

        jump.rlevel.insert(0, HashSet::new());
        jump.rev_rlevel.insert(0, HashSet::new());

        for state in initial_level {
            jump.levelset.register(state, 0);
            jump.jl.insert((0, state), 0);
        }

        jump.extend_level(0, nonjump_adj);
        jump
    }

    /// Compute next level given the adjacency list of jumpable edges from current level to the
    /// next one and adjacency list of non-jumpable edges inside the next level.
    pub fn init_next_level(&mut self, jump_adj: &Vec<Vec<usize>>, nonjump_adj: &Vec<Vec<usize>>) {
        let levelset = &mut self.levelset;
        let jl = &mut self.jl;

        let last_level = self.last_level;
        let next_level = self.last_level + 1;

        // NOTE: this clone is only necessary for the borrow checker.
        let last_level_vertices = levelset.get_level(last_level).unwrap().clone();

        // Register jumpable transitions from this level to the next one
        for source in last_level_vertices {
            for target in &jump_adj[source] {
                let target_jl = *jl.entry((next_level, *target)).or_insert_with(|| {
                    levelset.register(next_level, *target);
                    0
                });

                if self.nonjump_vertices.contains(&(last_level, source)) {
                    jl.insert((next_level, *target), last_level);
                } else {
                    let source_jl = jl[&(last_level, source)];
                    jl.insert((next_level, *target), max(source_jl, target_jl));
                }
            }
        }

        levelset
            .get_level(next_level)
            .expect("Behaviour not implemented for empty output");

        // NOTE: isn't there a better way of organizing this?
        self.extend_level(next_level, nonjump_adj);
        self.last_level = next_level;
    }

    /// Extend current level by reading non-jumpable edges inside the given level.
    fn extend_level(&mut self, level: usize, nonjump_adj: &Vec<Vec<usize>>) {
        let levelset = &mut self.levelset;
        let nonjump_vertices = &mut self.nonjump_vertices;
        let old_level = levelset.get_level(level).unwrap().clone();

        for source in old_level {
            for target in &nonjump_adj[source] {
                levelset.register(level, *target);
                nonjump_vertices.insert((level, *target));
            }
        }
    }
}
