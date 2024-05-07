pub mod astar;
pub mod audio;
pub mod bfs;
pub mod dijkstra;
pub mod heapsort;
pub mod map;
pub mod mergesort;
pub mod quicksort;
pub mod sorting_graph;

use std::thread;
use std::time::Duration;

use audio::Notes;
use colored::Colorize;

use crate::astar::AStar;
use crate::audio::AudioDevice;
use crate::bfs::BFS;
use crate::dijkstra::Dijkstra;
use crate::heapsort::Heap;
use crate::map::{Direction, Map};
use crate::mergesort::MergeSort;
use crate::quicksort::QuickSort;
use crate::sorting_graph::SortGraph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio_device = AudioDevice::new().unwrap();
    let (tx, handle) = audio_device.play_audio_live();

    let mut map = Map::new("dijkstra's Algorithm", Some(tx), Some(handle));
    map.generate();

    Map::clear_screen();
    Map::reset_cursor();
    let mut dijkstra = Dijkstra::new(&map);
    dijkstra.run();
    map.reset("A*Star Algorithm");
    let mut astar = AStar::new(&map);
    let _astar_path = astar.find_path();
    astar.display_path();
    map.reset("Breadth First Search Algorithm");
    let mut bfs = BFS::new(&map);
    let bfs_path = bfs.run();
    bfs.display_path(bfs_path);
    map.stop_audio();
    map.join_audio();

    let (tx, handle) = audio_device.play_audio_live();

    let mut sort_graph = SortGraph::new("Quick Sort Algorithm", Some(tx), Some(handle));
    let mut quick_sort = QuickSort::new(&mut sort_graph);
    quick_sort.sort();
    // Stop the audio thread
    sort_graph.stop_audio();
    sort_graph.join_audio();

    thread::sleep(Duration::from_millis(1000));

    let (tx, handle) = audio_device.play_audio_live();
    let mut sort_graph = SortGraph::new("Merge Sort Algorithm", Some(tx.clone()), Some(handle));
    let mut merge_sort = MergeSort::new(&mut sort_graph);
    merge_sort.sort();
    // Stop the audio thread
    sort_graph.stop_audio();
    sort_graph.join_audio();

    thread::sleep(Duration::from_millis(1000));

    let (tx, handle) = audio_device.play_audio_live();
    let mut sort_graph = SortGraph::new("HeapSort Algorithm", Some(tx), Some(handle));
    let mut heap = Heap::from_graph(&mut sort_graph);
    heap.heapsort();
    // Stop the audio thread
    sort_graph.stop_audio();
    sort_graph.join_audio();

    Ok(())
}
