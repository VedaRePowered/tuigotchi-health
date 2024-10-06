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

use std::{fs::File, path::Path, time::Duration};

use color_eyre::Result;
use crossterm::style::Color;
use serde::{Deserialize, Serialize};

use crate::task::Task;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Color")]
enum ColorDef {
    Reset,
    Black,
    DarkGrey,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Grey,
    Rgb { r: u8, g: u8, b: u8 },
    AnsiValue(u8),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub character: CharacterChoice,
    #[serde(with = "humantime_serde")]
    pub task_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub task_timeout_max: Duration,
    #[serde(with = "humantime_serde")]
    pub idle_animation_time_min: Duration,
    #[serde(with = "humantime_serde")]
    pub idle_animation_time_max: Duration,
    #[serde(with = "humantime_serde")]
    pub task_animation_duration: Duration,
    #[serde(with = "ColorDef")]
    pub colour: Color,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterChoice {
    #[serde(rename = "Debug Guy")]
    DebugGuy,
    Kitty,
}

impl CharacterChoice {
    pub fn character_name(&self) -> &'static str {
        match self {
            CharacterChoice::DebugGuy => "Debug Guy (very cool)",
            CharacterChoice::Kitty => "Kitted Catte",
        }
    }
    pub fn animation_file(&self) -> &'static str {
        match self {
            CharacterChoice::DebugGuy => include_str!("animations/debug_guy.txt"),
            CharacterChoice::Kitty => include_str!("animations/kitty.txt"),
        }
    }
}

impl Config {
    pub fn load_config(config_path: impl AsRef<Path>) -> Result<Self> {
        let mut path = config_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;
        path.push("config.yaml");
        Ok(if path.exists() {
            serde_yaml::from_str(&std::fs::read_to_string(&path)?)?
        } else {
            let config = Self::default();
            serde_yaml::to_writer(File::create(&path)?, &config)?;
            config
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        serde_yaml::from_str(include_str!("default_config.yaml")).unwrap()
    }
}
