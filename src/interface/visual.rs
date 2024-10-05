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

use std::{
    collections::HashMap, io::Write, ops::Range, str::FromStr, time::{Duration, Instant}
};

use color_eyre::{eyre::bail, Result};
use crossterm::{cursor::MoveTo, queue, style::Print};
use rand::{thread_rng, Rng};

use crate::{config::CharacterChoice, task::TaskType};

#[derive(Debug)]
pub struct LilGuyState {
    animations: HashMap<LilGuyAnimation, Vec<AnimationFrame>>,
    current_animation: LilGuyAnimation,
    animation_frame: usize,
    next_frame_time: Instant,
    idle_animation_change: Instant,
    pos: (i32, i32),
}

fn load_animations(text: &str) -> Result<HashMap<LilGuyAnimation, Vec<AnimationFrame>>> {
    text.lines()
        .collect::<Vec<_>>()
        .chunk_by(|_a, b| !b.starts_with("animation "))
        .map(|animation_lines| {
            let animation_name: LilGuyAnimation = animation_lines[0]
                .trim_start_matches("animation ")
                .parse()?;
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
                        lines: frame_lines[1..].iter().map(|s| s.to_string()).collect(),
                    })
                })
                .collect::<Result<_>>()?;
            Ok((animation_name, animation_frames))
        })
        .collect::<Result<_>>()
}

#[derive(Debug, Default)]
pub struct AnimationFrame {
    duration: Duration,
    lines: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum LilGuyAnimation {
    #[default]
    Idle,
    Walk,
    WalkLeft,
    WalkRight,
    Sad(u32),
    Want(TaskType),
    Task(TaskType),
}

impl LilGuyAnimation {
    fn get_fallback(&self) -> LilGuyAnimation {
        match self {
            LilGuyAnimation::WalkLeft => LilGuyAnimation::Walk,
            LilGuyAnimation::WalkRight => LilGuyAnimation::Walk,
            LilGuyAnimation::Sad(n) if *n > 0 => LilGuyAnimation::Sad(*n - 1),
            LilGuyAnimation::Want(t) if *t != TaskType::Other("".to_string()) => {
                LilGuyAnimation::Sad(0)
            }
            LilGuyAnimation::Task(t) if *t != TaskType::Other("".to_string()) => {
                LilGuyAnimation::Task(TaskType::Other("".to_string()))
            }
            _ => LilGuyAnimation::Idle,
        }
    }
}

impl FromStr for LilGuyAnimation {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "idle" => Self::Idle,
            "walk" => Self::Walk,
            "walk/left" => Self::WalkLeft,
            "walk/right" => Self::WalkRight,
            "sad/0" => Self::Sad(0),
            "sad/1" => Self::Sad(1),
            "want/eat" => Self::Want(TaskType::Eat),
            "want/drink" => Self::Want(TaskType::Drink),
            "want/brush_teeth" => Self::Want(TaskType::BrushTeeth),
            "want/shower" => Self::Want(TaskType::Shower),
            "want/eyes_rest" => Self::Want(TaskType::EyesRest),
            "want/take_meds" => Self::Want(TaskType::TakeMeds),
            "want/sleep" => Self::Want(TaskType::Sleep),
            "want/bathroom" => Self::Want(TaskType::Bathroom),
            "task/general" => Self::Task(TaskType::Other(String::new())),
            "task/eat" => Self::Task(TaskType::Eat),
            "task/drink" => Self::Task(TaskType::Drink),
            "task/brush_teeth" => Self::Task(TaskType::BrushTeeth),
            "task/shower" => Self::Task(TaskType::Shower),
            "task/eyes_rest" => Self::Task(TaskType::EyesRest),
            "task/take_meds" => Self::Task(TaskType::TakeMeds),
            "task/sleep" => Self::Task(TaskType::Sleep),
            "task/bathroom" => Self::Task(TaskType::Bathroom),
            _ => bail!("Unknown task type: {s}"),
        })
    }
}

impl LilGuyState {
    pub fn new(character: CharacterChoice) -> Result<Self> {
        Ok(LilGuyState {
            animations: load_animations(character.get_animation_file())?,
            current_animation: LilGuyAnimation::Idle,
            animation_frame: 0,
            next_frame_time: Instant::now(),
            idle_animation_change: Instant::now(),
            pos: (0, 0),
        })
    }
    pub fn update(
        &mut self,
        happiness: f32,
        ongoing_task: Option<TaskType>,
        room_bounds: (Range<i32>, Range<i32>),
    ) {
        let now = Instant::now();
        if self.pos.0 < room_bounds.0.start {
            self.current_animation = LilGuyAnimation::WalkRight;
        } else if self.pos.0 > room_bounds.0.end {
            self.current_animation = LilGuyAnimation::WalkLeft;
        } else if let Some(task) = ongoing_task {
            self.current_animation = LilGuyAnimation::Task(task);
        } else if happiness < 0.6 {
            let sad_level = (((1.0 - happiness / 0.6) * 2.0).floor() as u32).clamp(0, 1);
            self.current_animation = LilGuyAnimation::Sad(sad_level);
        } else if self.idle_animation_change < Instant::now() {
            let mut rng = thread_rng();
            self.idle_animation_change =
                Instant::now() + Duration::from_secs_f32(rng.gen_range(1.0..5.0));
            // FIXME: I don't like this but eh I'll fix it later...
            if rng.gen_ratio(1, 3) {
                self.current_animation = if rng.gen_bool(0.5) {
                    LilGuyAnimation::WalkLeft
                } else {
                    LilGuyAnimation::WalkRight
                };
            } else if rng.gen_ratio(1, 2) {
                self.current_animation = LilGuyAnimation::Idle;
            }
        }
        let anim = &self
            .animations
            .get(&self.current_animation)
            .or_else(|| self.animations.get(&self.current_animation.get_fallback()))
            .expect("No Suitable Animation Found!");
        if now > self.next_frame_time {
            self.animation_frame += 1;
            if self.animation_frame >= anim.len() {
                self.animation_frame = 0;
            }
            self.next_frame_time = now + anim[self.animation_frame].duration;
            match self.current_animation {
                LilGuyAnimation::WalkLeft => self.pos.0 -= 1,
                LilGuyAnimation::WalkRight => self.pos.0 += 1,
                _ => {}
            }
        }
    }
    pub fn render(&self, writer: &mut impl Write, center: (i32, i32)) -> Result<()> {
        let pos = (center.0 + self.pos.0, center.1 + self.pos.1);
        let frame = &self.animations[&self.current_animation][self.animation_frame];
        for (y, line) in frame.lines.iter().enumerate() {
            queue!(
                writer,
                MoveTo(
                    pos.0.clamp(0, 65535) as u16,
                    (pos.1 + (y as i32)).clamp(0, 65535) as u16
                ),
                Print(line),
            )?;
        }
        Ok(())
    }
}
