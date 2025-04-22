# Workout Timer

A simple interval timer for workouts, built with Rust and eframe.

![Interval timer Screenshot 2025-04-14 233919](https://github.com/user-attachments/assets/67592c30-64c2-46f2-9311-b5a23a6a558e)

## Features

* Customizable workout and rest intervals
* Multiple rounds with automatic progression
* Audio cues for workout and rest intervals
* Simple and intuitive UI

## Usage from IDE

1. Run the program using `cargo run` (requires Rust and Cargo installed)
2. Configure the workout and rest intervals, number of rounds, and audio cues as desired
3. Click "Start" to begin the workout
4. The program will automatically progress through the intervals and rounds, playing audio cues as needed

## Usage from exe

1. Just run exe file

## Configuration

The program uses the following configuration options:

* `workout_duration`: the length of the workout interval in seconds (default: 60)
* `rest_duration`: the length of the rest interval in seconds (default: 45)
* `rounds`: the number of rounds to complete (default: 10)
* `work_finish_audio`: the audio file to play at the end of the workout interval (default: `../work_finish.mp3`)
* `rest_finish_audio`: the audio file to play at the end of the rest interval (default: `../rest_finish.mp3`)

## Dependencies

* Rust 1.51 or later
* Cargo 1.51 or later
* eframe 0.16 or later
* rodio 0.16 or later

## License

This program is licensed under the MIT License. See `LICENSE` for details.
