use crossterm::{
    cursor::{Hide, MoveTo, RestorePosition, SavePosition, Show},
    style::Print,
    ExecutableCommand,
};

use std::{collections::HashSet, collections::VecDeque};
use std::{io::Write, thread::sleep};
use std::{
    sync::mpsc::{Sender},
    thread::JoinHandle,
};
use std::{thread, time::Duration}; // Crossterm handles cursor movement and more

use crate::audio::AudioSignal;
use colored::{ColoredString, Colorize};
use rand::{seq::SliceRandom, thread_rng, Rng};

const WIDTH: usize = 30;
const HEIGHT: usize = 30;
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

#[derive(Debug)]
pub struct Map<'a, 'b> {
    pub graph_title: String,
    pub graph: Vec<Vec<u8>>, // Using a Vec<Vec<u8>> for simplicity, 0 is non-traversable, 1 is traversable
    pub width: usize,
    pub height: usize,
    pub current: (usize, usize),
    pub visited: Vec<(usize, usize)>,
    pub start: (usize, usize), // Coordinates for the start square
    pub end: (usize, usize),   // Coordinates for the end square
    pub audio_sender: &'a mut Option<Sender<AudioSignal>>, // Audio sender for live updates
    pub audio_handle: &'b mut Option<JoinHandle<()>>, // Audio thread handle
}

impl<'a, 'b> Map<'a, 'b> {
    pub fn new(
        title: &str,
        audio_sender: &'a mut Option<Sender<AudioSignal>>,
        audio_handle: &'b mut Option<JoinHandle<()>>,
    ) -> Self {
        let mut rng = thread_rng(); // Get a random number generator

        // Define the map dimensions
        let (width, height) = (WIDTH, HEIGHT);

        // Define the start position
        let start = (0, 0);

        // Randomly choose to place 'end' based on width or height
        let (end_x, end_y) = if rng.gen::<bool>() {
            // Randomly choose 'end' beyond half the width, ensure it is not in the first row
            (rng.gen_range(width / 2..width), rng.gen_range(1..height))
        } else {
            // Randomly choose 'end' beyond half the height, ensure it is not in the first column
            (rng.gen_range(1..width), rng.gen_range(height / 2..height))
        };
        Self::clear_screen();
        Self::reset_cursor();
        // Create the map
        Map {
            graph_title: title.to_string(),
            graph: vec![vec![0; width]; height], // Initialize all cells as non-traversable
            width,
            height,
            current: start,       // Set current position to start
            visited: vec![start], // Start has been visited
            start,
            end: (end_x, end_y), // Set the random 'end' position
            audio_sender,
            audio_handle,
        }
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
    pub fn play_visited(&self, distances: Vec<usize>, duration: u64) {
        if let Some(sender) = &self.audio_sender {
            for distance in distances.iter() {
                let freq = 440.0 + (440.0 * (1.0 - *distance as f32 / self.height as f32));
                sender.send(AudioSignal::Single(freq)).unwrap_or_default();
                let duration = duration / distances.len() as u64;
                thread::sleep(Duration::from_millis(duration));
                sender.send(AudioSignal::Single(0.0)).unwrap_or_default();
            }
        }
    }
    pub fn play_distance(&self, distance: u32, position: (usize, usize)) {
        let (x, y) = (position.0 as f32, position.1 as f32);
        if let Some(sender) = &self.audio_sender {
            let freq =
                440.0 + (440.0 * (1.0 - distance as f32 / (self.height * self.width) as f32));
            let freq_x = 440.0 + (440.0 * (1.0 - x as f32 / self.width as f32));
            let freq_y = 440.0 + (440.0 * (1.0 - y as f32 / self.height as f32));
            sender
                .send(AudioSignal::Chord(vec![freq, freq_y, freq_x]))
                .unwrap_or_default();
            thread::sleep(Duration::from_millis(30));
        }
    }
    pub fn play_end_location(&self) {
        if let Some(sender) = &self.audio_sender {
            let distance = 0.;
            let freq = 440.0 + (440.0 * (1.0 - distance as f32 / (self.end.1 * self.end.0) as f32));
            let freq_x = 440.0 + (440.0 * (1.0 - self.end.0 as f32 / self.width as f32));
            let freq_y = 440.0 + (440.0 * (1.0 - self.end.1 as f32 / self.height as f32));
            sender
                .send(AudioSignal::Chord(vec![freq, freq_y, freq_x]))
                .unwrap_or_default();
            thread::sleep(Duration::from_millis(1000));
        }
    }

    pub fn reset(&mut self, title: &str) {
        self.visited.clear();
        self.graph_title = title.to_string();
        self.display();
    }
    pub fn generate(&mut self) {
        loop {
            // Attempt to generate the map
            self.attempt_generate();

            // Check if there's a path from start to end
            if self.is_path_from_start_to_end() {
                break; // Break the loop if a valid path exists
            }
        }
    }
    // Generates a maze-like map
    pub fn attempt_generate(&mut self) {
        let (_width, _height) = (self.width, self.height);
        let mut rng = rand::thread_rng();

        // Initialize all cells as walls
        for row in self.graph.iter_mut() {
            for square in row.iter_mut() {
                *square = 0;
            }
        }

        // Starting cell
        let mut stack = vec![(self.start.0 as isize, self.start.1 as isize)];
        self.graph[self.start.1][self.start.0] = 1;

        // Directions for moving (left, right, up, down)
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        while let Some((cx, cy)) = stack.pop() {
            // Shuffle directions for randomness
            let mut shuffled_directions = directions;
            shuffled_directions.shuffle(&mut rng);

            for &(dx, dy) in &shuffled_directions {
                let nx = cx + 2 * dx;
                let ny = cy + 2 * dy;

                if self.is_valid(nx, ny) && self.graph[ny as usize][nx as usize] == 0 {
                    // Make current and next cell traversable
                    self.graph[(cy + dy) as usize][(cx + dx) as usize] = 1;
                    self.graph[ny as usize][nx as usize] = 1;

                    // Add next cell to stack
                    stack.push((nx, ny));
                }
            }
        }

        // Ensure start and end points are traversable
        self.graph[self.start.1][self.start.0] = 1;
        self.graph[self.end.1][self.end.0] = 1;
        self.current = self.start;
        self.visited.clear();
        self.visited.push(self.start);
    }

    // Checks if a cell is within bounds of the map
    fn is_valid(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize
    }
    fn is_path_from_start_to_end(&self) -> bool {
        let mut visited = vec![vec![false; self.width]; self.height];
        let mut queue = VecDeque::new();
        queue.push_back(self.start);
        visited[self.start.1][self.start.0] = true;

        while let Some((x, y)) = queue.pop_front() {
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = (x as isize + dx) as usize;
                let ny = (y as isize + dy) as usize;
                if nx < self.width
                    && ny < self.height
                    && self.graph[ny][nx] == 1
                    && !visited[ny][nx]
                {
                    if (nx, ny) == self.end {
                        return true;
                    }
                    queue.push_back((nx, ny));
                    visited[ny][nx] = true;
                }
            }
        }

        false
    }
    pub fn clear_screen() {
        print!("\x1B[2J"); // Clears the entire screen
    }

