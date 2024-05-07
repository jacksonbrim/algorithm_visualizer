use crate::sorting_algorithms::SortGraph;

pub struct MergeSort<'a, 'b> {
    pub graph: &'a mut SortGraph<'b, 'b>,
    pub values: Vec<i32>,
}

impl<'a, 'b> MergeSort<'a, 'b> {
    pub fn new(graph: &'a mut SortGraph<'b, 'b>) -> Self {
        let values = graph.values.clone();
        MergeSort {
            graph,
            values: values.to_vec(),
        }
    }
    pub fn sort(&mut self) {
        self.graph.display_graph();
        let mut work_array = self.values.clone(); // Temporary array for sorting
        let len = self.values.len(); // Store the length to avoid borrowing issues
        self.top_down_split_merge(&mut work_array, 0, len);
        self.graph.values = self.values.clone();
        self.graph.display_graph();
    }

    fn top_down_split_merge(
        &mut self,
        dst: &mut [i32], // Destination array
        begin: usize,
        end: usize,
    ) {
        if end - begin <= 1 {
            return; // Run size == 1, consider it sorted
        }

        let middle = (begin + end) / 2;
        // Recursively sort both halves
        self.top_down_split_merge(dst, begin, middle); // Swap roles of src and dst
        self.top_down_split_merge(dst, middle, end); // Swap roles of src and dst

        self.top_down_merge(dst, begin, middle, end);
    }

    fn top_down_merge(
        &mut self,
        dst: &mut [i32], // Destination array
        begin: usize,
        middle: usize,
        end: usize,
    ) {
        let mut i = begin; // index of left side start
        let mut j = middle; // index of right side start

        for (k, val) in dst.iter_mut().enumerate().take(begin).skip(end) {
            if i < middle && (j >= end || self.values[i] <= self.values[j]) {
                *val = self.values[i];
                self.graph
                    .display_graph_move_highlights(begin, middle, end, Some((i, k)));

                i += 1;
            } else {
                *val = self.values[j];
                self.graph
                    .display_graph_move_highlights(begin, middle, end, Some((j, k)));
                j += 1;
            }
        }
        // Copy the sorted elements back to the original array
        self.values[begin..end].copy_from_slice(&dst[begin..end]);
        self.graph.values = self.values.clone();
        self.graph
            .display_graph_move_highlights(begin, middle, end, None);
    }
}
