use crate::pathfinding_algorithms::Map;
use std::collections::{BinaryHeap, HashMap, HashSet};
#[derive(Debug)]
pub struct Dijkstra<'a, 'b> {
    map: &'a Map<'b, 'b>,
    distances: HashMap<(usize, usize), u32>,
    priority_queue: BinaryHeap<(i32, (usize, usize))>, // Use negative cost for max-heap behavior
    visited: HashSet<(usize, usize)>,
    predecessors: HashMap<(usize, usize), (usize, usize)>, // Store each node's predecessor
}

impl<'a, 'b> Dijkstra<'a, 'b> {
    pub fn new(map: &'a Map<'b, 'b>) -> Self {
        let mut distances = HashMap::new();
        let mut priority_queue = BinaryHeap::new();

        // Initialize distances to a very high value (infinity)
        for y in 0..map.height {
            for x in 0..map.width {
                distances.insert((x, y), u32::MAX);
            }
        }

        // Set the start point distance to 0
        distances.insert(map.start, 0);
        priority_queue.push((0, map.start)); // Start with the start node

        Dijkstra {
            map,
            distances,
            priority_queue,
            visited: HashSet::new(),
            predecessors: HashMap::new(),
        }
    }

    /// Runs the Dijkstra's algorithm to find the shortest path from the start to the end
    pub fn run(&mut self) {
        self.map.display();
        while let Some((current_distance, current_position)) = self.priority_queue.pop() {
            self.display_visited();

            let current_distance = -current_distance as u32; // Convert back to positive

            self.visited.insert(current_position);
            // Early exit if we reached the end point
            if current_position == self.map.end {
                break;
            }

            // If we found a better path to the current node, continue processing
            if current_distance > *self.distances.get(&current_position).unwrap() {
                continue;
            }

            // Check each neighbor
            for neighbor in self
                .map
                .get_neighbors(current_position.0, current_position.1)
            {
                let next = neighbor;
                let new_cost = current_distance + self.map.cost(current_position, next);

                if new_cost < *self.distances.get(&next).unwrap() {
                    // Found a better way to this neighbor
                    self.distances.insert(next, new_cost);
                    self.predecessors.insert(next, current_position); // Update the predecessor
                    self.priority_queue.push((-(new_cost as i32), next)); // Push new cost as negative
                    self.map.play_distance(current_distance, current_position);
                }
            }
        }
        self.display_visited();
        self.map.update_audio(0.0);
        self.display_path();
    }
    pub fn display_visited(&self) {
        let visited = &self.visited;
        self.map.display_visited(&visited);
    }
    pub fn display_path(&self) {
        let path = self.get_path();
        self.map.display_path(&path)
    }

    pub fn get_path(&self) -> Vec<(usize, usize)> {
        let mut path = Vec::new();
        let mut step = self.map.end;

        if !self.predecessors.contains_key(&step) {
            return vec![]; // If there's no path to the end, return empty vector
        }

        while step != self.map.start {
            path.push(step);
            step = *self.predecessors.get(&step).unwrap(); // Retrieve the predecessor of the current step
        }

        path.push(self.map.start); // Add the start position at the end
        path.reverse(); // Reverse to show path from start to end
        path
    }
}
