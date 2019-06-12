use std::collections::{HashMap, HashSet};
use std::iter;

/// Represent the partitioning into levels of a product graph.
///
/// A same vertex can be store in several levels, and this level hierarchy can
/// be accessed rather efficiently.
#[derive(Debug)]
pub struct LevelSet {
    /// Index level contents: `level id` -> `vertex id's list`.
    levels: HashMap<usize, Vec<usize>>,

    /// Index the id of a vertex iner to a level:
    ///     `(level id, vertex id)` -> `vertex position`.
    /// It can also be used to check if a pair `(level, vertex)` is already
    /// represented in the structure.
    vertex_index: HashMap<(usize, usize), usize>,
}

impl LevelSet {
    pub fn new() -> LevelSet {
        LevelSet {
            levels:       HashMap::new(),
            vertex_index: HashMap::new(),
        }
    }

    pub fn has_level(&self, level: usize) -> bool {
        self.levels.contains_key(&level)
    }

    pub fn get_level(&self, level: usize) -> Option<&Vec<usize>> {
        self.levels.get(&level)
    }

    pub fn get_nb_levels(&self) -> usize {
        self.levels.len()
    }

    pub fn get_vertex_index(&self, level: usize, vertex: usize) -> Option<usize> {
        self.vertex_index.get(&(level, vertex)).cloned()
    }

    /// Iterate over pairs (vertex, vertex_index) of a level
    pub fn iter_level<'a>(&'a self, level: usize) -> impl Iterator<Item = (usize, usize)> + 'a {
        let vertices = self.levels[&level].iter();
        let levels = iter::repeat(level);

        levels
            .zip(vertices)
            .map(move |(level, &vertex)| (vertex, self.vertex_index[&(level, vertex)]))
    }

    /// Save a vertex in a level, the vertex need to be unique inside this level
    /// but can be registered in other levels.
    pub fn register(&mut self, level: usize, vertex: usize) {
        let levels = &mut self.levels;
        let vertex_index = &mut self.vertex_index;

        // Insert the vertex in self.level, and return its index
        let insert_in_level = || {
            // Create the level if necessary
            let level = levels.entry(level).or_insert(Vec::new());
            // Add the vertex to the level, and return its index
            level.push(vertex);
            level.len() - 1
        };

        // If the pair (level, vertex) is not part of the structure, add it
        vertex_index
            .entry((level, vertex))
            .or_insert_with(insert_in_level);
    }

    /// Remove a set of vertices from a level, if the level is left empty, it is
    /// then removed.
    pub fn remove_from_level(&mut self, level: usize, del_vertices: &HashSet<usize>) {
        // TODO: in place deletion (may be doable with unsafe rust?)
        let mut new_level = Vec::new();
        let old_level = &self.levels[&level];

        for &old_vertex in old_level {
            if !del_vertices.contains(&old_vertex) {
                self.vertex_index
                    .insert((level, old_vertex), new_level.len());
                new_level.push(old_vertex);
            } else {
                self.vertex_index.remove(&(level, old_vertex));
            }
        }

        if !new_level.is_empty() {
            self.levels.insert(level, new_level);
        } else {
            self.levels.remove(&level);
        }
    }
}