    pub fn reset_cursor() {
        print!("\x1B[H"); // Moves the cursor to the top-left corner
    }
    // Prints the map for visualization
    pub fn display(&self) {
        let mut buffer = String::new();
        // Hide the cursor to avoid flickering
        buffer.push_str("\x1B[?25l");

        // Move cursor to the top-left
        buffer.push_str("\x1B[H");

        // Clear the screen from the cursor to the end of the screen
        buffer.push_str("\x1B[J");

        // Print the title and move to the next line
        buffer.push_str(&format!("{}\n", self.graph_title));
        let max_distance = self.width + self.height; // Simplified max distance
        for (i, row) in self.graph.iter().enumerate() {
            for (j, square) in row.iter().enumerate() {
                let distance = Self::manhattan_distance(self.current.0, self.current.1, j, i);
                let colored_dot = Self::distance_to_color(distance, max_distance);

                buffer += &format!(
                    "{} ",
                    match (j, i) {
                        _ if (j, i) == self.current => "C".green(), // Current point
                        _ if (j, i) == self.start => "S".blue(),    // Start point
                        _ if (j, i) == self.end => "E".green(),     // End point
                        _ if self.visited.contains(&(j, i)) => colored_dot,
                        _ if *square == 1 => ".".white(), // Traversable
                        _ => "#".bright_black(),          // Non-traversable
                    }
                );
            }
            buffer.push('\n'); // Add a new line at the end of each row
        }
        // Show the cursor again
        buffer.push_str("\x1B[?25h");

        // Print the entire buffer at once to the terminal
        println!("{}", buffer);
        thread::sleep(Duration::from_millis(2000))
    }

