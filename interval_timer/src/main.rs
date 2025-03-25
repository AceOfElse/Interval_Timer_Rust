#![windows_subsystem = "windows"]
#[global_allocator]
static ALLOC: std::alloc::System = std::alloc::System;

use eframe::egui;
use rodio::{Decoder, OutputStream, Sink};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use std::fs;

const FANFARE_STAR: &[u8] = include_bytes!("../star.png");
const WORK_FINISH_AUDIO: &[u8] = include_bytes!("../work_finish.mp3");
const REST_FINISH_AUDIO: &[u8] = include_bytes!("../rest_finish.mp3");
const COMPLETE_FINISH_AUDIO: &[u8] = include_bytes!("../complete_finish.mp3");

#[derive(Debug, Clone, Copy)]
enum TimerState {
    Idle,
    LeadUp,
    Workout,
    Rest,
    PausedWorkout,
    PausedRest,
    PausedLeadUp,
}

#[derive(Serialize, Deserialize)]
struct Settings {
    workout_duration: u64,
    rest_duration: u64,
    rounds: u32,
    lead_up_duration: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            workout_duration: 60,
            rest_duration: 45,
            rounds: 10,
            lead_up_duration: 5,
        }
    }
}

impl Settings {
    fn load_from_file() -> Self {
        if let Ok(data) = fs::read_to_string("settings.json") {
            serde_json::from_str(&data).unwrap_or_else(|_| {
                let default_settings = Self::default();
                default_settings.save_to_file(); // Save defaults if file is corrupted
                default_settings
            })
        } else {
            let default_settings = Self::default();
            default_settings.save_to_file(); // Save defaults if file doesn't exist
            default_settings
        }
    }

    fn save_to_file(&self) {
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write("settings.json", data);
        }
    }
}

struct WorkoutTimer {
    workout_duration: u64,
    rest_duration: u64,
    rounds: u32,
    current_round: u32,
    remaining_time: u64,
    lead_up_duration: u32,
    start_time: Option<Instant>,
    fanfare_start_time: Option<Instant>,
    state: TimerState,
    sound_sink: Option<Sink>,
    _stream: Option<OutputStream>,
}

impl Default for WorkoutTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkoutTimer {
    fn new() -> Self {
        let settings = Settings::load_from_file();
        let stream = OutputStream::try_default().ok().map(|(s, _)| s);

        Self {
            workout_duration: settings.workout_duration,
            rest_duration: settings.rest_duration,
            rounds: settings.rounds,
            lead_up_duration: settings.lead_up_duration,
            current_round: 0,
            remaining_time: 0,
            start_time: None,
            state: TimerState::Idle,
            sound_sink: None,
            _stream: stream,
            fanfare_start_time: None,
        }
    }

    fn save_settings(&self) {
        let settings = Settings {
            workout_duration: self.workout_duration,
            rest_duration: self.rest_duration,
            rounds: self.rounds,
            lead_up_duration: self.lead_up_duration,
        };
        settings.save_to_file();
    }

