/*
This file is part of Tamagotchi Health.

Tamagotchi Health is free software: you can redistribute it and/or
modify it under the terms of the GNU General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

Tamagotchi Health is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with Tamagotchi Health. If not, see
<https://www.gnu.org/licenses/>.
*/

use std::{collections::HashMap, time::{Duration, Instant}};

use color_eyre::Result;
use log::info;

use crate::{config::CharacterChoice, task::TaskType};

#[derive(Debug)]
pub struct LilGuyState {
    animations: HashMap<String, Vec<AnimationFrame>>,
    current_animation: LilGuyAnimation,
    animation_frame: u32,
    next_frame_time: Instant,
    lil_guy_pos: (u32, u32),
}

pub fn load_animations(text: &str) -> Result<HashMap<String, Vec<AnimationFrame>>> {
    Ok(text
        .lines()
        .collect::<Vec<_>>()
        .chunk_by(|_a, b| !b.starts_with("animation "))
        .map(|animation_lines| {
            let animation_name = animation_lines[0]
                .trim_start_matches("animation ")
                .to_string();
            let animation_frames: Vec<_> = animation_lines[1..]
                .chunk_by(|_a, b| !b.starts_with("frame "))
                .map(|frame_lines| {
                    let frame_time = frame_lines[0]
                        .trim_start_matches("frame ")
                        .trim_end_matches("ms");
                    let frame_time: f64 = frame_time.parse()?;
                    let frame_time = Duration::from_secs_f64(frame_time / 1000.0);
                    Ok(AnimationFrame {
                        duration: frame_time,
                        lines: frame_lines[1..]
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                    })
                })
                .collect::<Result<_>>()?;
            Ok((animation_name, animation_frames))
        })
        .collect::<Result<_>>()?)
}

#[derive(Debug, Default)]
pub struct AnimationFrame {
    duration: Duration,
    lines: Vec<String>,
}

#[derive(Debug, Default)]
pub enum LilGuyAnimation {
    #[default]
    Idle,
    Walk,
    Task(TaskType),
}

impl LilGuyState {
    pub fn new(character: CharacterChoice) -> Result<Self> {
        Ok(LilGuyState {
            animations: load_animations(character.get_animation_file())?,
            current_animation: LilGuyAnimation::Idle,
            animation_frame: 0,
            next_frame_time: Instant::now(),
            lil_guy_pos: (0, 0),
        })
    }
    pub fn update(&mut self) {}
}
