use crate::audio::AudioSignal;
use colored::Colorize;
use rand::Rng;
use std::time::Duration;
use std::time::Instant;
use std::{
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};
pub struct SortGraph<'a, 'b> {
    pub title: String,
    pub values: Vec<i32>,
    pub max_height: i32,
    pub audio_sender: &'a mut Option<Sender<AudioSignal>>, // Audio sender for live updates
    pub audio_handle: &'b mut Option<JoinHandle<()>>,      // Audio thread handle
}

const WIDTH: i32 = 30;
const HEIGHT: i32 = 50;

impl<'a, 'b> SortGraph<'a, 'b> {
    /// Creates a new `SortGraph` with randomly generated values.
    pub fn new(
        title: &str,
        audio_sender: &'a mut Option<Sender<AudioSignal>>,
        audio_handle: &'b mut Option<JoinHandle<()>>,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let values = (0..=WIDTH)
            .map(|_| rng.gen_range(0..=HEIGHT))
            .collect();
        SortGraph {
            title: title.to_string(),
            values,
            max_height: HEIGHT,
            audio_sender,
            audio_handle,
        }
    }
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }
    pub fn update_audio(&self, frequency: f32) {
        if let Some(ref sender) = self.audio_sender {
            sender
                .send(AudioSignal::Single(frequency))
                .unwrap_or_default();
        }
    }

    pub fn stop_audio(&self) {
        if let Some(ref handle) = self.audio_handle {
            handle.thread().unpark();
        }
    }

    pub fn join_audio(&mut self) {
        if let Some(handle) = self.audio_handle.take() {
            handle.join().unwrap();
        }
    }
    fn send_swap_values(&self, from: i32, to: i32, duration: u64) {
        let max_frequency = 880.0; // Maximum frequency to represent max_height
        let half = duration / 2;
        let max_height = self.max_height as f32;

        let scale_frequency = |value: i32| -> f32 { max_frequency * (value as f32 / max_height) };
        let start_time = Instant::now();
        // Send frequencies to the audio handler one at a time
        if let Some(ref sender) = self.audio_sender {
            // Scale 'from' and 'to' values based on max_height
            let from_frequency = scale_frequency(from);
            let to_frequency = scale_frequency(to);
            sender
                .send(AudioSignal::Single(from_frequency))
                .unwrap_or_default();
            std::thread::sleep(Duration::from_millis(half));
            sender
                .send(AudioSignal::Single(to_frequency))
                .unwrap_or_default();
            std::thread::sleep(Duration::from_millis(half));
            sender.send(AudioSignal::Single(0.0)).unwrap_or_default();
        } else {
            // If no audio sender, just sleep for a bit
            std::thread::sleep(Duration::from_millis(duration));
        }
        let elapsed = start_time.elapsed().as_millis() as u64;
        if elapsed < duration {
            thread::sleep(Duration::from_millis(duration - elapsed));
        }
    }

    pub fn play_graph(&self, duration: u64) {
        let duration_per_value = duration / self.values.len() as u64;
        let max_frequency = 880.0; // Maximum frequency to represent max_height
        let max_height = self.max_height as f32;

        let scale_frequency = |value: i32| -> f32 { max_frequency * (value as f32 / max_height) };

        let start_time = Instant::now();
        // Send frequencies to the audio handler one at a time
        if let Some(ref sender) = self.audio_sender {
            for val in self.values.iter() {
                let frequency = scale_frequency(*val);
                sender
                    .send(AudioSignal::Single(frequency))
                    .unwrap_or_default();
                std::thread::sleep(Duration::from_millis(duration_per_value));
            }
        } else {
            // If no audio sender, just sleep for a bit
            std::thread::sleep(Duration::from_millis(duration));
        }
        let elapsed = start_time.elapsed().as_millis() as u64;
        if elapsed < duration {
            thread::sleep(Duration::from_millis(duration - elapsed));
        }
    }
    // Prints the map for visualization
    pub fn display_graph(&mut self) {
        let mut buffer = String::new();
        // Hide the cursor to avoid flickering
        buffer.push_str("\x1B[?25l");

        // Move cursor to the top-left
        buffer.push_str("\x1B[H");

        // Clear the screen from the cursor to the end of the screen
        buffer.push_str("\x1B[J");

        // Print the title and move to the next line
        buffer.push_str(&format!("{}\n", self.title));
        let height = self.max_height;
        for y in 0..=height {
            for (_, val) in self.values.iter().enumerate() {
                let y_pos = height - y;
                buffer += match y_pos {
                        pos if *val >= pos => "[x]",
                        _ => "   ",
                    };
            }
            buffer.push('\n'); // Add a new line at the end of each row
        }
        // Show the cursor again
        buffer.push_str("\x1B[?25h");

        // Print the entire buffer at once to the terminal
        let _start_time = Instant::now();
        println!("{}", buffer);
        self.play_graph(2000);
    }
    pub fn display_graph_with_highlights(
        &self,
        pivot_index: usize,
        low: i32,
        high: i32,
        swap: (usize, usize),
    ) {
        let height = self.max_height;
        let mut buffer = String::new();
        let swap_frequencies_from: i32 = self.values[swap.0];
        let swap_frequencies_to: i32 = self.values[swap.1];

        // Hide the cursor to avoid flickering
        // swap = (a, b), columns is swapping from index a to index b
        buffer.push_str("\x1B[?25l");

        // Move cursor to the top-left
        buffer.push_str("\x1B[H");

        // Print the title and move to the next line
        buffer.push_str(&format!("{}\n", self.title));
        for y in 0..=height {
            for (x, val) in self.values.iter().enumerate() {
                let symbol = if *val >= height - y { "[x]" } else { "   " };
                let styled_symbol = if x == pivot_index {
                    symbol.red().on_truecolor(128, 128, 128)
                } else if x as i32 >= low && x as i32 <= high {
                    // is in the partition
                    if swap.0 == x || swap.1 == x {
                        // column is in partition
                        symbol.bright_yellow().on_truecolor(128, 128, 128) // this column is a swap value
                    } else {
                        symbol.on_truecolor(128, 128, 128) // this column is not a swap value
                    }
                } else {
                    // column is not in the partition
                    symbol.white()
                };

                if self.max_height - y == self.values[pivot_index] {
                    // blue bar across graph equalling the pivot column's height
                    buffer += &format!("{}", styled_symbol.on_blue());
                } else {
                    buffer += &format!("{}", styled_symbol);
                }
            }
            buffer.push('\n');
        }
        // Show the cursor again
        buffer.push_str("\x1B[?25h");

        // Print the entire buffer at once to the terminal
        print!("{}", buffer);
        self.send_swap_values(swap_frequencies_from, swap_frequencies_to, 25)
    }
    pub fn display_graph_move_highlights(
        &self,
        start: usize,
        middle: usize,
        end: usize,
        swap: Option<(usize, usize)>,
    ) {
        let height = self.max_height;
        let mut buffer = String::new();
        let mut swap_frequencies_from: i32 = 0;
        let mut swap_frequencies_to: i32 = 0;
        if let Some((from, to)) = swap {
            swap_frequencies_from = from as i32;
            swap_frequencies_to = to as i32;
        }
        // Hide the cursor to avoid flickering
        // swap = (a, b), columns is swapping from index a to index b
        buffer.push_str("\x1B[?25l");

        // Move cursor to the top-left
        buffer.push_str("\x1B[H");

        // Print the title and move to the next line
        buffer.push_str(&format!("{}\n", self.title));
        for y in 0..=height {
            for (x, val) in self.values.iter().enumerate() {
                let symbol = if *val >= height - y { "[x]" } else { "   " };
                let styled_symbol = match swap {
                    Some((from, to)) if from == x && to != x => {
                        symbol.bright_red() // this column is a swap value
                    }
                    Some((_from, to)) if to == x => {
                        symbol.green() // this column is a swap value
                    }
                    _ => symbol.clear(),
                };
                if x >= start && x < middle {
                    // highlight left side
                    buffer += &format!("{}", styled_symbol.on_truecolor(140, 140, 140));
                } else if x >= middle && x < end {
                    // highlight right side
                    buffer += &format!("{}", styled_symbol.on_truecolor(180, 180, 180));
                } else {
                    buffer += &format!("{}", styled_symbol);
                }
            }
            buffer.push('\n');
        }
        // Show the cursor again
        buffer.push_str("\x1B[?25h");

        // Print the entire buffer at once to the terminal
        print!("{}", buffer);
        self.send_swap_values(swap_frequencies_from, swap_frequencies_to, 50)
    }
    pub fn display_simple_swap_graph(&self, swap: Option<(usize, usize)>) {
        let height = self.max_height;
        let mut buffer = String::new();

        let mut swap_frequencies_from: i32 = 0;
        let mut swap_frequencies_to: i32 = 0;
        if let Some((from, to)) = swap {
            swap_frequencies_from = from as i32;
            swap_frequencies_to = to as i32;
        }
        // Hide the cursor to avoid flickering
        // swap = (a, b), columns is swapping from index a to index b
        buffer.push_str("\x1B[?25l");

        // Move cursor to the top-left
        buffer.push_str("\x1B[H");

        // Print the title and move to the next line
        buffer.push_str(&format!("{}\n", self.title));
        for y in 0..=height {
            for (x, val) in self.values.iter().enumerate() {
                let symbol = if *val >= height - y { "[x]" } else { "   " };
                let styled_symbol = match swap {
                    Some((from, to)) if from == x && to != x => {
                        symbol.bright_red() // this column is a swap value
                    }
                    Some((_from, to)) if to == x => {
                        symbol.green() // this column is a swap value
                    }
                    _ => symbol.clear(),
                };

                buffer += &format!("{}", styled_symbol);
            }
            buffer.push('\n');
        }
        // Show the cursor again
        buffer.push_str("\x1B[?25h");

        // Print the entire buffer at once to the terminal
        print!("{}", buffer);
        self.send_swap_values(swap_frequencies_from, swap_frequencies_to, 25);
    }
}