    pub fn display_visited(&self, path: &HashSet<(usize, usize)>) {
        let mut buffer = String::new();
        // Hide the cursor to avoid flickering
        buffer.push_str("\x1B[?25l");

        // Move cursor to the top-left
        buffer.push_str("\x1B[H");

        // Print the title and move to the next line
        buffer.push_str(&format!("{}\n", self.graph_title));
        let max_distance = self.width + self.height;
        for (y, row) in self.graph.iter().enumerate() {
            for (x, square) in row.iter().enumerate() {
                let distance = Self::manhattan_distance(self.current.0, self.current.1, x, y);
                let colored_dot = Self::distance_to_color(distance, max_distance);

                buffer += &format!(
                    "{} ",
                    match (x, y) {
                        _ if (x, y) == self.start => "S".magenta(),
                        _ if (x, y) == self.end => "E".green(),
                        _ if path.contains(&(x, y)) => {
                            colored_dot
                        }
                        _ if *square == 1 => ".".white(),
                        _ => "#".bright_black(),
                    }
                );
            }
            buffer.push('\n'); // Add a new line at the end of each row
        }
        // Show the cursor again
        buffer.push_str("\x1B[?25h");

        // Print the entire buffer at once to the terminal
        print!("{}", buffer);
        std::thread::sleep(Duration::from_millis(5));
    }
    pub fn display_path(&self, path: &Vec<(usize, usize)>) {
        let mut stdout = std::io::stdout();
        // Save the current cursor position
        stdout.execute(SavePosition).unwrap();
        // Hide the cursor to avoid flickering during updates
        stdout.execute(Hide).unwrap();

        // Initially display the entire grid
        self.display();

        // Render the path with a visible delay between updates
        for &(x, y) in path {
            // Move cursor to the correct position for each cell in the path
            stdout.execute(MoveTo(x as u16 * 2, y as u16 + 1)).unwrap(); // +1 to account for the title line if any

            // Determine and print the content for each path node
            let content = match (x, y) {
                _ if (x, y) == self.start => "S".cyan(),
                _ if (x, y) == self.end => "E".cyan(),
                _ => {
                    let dist = Self::manhattan_distance(x, y, self.end.0, self.end.1) as u32;
                    self.play_distance(dist, (x, y));
                    "•".bright_green()
                } // Visited nodes in the path
            };

            // Print the cell content
            stdout.execute(Print(content)).unwrap();

            // Sleep to visually demonstrate the update
            std::thread::sleep(Duration::from_millis(20)); // Adjust sleep duration as needed

            // Flush stdout to ensure the update is shown immediately
            stdout.flush().unwrap();
        }

        // Move the cursor below the last row of the grid
        stdout.execute(MoveTo(0, self.height as u16 + 10)).unwrap();

        // Show the cursor again
        stdout.execute(Show).unwrap();

        stdout.execute(RestorePosition).unwrap();
        // Flush to ensure all commands take effect
        stdout.flush().unwrap();
        self.update_audio(0.0);
        sleep(Duration::from_millis(1000));
    }
    pub fn is_traversable(&self, x: usize, y: usize) -> bool {
        self.graph[y][x] == 1
    }
    pub fn get_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = (x as isize + dx) as usize;
            let ny = (y as isize + dy) as usize;
            if nx < self.width && ny < self.height && self.is_traversable(nx as usize, ny as usize)
            {
                neighbors.push((nx, ny));
            }
        }
        neighbors
    }
    // Function to calculate Manhattan distance
    pub fn manhattan_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
        ((x1 as isize - x2 as isize).abs() + (y1 as isize - y2 as isize).abs()) as usize
    }
    // Function to calculate color based on distance
    fn distance_to_color(distance: usize, max_distance: usize) -> ColoredString {
        let intensity = 255 - (255 * distance / max_distance).min(255) as u8;
        "•".truecolor(intensity, 0, 255 - intensity) // Red to blue gradient
    }

    pub fn cost(&self, _from: (usize, usize), _to: (usize, usize)) -> u32 {
        1 // Uniform cost; adjust as necessary for different terrains or obstacles
    }
    pub fn move_back(&mut self) {
        if self.visited.is_empty() {
            return;
        }
        let last_position = self.visited.pop().unwrap();
        self.current = last_position;
    }
    pub fn move_direction(&mut self, direction: &Direction) {
        match direction {
            Direction::Left => self.move_left(),
            Direction::Up => self.move_up(),
            Direction::Right => self.move_right(),
            Direction::Down => self.move_down(),
        }
    }
    // Movement methods
    fn move_up(&mut self) {
        if self.current.0 <= 0 {
            return;
        }
        let next_location = (self.current.0 - 1, self.current.1);
        if next_location == self.start {
            return;
        }
        if self.current.0 > 0 && self.is_traversable(self.current.0 - 1, self.current.1) {
            self.current.0 -= 1;
            self.visited.push(self.current);
        }
    }

    fn move_down(&mut self) {
        if self.current.0 <= self.height - 1 {
            return;
        }
        let next_location = (self.current.0 + 1, self.current.1);
        if next_location == self.start {
            return;
        }

        if self.current.0 < self.height - 1
            && self.is_traversable(self.current.0 + 1, self.current.1)
        {
            self.current.0 += 1;
            self.visited.push(self.current);
        }
    }

    fn move_left(&mut self) {
        if self.current.1 <= 0 {
            return;
        }
        let next_location = (self.current.0, self.current.1 - 1);
        if next_location == self.start {
            return;
        }

        if self.current.1 > 0 && self.is_traversable(self.current.0, self.current.1 - 1) {
            self.current.1 -= 1;
            self.visited.push(self.current);
        }
    }

    fn move_right(&mut self) {
        if self.current.1 >= self.width - 1 {
            return;
        }
        let next_location = (self.current.0, self.current.1 + 1);
        if next_location == self.start {
            return;
        }

        if self.current.1 < self.width - 1
            && self.is_traversable(self.current.0, self.current.1 + 1)
        {
            self.current.1 += 1;
            self.visited.push(self.current);
        }
    }
}
