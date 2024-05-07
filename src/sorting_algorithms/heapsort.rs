use crate::sorting_algorithms::SortGraph;
use std::fmt;

pub struct Heap<'a, 'b> {
    pub graph: &'a mut SortGraph<'b, 'b>,
    pub nodes: Vec<i32>,
}

impl fmt::Display for Heap<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, &val) in self.nodes.iter().enumerate() {
            let left_idx = 2 * idx + 1;
            let right_idx = 2 * idx + 2;

            if idx > self.total_filled() as usize {
                write!(f, "{}\n", val)?;
                continue;
            }

            write!(f, "{} -> ", val)?;
            if left_idx < self.nodes.len() {
                write!(f, "{} ", self.nodes[left_idx])?;
            }

            if right_idx < self.nodes.len() {
                write!(f, "{}", self.nodes[right_idx])?;
            }

            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl<'a, 'b> Heap<'a, 'b> {
    pub fn new(graph: &'a mut SortGraph<'b, 'b>) -> Self {
        Self {
            graph,
            nodes: Vec::new(),
        }
    }

    pub fn from_graph(graph: &'a mut SortGraph<'b, 'b>) -> Self {
        let nodes = graph.values.clone();
        Self { graph, nodes }
    }
    pub fn insert(&mut self, value: i32) {
        self.nodes.push(value);
        let mut idx = self.nodes.len() - 1;
        while idx > 0 {
            let parent = (idx - 1) / 2;
            if self.nodes[parent] < self.nodes[idx] {
                self.nodes.swap(parent, idx);
                idx = parent;
            } else {
                break;
            }
        }
    }
    pub fn update_graph(&mut self) {
        self.graph.values = self.nodes.clone();
    }
    pub fn update_from_slice(&mut self, vals: &[i32]) {
        for val in vals {
            self.insert(*val);
        }
        self.update_graph();
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn clear(&mut self) {
        self.nodes.clear()
    }

    pub fn extract_max(&mut self) -> Option<i32> {
        if self.is_empty() {
            return None;
        }
        let max = self.nodes.swap_remove(0);
        if !self.nodes.is_empty() {
            self.sift_down(0, self.nodes.len() - 1);
        }
        Some(max)
    }
    fn sift_down(&mut self, idx: usize, end: usize) {
        let mut idx = idx;
        let mut child = 2 * idx + 1;
        while child < end {
            let right = child + 1;
            if right < end && self.nodes[right] > self.nodes[child] {
                child = right;
            }
            if self.nodes[child] > self.nodes[idx] {
                self.nodes.swap(idx, child);
                self.graph.display_simple_swap_graph(Some((idx, child)));
                self.update_graph();
                idx = child;
                child = 2 * idx + 1;
            } else {
                break;
            }
        }
    }
    pub fn sift_up(&mut self, mut idx: usize) {
        while idx > 0 {
            let parent_idx = (idx - 1) / 2;
            if self.nodes[idx] > self.nodes[parent_idx] {
                self.nodes.swap(idx, parent_idx);
                self.graph
                    .display_simple_swap_graph(Some((idx, parent_idx)));
                self.update_graph();
                idx = parent_idx;
            } else {
                break;
            }
        }
    }
    fn build_max_heap(&mut self) {
        let start = (self.nodes.len() / 2).saturating_sub(1);
        for i in (0..=start).rev() {
            self.sift_down(i, self.nodes.len());
        }
    }

    pub fn heapsort(&mut self) {
        self.graph.display_graph();
        self.build_max_heap();
        for end in (1..self.nodes.len()).rev() {
            self.nodes.swap(0, end);
            self.graph.display_simple_swap_graph(Some((0, end)));
            self.update_graph();
            self.graph.display_simple_swap_graph(Some((0, end)));
            self.sift_down(0, end);
        }
        self.graph.display_graph();
    }
    pub fn get_parent(&self, position: usize) -> i32 {
        let parent_loc = (position) / 2;
        self.nodes[parent_loc]
    }
    pub fn get_parent_loc(&self, position: usize) -> usize {
        (position) / 2
    }
    pub fn get_sibling(&self, position: usize) -> i32 {
        if position % 2 == 0 {
            self.nodes[position - 1]
        } else {
            self.nodes[position + 1]
        }
    }

    pub fn get_children(&self, position: usize) -> (i32, i32) {
        let left_loc = 2 * position + 1;
        let right_loc = 2 * position + 2;

        (self.nodes[left_loc], self.nodes[right_loc])
    }

    fn get_depth(idx: usize) -> usize {
        let mut depth = 0;
        let mut current_idx = idx;
        while current_idx > 0 {
            depth += 1;
            current_idx = (current_idx - 1) / 2;
        }
        depth
    }
    pub fn depth(&self) -> u32 {
        self.nodes.len().ilog2()
    }

    pub fn total_filled(&self) -> u32 {
        let mut last_filled_depth = self.depth() - 1;
        let mut total_filled = 0;
        while last_filled_depth > 0 {
            total_filled += last_filled_depth.pow(2);
            last_filled_depth -= 1;
        }
        total_filled
    }
    pub fn remainder(&self) -> u32 {
        let len = self.nodes.len();
        let remainder = len as u32 - self.total_filled();
        remainder
    }
    pub fn available(&self) -> u32 {
        self.nodes.len() as u32 - self.remainder()
    }

    pub fn get_cousins(&self, idx: usize) -> Vec<i32> {
        // Calculate the depth of a given index

        let node_depth = Self::get_depth(idx);
        let mut cousins = Vec::new();
        let start_level_idx = 2usize.pow(node_depth as u32) - 1;
        let end_level_idx = 2usize.pow(node_depth as u32 + 1) - 1;

        // Parent of the current node
        let parent_idx = if idx == 0 { usize::MAX } else { (idx - 1) / 2 };

        let len = self.nodes.len();
        for i in start_level_idx..end_level_idx {
            if i >= len {
                break;
            }
            if (i - 1) / 2 != parent_idx {
                cousins.push(self.nodes[i]);
            }
        }

        cousins
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }
    pub fn heapify(vals: &Vec<i32>, graph: &'a mut SortGraph<'b, 'b>) -> Self {
        let mut nodes = Vec::with_capacity(vals.len());
        for (idx, val) in vals.iter().enumerate() {
            nodes.push(*val);
            if nodes.is_empty() {
                continue;
            }
            if *val > nodes[0] {
                nodes.swap(0, idx);
            }
        }
        graph.values = nodes.clone();
        Self { graph, nodes }
    }
}
