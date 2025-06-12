use rand::Rng;
use std::{
    error::Error,
    str::FromStr,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, FromSample, Host, Sample, StreamConfig,
};
pub struct AudioDevice {
    host: Host,
    pub device: Device,
    pub config: StreamConfig,
}
pub enum AudioSignal {
    Single(f32),
    Chord(Vec<f32>),
    //SynthChord(Vec<f32>),
    //ChimeChord(Vec<f32>),
    //Silence,
}
impl AudioDevice {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();

        println!("Available output devices:");
        for device in host.output_devices().unwrap() {
            println!("Available device: {}", device.name().unwrap());
        }

        match host.default_output_device() {
            Some(device) => println!("Default device: {}", device.name().unwrap()),
            None => eprintln!("No default output device found."),
        }
        let device = host
            .default_output_device()
            .expect("failed to find output device");
        let mut config: StreamConfig = device.default_output_config()?.into();

        config.buffer_size = cpal::BufferSize::Fixed(128);
        println!("Default output config: {:?}", config);
        Ok(Self {
            host,
            device,
            config,
        })
    }

    pub fn play_audio(&self, music: Option<Notes>) -> Result<(), Box<dyn Error>> {
        let notes = if let Some(score) = music {
            score
        } else {
            Notes::new(self.config.sample_rate.0 as f32)?
        };
        play_notes(&self.device, &self.config.clone(), notes)
    }

    pub fn play_audio_live(&self) -> (Sender<AudioSignal>, JoinHandle<()>) {
        let (tx, rx): (Sender<AudioSignal>, Receiver<AudioSignal>) = mpsc::channel();
        let config = self.config.clone();
        let device = self.device.clone();

        let shared_frequencies = Arc::new(Mutex::new(Vec::new()));
        let shared_phases = Arc::new(Mutex::new(Vec::new()));

        let handle = {
            let shared_frequencies = Arc::clone(&shared_frequencies);
            let shared_phases = Arc::clone(&shared_phases);
            thread::spawn(move || {
                let sample_rate = config.sample_rate.0 as f32;
                let channels = config.channels as usize;
                let mut next_value = chord_with_phase(
                    sample_rate,
                    shared_frequencies.clone(),
                    shared_phases.clone(),
                );
                let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

                //                let mut phase_increments = Vec::new();

                let stream = device
                    .build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            while let Ok(signal) = rx.try_recv() {
                                match signal {
                                    AudioSignal::Single(frequency) => {
                                        let mut frequencies_lock =
                                            shared_frequencies.lock().unwrap();
                                        *frequencies_lock = vec![frequency];

                                        // Keep the same phases
                                        let mut phases_lock = shared_phases.lock().unwrap();
                                        if phases_lock.len() != 1 {
                                            *phases_lock = vec![0.0];
                                        }
                                    }
                                    AudioSignal::Chord(new_frequencies) => {
                                        let mut frequencies_lock =
                                            shared_frequencies.lock().unwrap();
                                        if new_frequencies.len() != frequencies_lock.len() {
                                            *frequencies_lock = new_frequencies.clone();
                                            let mut phases_lock = shared_phases.lock().unwrap();
                                            *phases_lock = vec![0.0; new_frequencies.len()];
                                        } else {
                                            *frequencies_lock = new_frequencies.clone();
                                        }
                                    }
                                }

                                next_value = chord_with_phase(
                                    sample_rate,
                                    shared_frequencies.clone(),
                                    shared_phases.clone(),
                                );
                            }

                            write_data_with_phases(
                                data,
                                channels,
                                &mut next_value,
                                shared_phases.clone(),
                            );
                        },
                        err_fn,
                        None,
                    )
                    .unwrap();

                stream.play().unwrap();

                // Keep thread alive to play audio
                thread::park();
            })
        };

        (tx, handle)
    }
}

fn chord_with_phase(
    sample_rate: f32,
    frequencies: Arc<Mutex<Vec<f32>>>,
    phases: Arc<Mutex<Vec<f32>>>,
) -> impl FnMut() -> f32 + Send + 'static {
    let mut clocks: Vec<f32> = phases.lock().unwrap().clone();
    let phase_increments: Vec<f32> = frequencies
        .lock()
        .unwrap()
        .iter()
        .map(|&f| f * 2.0 * std::f32::consts::PI / sample_rate)
        .collect();
    move || {
        let mut value = 0.0;
        for (i, &increment) in phase_increments.iter().enumerate() {
            let primary_wave = (clocks[i] * increment).sin();
            let harmonic_wave = (clocks[i] * increment * 2.0).sin() * 0.5;
            value += (primary_wave + harmonic_wave) / frequencies.lock().unwrap().len() as f32;
            clocks[i] = (clocks[i] + increment) % (2.0 * std::f32::consts::PI);
        }

        // Update the phases
        for (i, &clock) in clocks.iter().enumerate() {
            phases.lock().unwrap()[i] = clock;
        }

        value * 0.6
    }
}

