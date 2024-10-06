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
// along with Tamagotchi Health. If not, see
// <https://www.gnu.org/licenses/>.
 */

use chrono::DurationRound;
use chrono::{DateTime, Duration, Local, NaiveTime};
use color_eyre::eyre::OptionExt;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::ops::Bound;

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    #[serde(rename = "type")]
    ty: TaskType,
    schedule: Schedule,
    #[serde(default = "Local::now", skip)]
    pub last_done: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    Eat,
    Drink,
    #[serde(rename = "Brush Teeth")]
    BrushTeeth,
    Shower,
    #[serde(rename = "Eyes Rest")]
    EyesRest,
    #[serde(rename = "Take Meds")]
    TakeMeds,
    Sleep,
    Bathroom,
    Other(String),
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TaskType::*;

        let disp = match self {
            TaskType::Eat => "I'm hungry!",
            TaskType::Drink => "I'm thirsty!",
            TaskType::BrushTeeth => "My breath smells!",
            TaskType::Shower => "I'm stinky!",
            TaskType::EyesRest => "My eyes are tired!",
            TaskType::TakeMeds => "I don't feel good >.<",
            TaskType::Sleep => "I'm eepy!",
            TaskType::Bathroom => "I have to go!",
            TaskType::Other(d) => {
                write!(f, "I need to {}", d)?;
                return Ok(());
            }
        };

        write!(f, "{}", disp)
    }
}

impl Task {
    pub fn new(ty: TaskType, schedule: Schedule) -> Self {
        Self {
            ty,
            schedule,
            last_done: Local::now(),
        }
    }

    pub fn ty(&self) -> &TaskType {
        &self.ty
    }

    pub fn schedule(&self) -> Schedule {
        self.schedule.clone()
    }

    pub fn complete(&mut self, now: DateTime<Local>) {
        self.last_done = now;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    Times(BTreeSet<NaiveTime>),
    Interval(#[serde(with = "humantime_serde")] std::time::Duration),
}
use Schedule::{Interval, Times};

impl Schedule {
    pub fn next_instance(&self, now: DateTime<Local>) -> Result<DateTime<Local>> {
        match self {
            Times(times) => {
                match times.lower_bound(Bound::Excluded(&now.time())).peek_next() {
                    // i don't wanna handle times that don't exist due
                    // to time change right now, just say those tasks
                    // happen at midnight for now
                    Some(&t) => Ok(now
                        .with_time(t)
                        .earliest()
                        .unwrap_or_else(|| now.duration_round(Duration::days(1)).unwrap())),
                    // If there's no next event, then it's
                    // tomorrow's first
                    None => Ok((now + chrono::Days::new(1))
                        .with_time(*times.first().ok_or_eyre("No times in schedule!")?)
                        .earliest()
                        .unwrap_or_else(|| now.duration_round(Duration::days(1)).unwrap())),
                }
            }
            &Interval(interval) => Ok(now + interval),
        }
    }
}
