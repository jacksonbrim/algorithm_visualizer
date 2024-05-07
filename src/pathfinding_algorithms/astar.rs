use crate::pathfinding_algorithms::Map;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::Duration;

// Assuming 'Map' is defined elsewhere and it has 'is_traversable' and 'get_neighbors' methods.
struct Node {
    position: (usize, usize),
    f_score: usize, // Total cost of node
    g_score: usize, // Cost from start to node
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f_score == other.f_score
    }
}

impl Eq for Node {}

impl Ord for Node {
    // Standard comparison based on f_score
    fn cmp(&self, other: &Self) -> Ordering {
        self.f_score.cmp(&other.f_score)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct AStar<'a, 'b> {
    map: &'a Map<'b, 'b>,
    start: (usize, usize),
    end: (usize, usize),
    open_set: BinaryHeap<Reverse<Node>>, // Use Reverse for min-heap behavior
    came_from: HashMap<(usize, usize), (usize, usize)>,
    g_score: HashMap<(usize, usize), usize>,
    visited: HashSet<(usize, usize)>,
}

impl<'a, 'b> AStar<'a, 'b> {
    pub fn new(map: &'a Map<'b, 'b>) -> Self {
        let start = map.start;
        let end = map.end;
        let mut open_set = BinaryHeap::new();
        let mut g_score = HashMap::new();

        // Initial setup
        g_score.insert(start, 0);
        open_set.push(Reverse(Node {
            position: start,
            f_score: Self::heuristic(start, end),
            g_score: 0,
        }));

        AStar {
            map,
            start,
            end,
            open_set,
            came_from: HashMap::new(),
            g_score,
            visited: HashSet::new(),
        }
    }

    pub fn find_path(&mut self) -> Option<Vec<(usize, usize)>> {
        while let Some(Reverse(current)) = self.open_set.pop() {
            std::thread::sleep(Duration::from_millis(20));
            self.display_visited();
            if current.position == self.end {
                self.map.update_audio(0.0);
                return Some(self.reconstruct_path(current.position));
            }

            self.visited.insert(current.position);
            for neighbor in self
                .map
                .get_neighbors(current.position.0, current.position.1)
            {
                if !self.map.is_traversable(neighbor.0, neighbor.1) {
                    continue;
                }

                let tentative_g_score = self.g_score[&current.position] + 1; // Assume cost from current to neighbor is 1

                if tentative_g_score < *self.g_score.get(&neighbor).unwrap_or(&usize::MAX) {
                    self.came_from.insert(neighbor, current.position);
                    self.g_score.insert(neighbor, tentative_g_score);
                    let f_score = tentative_g_score + Self::heuristic(neighbor, self.end);
                    self.open_set.push(Reverse(Node {
                        position: neighbor,
                        f_score,
                        g_score: tentative_g_score,
                    }));
                    self.map.play_distance(tentative_g_score as u32, neighbor);
                }
            }
        }
        self.map.update_audio(0.0);
        None
    }
    pub fn display_visited(&self) {
        let visited = &self.visited;
        self.map.display_visited(&visited);
    }
    pub fn display_path(&self) {
        let path = self.reconstruct_path(self.end);
        self.map.display_path(&path);
    }

    fn heuristic(start: (usize, usize), end: (usize, usize)) -> usize {
        // Manhattan distance as a heuristic
        ((start.0 as isize - end.0 as isize).abs() + (start.1 as isize - end.1 as isize).abs())
            as usize
    }

    fn reconstruct_path(&self, mut current: (usize, usize)) -> Vec<(usize, usize)> {
        let mut path = Vec::new();
        path.push(current);
        while let Some(&next) = self.came_from.get(&current) {
            path.push(next);
            current = next;
        }
        path.reverse();
        path
    }
}
