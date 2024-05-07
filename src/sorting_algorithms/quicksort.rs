use crate::sorting_algorithms::SortGraph;

pub struct QuickSort<'a, 'b, 'c> {
    pub graph: &'a mut SortGraph<'b, 'c>,
    pub values: Vec<i32>,
}

impl<'a, 'b, 'c> QuickSort<'a, 'b, 'c> {
    pub fn new(graph: &'a mut SortGraph<'b, 'c>) -> Self {
        let values = graph.values.clone();
        QuickSort {
            graph,
            values: values.to_vec(),
        }
    }
    pub fn sort(&mut self) {
        self.graph.display_graph();
        self.quick_sort_helper(0, self.values.len() as i32 - 1);
        self.graph.display_graph();
    }
    pub fn values(&self) -> &[i32] {
        &self.values
    }
    pub fn quick_sort_helper(&mut self, low: i32, high: i32) {
        if low < high {
            let pi = self.partition(low, high);
            self.quick_sort_helper(low, pi - 1);
            self.quick_sort_helper(pi + 1, high);
        }
    }
    /// Partition the vector and return the index of the pivot.
    fn partition(&mut self, low: i32, high: i32) -> i32 {
        let pivot = self.values[high as usize];
        let mut i = low - 1;
        for j in low..high {
            if self.values[j as usize] < pivot {
                i += 1;
                self.graph.display_graph_with_highlights(
                    high as usize,
                    low,
                    high,
                    (i as usize, j as usize),
                );
                self.values.swap(i as usize, j as usize);
                self.graph.values.swap(i as usize, j as usize);
                self.graph.display_graph_with_highlights(
                    high as usize,
                    low,
                    high,
                    (i as usize, j as usize),
                );
            }
        }
        self.values.swap((i + 1) as usize, high as usize);
        self.graph.values = self.values.clone();
        self.graph.display_graph_with_highlights(
            high as usize,
            low,
            high,
            ((i + 1) as usize, high as usize),
        );
        self.graph.values.swap((i + 1) as usize, high as usize);
        self.graph.display_graph_with_highlights(
            high as usize,
            low,
            high,
            ((i + 1) as usize, high as usize),
        );

        i + 1
    }
}