    fn play_sound(&mut self, is_work: bool, is_complete: bool) {
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            let sink = Sink::try_new(&stream_handle).unwrap();

            let audio_data = if is_complete {
                COMPLETE_FINISH_AUDIO
            } else if is_work {
                WORK_FINISH_AUDIO
            } else {
                REST_FINISH_AUDIO
            };

            let cursor = std::io::Cursor::new(audio_data);
            let source = Decoder::new(cursor).unwrap();
            sink.append(source);
            self.sound_sink = Some(sink);
            self._stream = Some(stream);
        }
    }

    fn trigger_visual_fanfare(&mut self) {
        self.fanfare_start_time = Some(Instant::now());
    }

    fn update(&mut self) {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs();

            match self.state {
                TimerState::LeadUp => {
                    // Handle lead-up phase
                    self.remaining_time = self.lead_up_duration as u64 - elapsed;
                    if elapsed >= self.lead_up_duration as u64 {
                        self.state = TimerState::Workout;
                        self.start_time = Some(Instant::now());
                        self.remaining_time = self.workout_duration;
                    }
                }
                TimerState::Workout => {
                    self.remaining_time = self.workout_duration.saturating_sub(elapsed);
                    if elapsed >= self.workout_duration {
                        self.state = TimerState::Rest;
                        self.start_time = Some(Instant::now());
                        self.remaining_time = self.rest_duration;
                        self.play_sound(true, false);
                    }
                }
                TimerState::Rest => {
                    self.remaining_time = self.rest_duration.saturating_sub(elapsed);
                    if elapsed >= self.rest_duration {
                        if self.current_round + 1 < self.rounds {
                            self.current_round += 1;
                            self.state = TimerState::Workout;
                            self.start_time = Some(Instant::now());
                            self.remaining_time = self.workout_duration;
                            self.play_sound(false, false);
                        } else {
                            self.state = TimerState::Idle;
                            self.start_time = None;
                            self.current_round = 0;
                            self.play_sound(false, true);
                            self.trigger_visual_fanfare();
                        }
                    }
                }
                TimerState::PausedLeadUp | TimerState::PausedWorkout | TimerState::PausedRest => {
                    // Do nothing while paused
                }
                TimerState::Idle => {
                    // Do nothing while idle
                }
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

            // Check if fanfare is active
            if let Some(start_time) = self.fanfare_start_time {
                let elapsed = start_time.elapsed().as_secs_f32();
                if elapsed < 2.0 {
                    // Display fanfare message
                    ui.vertical(|ui| {
                        ui.label(format!("Congratulations, you completed {} rounds!", self.rounds));

                        // Display three spinning stars
                        let angle = elapsed * 2.0 * std::f32::consts::PI; // Rotate 360 degrees per second
                        let image = {
                            let decoder = image::load_from_memory(FANFARE_STAR).unwrap();
                            let rgba = decoder.to_rgba8();
                            let size = [rgba.width() as usize, rgba.height() as usize];
                            egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice())
                        };
                        let texture = ctx.load_texture("star", image, egui::TextureOptions::default());

                        ui.horizontal(|ui| {
                            for _ in 0..3 {
                                ui.add(egui::Image::new(&texture).rotate(angle, egui::Vec2::new(0.5, 0.5)));
                            }
                        });
                    });
                } else {
                    self.fanfare_start_time = None; // End fanfare
                }
            }

            let slider_width = ui.available_width();

            let mut changed = false;

            changed |= ui.add_sized(
                [slider_width, 20.0],
                egui::Slider::new(&mut self.workout_duration, 2..=180)
                    .text("Workout (sec)"),
            ).changed();

            changed |= ui.add_sized(
                [slider_width, 20.0],
                egui::Slider::new(&mut self.rest_duration, 2..=90)
                    .text("Rest (sec)"),
            ).changed();

            changed |= ui.add_sized(
                [slider_width, 20.0],
                egui::Slider::new(&mut self.rounds, 1..=50)
                    .text("Rounds"),
            ).changed();

            changed |= ui.add_sized(
                [slider_width, 20.0],
                egui::Slider::new(&mut self.lead_up_duration, 0..=10)
                    .text("Lead-up (sec)"),
            ).changed();

            // Save settings if any slider value changed
            if changed {
                self.save_settings();
            }

            match self.state {
                TimerState::Idle => {
                    ui.horizontal(|ui| {
                        if ui.button("Start").clicked() {
                            self.current_round = 0;
                            self.start_time = Some(Instant::now());
                            self.state = TimerState::LeadUp;
                            self.remaining_time = self.lead_up_duration as u64;
                        }
                    });
                }
                TimerState::LeadUp => {
                    ui.horizontal(|ui| {
                        if ui.button("Pause").clicked() {
                            self.state = TimerState::PausedLeadUp;
                            self.start_time = None;
                        }
                        if ui.button("Stop").clicked() {
                            self.state = TimerState::Idle;
                            self.start_time = None;
                            self.remaining_time = 0;
                            self.current_round = 0;
                        }
                    });
                }
                TimerState::Workout | TimerState::Rest => {
                    ui.horizontal(|ui| {
                        if ui.button("Pause").clicked() {
                            self.state = match self.state {
                                TimerState::Workout => TimerState::PausedWorkout,
                                TimerState::Rest => TimerState::PausedRest,
                                _ => unreachable!(),
                            };
                            self.start_time = None;
                        }
                        if ui.button("Stop").clicked() {
                            self.state = TimerState::Idle;
                            self.start_time = None;
                            self.remaining_time = 0;
                            self.current_round = 0;
                        }
                    });
                }
                TimerState::PausedLeadUp | TimerState::PausedWorkout | TimerState::PausedRest => {
                    ui.horizontal(|ui| {
                        if ui.button("Resume").clicked() {
                            self.start_time = Some(Instant::now() - Duration::from_secs(
                                match self.state {
                                    TimerState::PausedLeadUp => self.lead_up_duration as u64 - self.remaining_time,
                                    TimerState::PausedWorkout => self.workout_duration - self.remaining_time,
                                    TimerState::PausedRest => self.rest_duration - self.remaining_time,
                                    _ => unreachable!(),
                                }
                            ));
                            self.state = match self.state {
                                TimerState::PausedLeadUp => TimerState::LeadUp,
                                TimerState::PausedWorkout => TimerState::Workout,
                                TimerState::PausedRest => TimerState::Rest,
                                _ => unreachable!(),
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
            let state_label = format!("State: {:?}", self.state)
                .replace("PausedLeadUp", "Paused Lead-Up")
                .replace("PausedWorkout", "Paused Workout")
                .replace("PausedRest", "Paused Rest");
            ui.label(state_label);

            // Add countdown timer
            ui.label(format!("Time remaining: {:02}:{:02}", self.remaining_time / 60, self.remaining_time % 60));

            // Add progress bar
            let progress = match self.state {
                TimerState::LeadUp | TimerState::PausedLeadUp => {
                    1.0 - (self.remaining_time as f32 / self.lead_up_duration as f32)
                }
                TimerState::Workout | TimerState::PausedWorkout => {
                    1.0 - (self.remaining_time as f32 / self.workout_duration as f32)
                }
                TimerState::Rest | TimerState::PausedRest => {
                    1.0 - (self.remaining_time as f32 / self.rest_duration as f32)
                }
                TimerState::Idle => 0.0,
            };

            let progress_bar = egui::ProgressBar::new(progress)
            .show_percentage()
            .fill(match self.state {
                TimerState::LeadUp | TimerState::PausedLeadUp => egui::Color32::from_rgb(0xFF, 0xA5, 0x00), // Orange
                TimerState::Workout | TimerState::PausedWorkout => egui::Color32::from_rgb(0x3B, 0xA4, 0x58), // Green
                TimerState::Rest | TimerState::PausedRest => egui::Color32::from_rgb(0x38, 0x77, 0xA2), // Blue
                TimerState::Idle => egui::Color32::from_rgb(0x3D, 0x3D, 0x3D), // Gray
            });
            
            ui.add(progress_bar);
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn main() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions::default();

    // Use the window_builder hook to set the initial window size
    options.window_builder = Some(Box::new(|builder| {
        builder
            .with_title("Workout Timer") // Set the window title
            .with_inner_size((450.0, 450.0)) // Set the initial window size
    }));

    eframe::run_native(
        "Workout Timer",
        options,
        Box::new(|_cc| Ok(Box::new(WorkoutTimer::new()))),
    )
}