fn write_data_with_phases<T>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> f32,
    _phases: Arc<Mutex<Vec<f32>>>,
) where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
pub fn play_notes(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    notes: Notes,
) -> Result<(), Box<dyn Error>> {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    for note in notes.notes.iter() {
        // Create the electric guitar wave generator for each note
        let mut next_value = generate_synthesizer_wave(sample_rate, note.frequency);

        // Build output stream
        let stream = device.build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value);
            },
            err_fn,
            None,
        )?;

        stream.play()?;
        std::thread::sleep(std::time::Duration::from_millis(note.time));
    }

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
pub struct Notes {
    sample_rate: f32,
    time: u64,
    notes: Vec<Note>,
}
impl Notes {
    pub fn new(sample_rate: f32) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            sample_rate,
            time: 0,
            notes: Vec::new(),
        })
    }
    pub fn chromatic_scale(sample_rate: f32) -> Result<Self, Box<dyn std::error::Error>> {
        let length = 150;
        let mut notes: Vec<Note> = Vec::new();
        let chromatic_scale = &["c", "d", "e", "f", "g", "a", "b"];

        for &i in chromatic_scale {
            let next_note = match i {
                "c" => Note::new(i, 3, length * 4),
                _ => Note::new(i, 3, length),
            };
            notes.push(next_note?);
            notes.push(Note::new(" ", 3, length / 3)?);
        }

        notes.push(Note::new("c", 4, length * 4)?);
        notes.push(Note::new(" ", 3, length)?);
        notes.push(Note::new("c", 4, length * 4)?);
        notes.push(Note::new(" ", 3, length / 3)?);
        for &i in chromatic_scale.iter().rev() {
            let next_note = match i {
                "c" => Note::new(i, 3, length * 4)?,
                _ => Note::new(i, 3, length)?,
            };
            notes.push(next_note);
            notes.push(Note::new(" ", 3, length / 3)?);
        }

        let total_time: u64 = notes.iter().fold(0, |acc, note| acc + note.time);

        Ok(Self {
            sample_rate,
            time: total_time,
            notes,
        })
    }
    pub fn pentatonic_blues(sample_rate: f32) -> Result<Self, Box<dyn std::error::Error>> {
        let up_notes: Vec<(&str, u32, u64)> = vec![
            ("c", 4, 500),
            (" ", 0, 50),
            ("eb", 4, 500),
            (" ", 0, 50),
            ("f", 4, 500),
            (" ", 0, 50),
            ("f#", 4, 500),
            (" ", 0, 50),
            ("g", 4, 500),
            (" ", 0, 50),
            ("bb", 4, 500),
            (" ", 0, 50),
            ("c", 5, 500),
            (" ", 0, 50),
            ("eb", 5, 500),
        ];
        let mut down_notes: Vec<(&str, u32, u64)> = up_notes.clone();
        let _ = down_notes.pop();
        down_notes.reverse();

        let music = [up_notes, down_notes].concat();
        let mut notes = Vec::with_capacity(music.len());
        let mut total_time: u64 = 0;
        for mus in music.iter() {
            let note = Note::new(mus.0, mus.1, mus.2)?;
            notes.push(note);
            total_time += mus.2;
            println!("note: {}, frequency: {}", note.name, note.frequency);
        }
        Ok(Self {
            sample_rate,
            time: total_time,
            notes,
        })
    }
    pub fn from(
        sample_rate: f32,
        music: Vec<(&str, u32, u64)>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut total_time = 0;
        let mut notes = Vec::new();
        for (input, octave, time) in music.iter() {
            total_time += time;
            notes.push(Note::new(input, *octave, *time)?);
        }
        Ok(Self {
            sample_rate,
            time: total_time,
            notes,
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoteName {
    C,
    CSharp,
    D,
    DSharp,
    DFlat,
    E,
    EFlat,
    F,
    FSharp,
    G,
    GSharp,
    GFlat,
    A,
    ASharp,
    AFlat,
    B,
    BFlat,
    Silence,
}
impl std::str::FromStr for NoteName {
    type Err = NoteError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_lowercase().as_str() {
            " " | "" => Ok(NoteName::Silence),
            "c" => Ok(NoteName::C),
            "csharp" | "c#" => Ok(NoteName::CSharp),
            "d" => Ok(NoteName::D),
            "dsharp" | "d#" => Ok(NoteName::DSharp),
            "dflat" | "db" => Ok(NoteName::DFlat),
            "e" => Ok(NoteName::E),
            "eflat" | "eb" => Ok(NoteName::EFlat),
            "f" => Ok(NoteName::F),
            "fsharp" | "f#" => Ok(NoteName::FSharp),
            "g" => Ok(NoteName::G),
            "gsharp" | "g#" => Ok(NoteName::GSharp),
            "gflat" | "gb" => Ok(NoteName::GFlat),
            "a" => Ok(NoteName::A),
            "asharp" | "a#" => Ok(NoteName::ASharp),
            "aflat" | "ab" => Ok(NoteName::AFlat),
            "b" => Ok(NoteName::B),
            "bflat" | "bb" => Ok(NoteName::BFlat),
            _ => Err(NoteError::InvalidNote(input.to_string())),
        }
    }
}
impl NoteName {
    pub fn to_note_index(&self) -> i32 {
        match self {
            NoteName::C => -9,
            NoteName::CSharp | NoteName::DFlat => -8,
            NoteName::D => -7,
            NoteName::DSharp | NoteName::EFlat => -6,
            NoteName::E => -5,
            NoteName::F => -4,
            NoteName::FSharp | NoteName::GFlat => -3,
            NoteName::G => -2,
            NoteName::GSharp | NoteName::AFlat => -1,
            NoteName::A | NoteName::Silence => 0,
            NoteName::ASharp | NoteName::BFlat => 1,
            NoteName::B => 2,
        }
    }
    pub fn to_str(&self) -> &str {
        match self {
            NoteName::Silence => "Silence",
            NoteName::C => "C",
            NoteName::CSharp => "CSharp",
            NoteName::DFlat => "DFlat",
            NoteName::D => "D",
            NoteName::DSharp => "DSharp",
            NoteName::EFlat => "EFlat",
            NoteName::E => "E",
            NoteName::F => "F",
            NoteName::FSharp => "FSharp",
            NoteName::GFlat => "GFlat",
            NoteName::G => "G",
            NoteName::GSharp => "GSharp",
            NoteName::AFlat => "AFlat",
            NoteName::A => "A",
            NoteName::ASharp => "ASharp",
            NoteName::BFlat => "BFlat",
            NoteName::B => "B",
        }
    }
}

impl std::fmt::Display for NoteName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = &self.to_str().to_string();
        write!(f, "{}", res)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum NoteError {
    InvalidNote(String),
}

impl std::fmt::Display for NoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNote(input) => write!(f, "{} is not a valid note.", input),
        }
    }
}

