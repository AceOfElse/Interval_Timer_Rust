use eframe::egui;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

#[derive(Debug)]
enum TimerState {
    Idle,
    Workout,
    Rest,
}

struct WorkoutTimer {
    workout_duration: u64,
    rest_duration: u64,
    rounds: u32,
    current_round: u32,
    remaining_time: u64,
    start_time: Option<Instant>,
    state: TimerState,
    sound_sink: Option<Sink>,
}

impl Default for WorkoutTimer {
    fn default() -> Self {
        Self {
            workout_duration: 60,
            rest_duration: 45,
            rounds: 10,
            current_round: 0,
            remaining_time: 0,
            start_time: None,
            state: TimerState::Idle,
            sound_sink: None,
        }
    }
}

const SUCCESS_AUDIO: &[u8] = include_bytes!("../success.mp3");

impl WorkoutTimer {
    fn play_sound(&mut self) {
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            let sink = Sink::try_new(&stream_handle).unwrap();

            let cursor = std::io::Cursor::new(SUCCESS_AUDIO);
            let source = Decoder::new(cursor).unwrap();
            sink.append(source);
            self.sound_sink = Some(sink);
            // Keep the stream alive
            std::mem::forget(stream);
        }
    }

    fn update(&mut self) {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs();
            let total_duration = match self.state {
                TimerState::Workout => self.workout_duration,
                TimerState::Rest => self.rest_duration,
                TimerState::Idle => 0,
            };
            self.remaining_time = total_duration.saturating_sub(elapsed);
            match self.state {
                TimerState::Workout if elapsed >= self.workout_duration => {
                    self.start_time = Some(Instant::now());
                    self.state = TimerState::Rest;
                    self.play_sound();
                }
                TimerState::Rest if elapsed >= self.rest_duration => {
                    if self.current_round + 1 < self.rounds {
                        self.current_round += 1;
                        self.start_time = Some(Instant::now());
                        self.state = TimerState::Workout;
                        self.play_sound();
                    } else {
                        self.state = TimerState::Idle;
                        self.start_time = None;
                    }
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for WorkoutTimer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workout Interval Timer");
            ui.add(egui::Slider::new(&mut self.workout_duration, 5..=180).text("Workout (sec)"));
            ui.add(egui::Slider::new(&mut self.rest_duration, 5..=90).text("Rest (sec)"));
            ui.add(egui::Slider::new(&mut self.rounds, 1..=50).text("Rounds"));

            if ui.button("Start").clicked() {
                self.current_round = 0;
                self.start_time = Some(Instant::now());
                self.state = TimerState::Workout;
                self.play_sound();
            }

            ui.label(format!("Round: {}/{}", self.current_round + 1, self.rounds));
            ui.label(format!("State: {:?}", self.state));

            // Add countdown timer
            ui.label(format!("Time remaining: {:02}:{:02}", self.remaining_time / 60, self.remaining_time % 60));

            // Add progress bar
            let progress = match self.state {
                TimerState::Workout => 1.0 - (self.remaining_time as f32 / self.workout_duration as f32),
                TimerState::Rest => 1.0 - (self.remaining_time as f32 / self.rest_duration as f32),
                TimerState::Idle => 0.0,
            };
            ui.add(egui::ProgressBar::new(progress).show_percentage());
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Workout Timer", options, Box::new(|_cc| Ok(Box::new(WorkoutTimer::default()))))
}
