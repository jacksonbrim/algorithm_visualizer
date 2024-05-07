use crate::map::Map;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct BFS<'a> {
    pub map: &'a Map,
    start: (usize, usize),
    end: (usize, usize),
    visited: HashSet<(usize, usize)>,
    queue: VecDeque<(usize, usize)>,
    parent: HashMap<(usize, usize), (usize, usize)>, // To track the path
}

impl<'a> BFS<'a> {
    pub fn new(map: &'a Map) -> Self {
        let start = map.start;
        let end = map.end;
        let mut bfs = BFS {
            map,
            start,
            end,
            visited: HashSet::new(),
            queue: VecDeque::new(),
            parent: HashMap::new(),
        };
        bfs.queue.push_back(start);
        bfs.visited.insert(start);
        bfs.parent.insert(start, start); // Initialize the parent of the start node to itself
        bfs
    }

    pub fn run(&mut self) -> Option<Vec<(usize, usize)>> {
        while let Some(current) = self.queue.pop_front() {
            self.display_visited();
            if current == self.end {
                self.map.update_audio(0.0);
                return Some(self.get_path(self.end));
            }
            for neighbor in self.map.get_neighbors(current.0, current.1) {
                if !self.visited.contains(&neighbor) {
                    self.queue.push_back(neighbor);
                    self.visited.insert(neighbor);
                    self.parent.insert(neighbor, current); // Track where we came from
                    let dist =
                        Map::manhattan_distance(neighbor.0, neighbor.1, self.end.0, self.end.1);
                    self.map.play_distance(dist as u32, neighbor);
                }
            }
        }
        None // Return None if the end is not reachable
    }
    pub fn display_visited(&self) {
        let visited = &self.visited;
        self.map.display_visited(&visited);
    }
    pub fn display_path(&mut self, path: Option<Vec<(usize, usize)>>) {
        if let Some(p) = path {
            self.map.display_path(&p);
        }
    }

    fn get_path(&self, mut current: (usize, usize)) -> Vec<(usize, usize)> {
        let mut path = Vec::new();
        // Follow the parent nodes from the end to the start
        while let Some(&parent) = self.parent.get(&current) {
            path.push(current);
            if current == parent {
                // Stop if the current node is the start node
                break;
            }
            current = parent;
        }
        path.reverse(); // The path is constructed backwards, so reverse it
        path
    }
}
