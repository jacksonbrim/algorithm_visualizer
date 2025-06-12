#![allow(dead_code)]
pub mod audio;
pub mod pathfinding_algorithms;
pub mod sorting_algorithms;

use std::{env, thread, time::Duration};

use crate::{
    audio::AudioDevice,
    pathfinding_algorithms::{AStar, Dijkstra, Map, BFS},
    sorting_algorithms::{Heap, MergeSort, QuickSort, SortGraph},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let audio_enabled = args.contains(&"audio".to_string());
    let (mut tx, mut handle) = if audio_enabled {
        let audio_device = AudioDevice::new();
        if let Ok(dev) = audio_device {
            let (tx, handle) = dev.play_audio_live();
            (Some(tx), Some(handle))
        } else {
            panic!();
            (None, None)
        }
    } else {
        (None, None)
    };

    let mut map = Map::new("dijkstra's Algorithm", &mut tx, &mut handle);
    map.generate();

    Map::reset_cursor();
    Map::clear_screen();
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

    let (mut tx, mut handle) = if audio_enabled {
        let audio_device = AudioDevice::new();
        let (tx, handle) = audio_device?.play_audio_live();
        (Some(tx), Some(handle))
    } else {
        (None, None)
    };

    let mut sort_graph = SortGraph::new("Quick Sort Algorithm", &mut tx, &mut handle);
    let mut quick_sort = QuickSort::new(&mut sort_graph);
    quick_sort.sort();
    // Stop the audio thread
    thread::sleep(Duration::from_millis(1000));

    sort_graph.reset();
    sort_graph.set_title("Merge Sort Algorithm");
    let mut merge_sort = MergeSort::new(&mut sort_graph);
    merge_sort.sort();

    thread::sleep(Duration::from_millis(1000));

    sort_graph.reset();
    sort_graph.set_title("HeapSort Algorithm");
    let mut heap = Heap::from_graph(&mut sort_graph);
    heap.heapsort();
    // Stop the audio thread
    sort_graph.stop_audio();
    sort_graph.join_audio();

    Ok(())
}
