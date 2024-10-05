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

use chrono::{NaiveTime,Duration,DateTime,Local};
use std::collections::BTreeSet;
use std::ops::Bound;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    ty: TaskType,
    schedule: Schedule,
    #[serde(default = "Local::now", skip)]
    pub last_done: DateTime<Local>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Eat,
    Drink,
    BrushTeeth,
    Shower,
    EyesRest,
    TakeMeds,
    Sleep,
    Bathroom,
    Other(String),
}
pub use TaskType::*;

impl Task {
    pub fn new(ty: TaskType, schedule: Schedule) -> Self {
        Self { ty, schedule, last_done: Local::now() }
    }

    pub fn ty(&self) -> TaskType {
        self.ty.clone()
    }

    pub fn schedule(&self) -> Schedule {
        self.schedule.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    Times(BTreeSet<NaiveTime>),
    Interval(#[serde(with = "humantime_serde")] std::time::Duration)
}
use Schedule::{Times,Interval};

impl Schedule {
    pub fn next_instance(&self, now: DateTime<Local>) -> Result<DateTime<Local>> {
        match self {
            Times(times) => {
                match times.lower_bound(Bound::Excluded(&now.time())).peek_next() {
                    Some(&t) => Ok(now.with_time(t).earliest().ok_or_eyre("Time is mean :(")?),
                    // If there's no next event, then it's
                    // tomorrow's first
                    None => Ok((now + chrono::Days::new(1))
                       .with_time(*times.first().ok_or_eyre("No times in schedule!")?)
                       .earliest()
                       .ok_or_eyre("Time is mean :(")?)
                }
            },
            &Interval(interval) => {
                Ok(now + interval)
            }
        }
    }
}
