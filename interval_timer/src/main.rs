#![windows_subsystem = "windows"]

use eframe::egui;
use rodio::{Decoder, OutputStream, Sink};
use std::time::{Duration, Instant};

#[derive(Debug)]
enum TimerState {
    Idle,
    Workout,
    Rest,
    Paused,
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
    _stream: Option<OutputStream>,
}

impl Default for WorkoutTimer {
    fn default() -> Self {
        let stream = OutputStream::try_default().ok().map(|(s, _)| s);

        Self {
            workout_duration: 60,
            rest_duration: 45,
            rounds: 10,
            current_round: 0,
            remaining_time: 0,
            start_time: None,
            state: TimerState::Idle,
            sound_sink: None,
            _stream: stream,
        }
    }
}

const WORK_FINISH_AUDIO: &[u8] = include_bytes!("../work_finish.mp3");  
const REST_FINISH_AUDIO: &[u8] = include_bytes!("../rest_finish.mp3");

impl WorkoutTimer {
    fn play_sound(&mut self, is_work: bool) {
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            let sink = Sink::try_new(&stream_handle).unwrap();

            let cursor = std::io::Cursor::new(if is_work {
                WORK_FINISH_AUDIO
            } else {
                REST_FINISH_AUDIO
            });

            let source = Decoder::new(cursor).unwrap();
            sink.append(source);
            self.sound_sink = Some(sink);
            self._stream = Some(stream);
        }
    }

    fn update(&mut self) {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs();
            let total_duration = match self.state {
                TimerState::Workout => self.workout_duration,
                TimerState::Rest => self.rest_duration,
                TimerState::Idle => 0,
                TimerState::Paused => return,
            };
            self.remaining_time = total_duration.saturating_sub(elapsed);

            match self.state {
                TimerState::Workout if elapsed >= self.workout_duration => {
                    self.start_time = Some(Instant::now());
                    self.state = TimerState::Rest;
                    self.play_sound(false);
                }
                TimerState::Rest if elapsed >= self.rest_duration => {
                    if self.current_round + 1 < self.rounds {
                        self.current_round += 1;
                        self.start_time = Some(Instant::now());
                        self.state = TimerState::Workout;
                        self.play_sound(true);
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

        // Define custom text styles
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(24.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(18.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Button, egui::FontId::new(30.0, egui::FontFamily::Proportional)),
        ]
        .into();
        
        // Adjust sizes for sliders and progress bars
        style.spacing.slider_width = 240.0; // Increase slider width
        style.spacing.item_spacing.y = 10.0; // Increase vertical spacing between items
        style.spacing.interact_size.y = 30.0; // Increase height of interactive elements (including sliders)

        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workout Interval Timer");

            let slider_width = ui.available_width();
            ui.add_sized(
                [slider_width, 20.0], 
                egui::Slider::new(&mut self.workout_duration, 5..=180).text("Workout (sec)")
            );
            ui.add_sized(
                [slider_width, 20.0], 
                egui::Slider::new(&mut self.rest_duration, 5..=90).text("Rest (sec)")
            );
            ui.add_sized(
                [slider_width, 20.0], 
                egui::Slider::new(&mut self.rounds, 1..=50).text("Rounds")
            );

            match self.state {
                TimerState::Idle => {
                    ui.horizontal(|ui| {
                        if ui.button("Start").clicked() {
                            self.current_round = 0;
                            self.start_time = Some(Instant::now());
                            self.state = TimerState::Workout;
                        }
                    });
                }
                TimerState::Workout | TimerState::Rest => {
                    ui.horizontal(|ui| {
                        if ui.button("Pause").clicked() {
                            self.state = TimerState::Paused;
                        }
                        if ui.button("Stop").clicked() {
                            self.current_round = 0;
                            self.remaining_time = 0;
                            self.state = TimerState::Idle;
                            self.start_time = None;
                        }
                    });
                }
                TimerState::Paused => {
                    ui.horizontal(|ui| {
                        if ui.button("Resume").clicked() {
                            self.start_time = Some(Instant::now() - Duration::from_secs(self.workout_duration - self.remaining_time));
                            self.state = if self.remaining_time > 0 {
                                TimerState::Workout
                            } else {
                                TimerState::Rest
                            };
                        }
                        if ui.button("Stop").clicked() {
                            self.state = TimerState::Idle;
                            self.start_time = None;
                            self.remaining_time = 0;
                            self.current_round = 0;
                        }
                    });
                }
            }

            ui.label(format!("Round: {}/{}", self.current_round + 1, self.rounds));
            ui.label(format!("State: {:?}", self.state));

            // Add countdown timer
            ui.label(format!("Time remaining: {:02}:{:02}", self.remaining_time / 60, self.remaining_time % 60));

            // Add progress bar
            let progress = match self.state {
                TimerState::Workout => 1.0 - (self.remaining_time as f32 / self.workout_duration as f32),
                TimerState::Rest => 1.0 - (self.remaining_time as f32 / self.rest_duration as f32),
                TimerState::Idle | TimerState::Paused => 0.0,
            };

            let progress_bar = egui::ProgressBar::new(progress)
            .show_percentage()
            .fill(match self.state {
                TimerState::Workout => egui::Color32::from_rgb(0x3B, 0xA4, 0x58), // Green
                TimerState::Rest => egui::Color32::from_rgb(0x38, 0x77, 0xA2), // Blue
                TimerState::Idle | TimerState::Paused => egui::Color32::from_rgb(0x3D, 0x3D, 0x3D), // Gray
            });
            
            ui.add(progress_bar);
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([450.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native("Workout Timer", options, Box::new(|_cc| Ok(Box::new(WorkoutTimer::default()))))
}