impl std::error::Error for NoteError {
    fn description(&self) -> &str {
        match *self {
            Self::InvalidNote(_) => "invalid note",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Note {
    pub name: NoteName,
    pub time: u64, // in milliseconds
    pub frequency: f32,
    pub octave: u32,
}

impl Note {
    pub fn new(note: &str, octave: u32, time: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let clean_note = if note.to_lowercase().starts_with('b') {
            let new_note = note.to_lowercase().replace('#', "sharp");
            if new_note.contains("flat") || new_note.contains("bb") {
                "bflat".to_string()
            } else {
                new_note
            }
        } else {
            note.to_lowercase()
                .replace('#', "sharp")
                .replace('b', "flat")
        };
        let name = NoteName::from_str(&clean_note)?;
        let base_frequency = Note::note_frequency(name);
        let frequency = Note::apply_octave(base_frequency, octave);

        Ok(Self {
            name,
            time,
            frequency,
            octave,
        })
    }

    fn note_frequency(note_name: NoteName) -> f32 {
        if note_name == NoteName::Silence {
            0.
        } else {
            let index = note_name.to_note_index();
            440.0 * 2.0_f32.powf((index - 9) as f32 / 12.0) // A4 is the reference note
        }
    }

    fn apply_octave(frequency: f32, octave: u32) -> f32 {
        if octave < 4 {
            frequency / 2_f32.powf(4.0 - octave as f32)
        } else {
            frequency * 2_f32.powf(octave as f32 - 4.0)
        }
    }
}
fn generate_sine_wave(sample_rate: f32, frequency: f32) -> impl FnMut() -> f32 {
    let mut sample_clock = 0f32;
    let phase_increment = frequency * 2.0 * std::f32::consts::PI / sample_rate;
    move || {
        let value = (sample_clock * phase_increment).sin();
        sample_clock = (sample_clock + 1.0) % sample_rate;
        value / 3.0
    }
}

fn generate_electric_guitar_wave(sample_rate: f32, frequency: f32) -> impl FnMut() -> f32 {
    let mut sample_clock = 0f32;
    let phase_increment = frequency * 2.0 * std::f32::consts::PI / sample_rate;
    move || {
        // Generate the sine wave
        let sine_wave = (sample_clock * phase_increment).sin();
        // Apply distortion
        let distorted_wave = if sine_wave >= 0.0 {
            sine_wave.powf(1.5)
        } else {
            -(-sine_wave).powf(1.5)
        };
        // Increment the sample clock
        sample_clock = (sample_clock + 1.0) % sample_rate;
        // Output
        distorted_wave / 3.0 // Reduce amplitude to avoid clipping
    }
}
fn generate_synthesizer_wave(sample_rate: f32, frequency: f32) -> impl FnMut() -> f32 + Send {
    let mut sample_clock = 0f32;
    let phase_increment = frequency / sample_rate;

    move || {
        // Generate the sawtooth wave
        let sawtooth_wave = 2.0 * (sample_clock % 1.0) - 1.0;

        // Increment the sample clock
        sample_clock = (sample_clock + phase_increment) % 1.0;

        // Apply a simple amplitude envelope
        let amplitude = 1.0 - (sample_clock % 0.2) * 5.0;
        sawtooth_wave * amplitude
    }
}
fn play_drum() {
    let sample_rate = 44100.0; // Example sample rate
    let mut next_value = generate_drum_sound(sample_rate);

    // Example audio output loop (for illustration only)
    for _ in 0..sample_rate as usize {
        let sample = next_value();
        // Send sample to audio output
        println!("{sample}");
    }
}

fn generate_drum_sound(sample_rate: f32) -> impl FnMut() -> f32 {
    let mut rng = rand::thread_rng();
    let noise_samples: Vec<f32> = (0..sample_rate as usize)
        .map(|_| rng.gen_range(-0.5..0.5))
        .collect();

    let mut index = 0;
    let decay_rate = 50.0;
    move || {
        // Get the current noise sample
        let white_noise = noise_samples[index];
        // Apply an exponential decay envelope
        let envelope = (-decay_rate * index as f32 / sample_rate).exp();
        // Increment the index
        index = (index + 1) % noise_samples.len();
        // Combine and return
        white_noise * envelope
    }
}
fn generate_chord_wave(sample_rate: f32, frequencies: Vec<f32>) -> impl FnMut() -> f32 + Send {
    let mut sample_clock = vec![0.0; frequencies.len()];
    let phase_increment: Vec<f32> = frequencies
        .iter()
        .map(|&f| f * 2.0 * std::f32::consts::PI / sample_rate)
        .collect();
    move || {
        let mut value = 0.0;
        for (i, increment) in phase_increment.iter().enumerate() {
            value += (sample_clock[i] * increment).sin() / 3.0;
            sample_clock[i] = (sample_clock[i] + 1.0) % sample_rate;
        }
        value
    }
}
fn silence_wave() -> impl FnMut() -> f32 + Send {
    move || 0.0
}
fn piano_chord(sample_rate: f32, frequencies: Vec<f32>) -> impl FnMut() -> f32 + Send {
    let mut clocks: Vec<f32> = vec![0.0; frequencies.len()];
    let phase_increments: Vec<f32> = frequencies
        .iter()
        .map(|&f| f * 2.0 * std::f32::consts::PI / sample_rate)
        .collect();
    move || {
        let mut value = 0.0;
        for (i, &increment) in phase_increments.iter().enumerate() {
            let primary_wave = (clocks[i] * increment).sin();
            let harmonic_wave = (clocks[i] * increment * 2.0).sin() * 0.5;
            value += (primary_wave + harmonic_wave) / 3.;
            clocks[i] = (clocks[i] + 1.0) % sample_rate;
        }
        value * 0.6 // Apply decay
    }
}
fn chime_chord(sample_rate: f32, frequencies: Vec<f32>) -> impl FnMut() -> f32 + Send {
    let mut clocks: Vec<f32> = vec![0.0; frequencies.len()];
    let phase_increments: Vec<f32> = frequencies
        .iter()
        .map(|&f| f * 2.0 * std::f32::consts::PI / sample_rate)
        .collect();
    move || {
        let mut value = 0.0;
        for (i, &increment) in phase_increments.iter().enumerate() {
            let primary_wave = (clocks[i] * increment).sin();
            let harmonic_wave = (clocks[i] * increment * 2.0).sin() * 0.5;
            value += (primary_wave + harmonic_wave) / 3.;
            clocks[i] = (clocks[i] + 1.0) % sample_rate;
        }
        value * 0.3 // Apply decay
    }
}
#[cfg(test)]
mod tests {

    fn note_frequency_success() {}
}
