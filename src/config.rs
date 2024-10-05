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

#[derive(Debug)]
pub struct Config {
    pub character: CharacterChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterChoice {
    DebugGuy,
}

impl CharacterChoice {
    pub fn get_character_name(&self) -> &'static str {
        match self {
            CharacterChoice::DebugGuy => "Debug Guy (very cool)",
        }
    }
    pub fn get_animation_file(&self) -> &'static str {
        match self {
            CharacterChoice::DebugGuy => include_str!("animations/debug_guy.txt"),
        }
    }
}

impl Config {
    pub fn load_config() -> Self {
        Self {
            character: CharacterChoice::DebugGuy,
        }
    }
}
