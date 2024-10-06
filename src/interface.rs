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
    collections::{BTreeMap, VecDeque},
    io::Write,
    time::{Duration, Instant},
};

use chrono::Local;
use color_eyre::Result;
use crossterm::{
    cursor,
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use lil_guy::LilGuyState;
use log::info;
use notify_rust::{Hint, Urgency};

use crate::{
    config::Config,
    task::TaskType,
    task_manager::{TaskManager, Tasks},
};

mod lil_guy;

const NOTIFY_APPNAME: &str = "tamagotchi-health";

#[derive(Debug)]
pub struct InterfaceState {
    lil_guy: LilGuyState,
    tasks: Tasks,
    keybinds: BTreeMap<char, TaskType>,
    task_timeout: Duration,
    task_timeout_max: Duration,
    task_animations: VecDeque<TaskType>,
    current_task_animation: Option<(TaskType, Instant)>,
    task_animation_duration: Duration,
    mood: &'static str,
    char_name: String,
}

impl InterfaceState {
    pub fn new(conf: &Config) -> Result<Self> {
        let mut stdout = std::io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            cursor::Hide,
            Clear(ClearType::All)
        )?;
        terminal::enable_raw_mode()?;
        Ok(InterfaceState {
            lil_guy: LilGuyState::new(
                conf.character,
                conf.colour,
                conf.idle_animation_time_min..conf.idle_animation_time_max,
            )?,
            tasks: Tasks::default(),
            keybinds: BTreeMap::new(),
            task_timeout: conf.task_timeout_max,
            task_timeout_max: conf.task_timeout_max,
            task_animations: VecDeque::new(),
            current_task_animation: None,
            task_animation_duration: conf.task_animation_duration,
            mood: "",
            char_name: conf.character_name().to_string(),
        })
    }
    /// Update the state of the interface, will run every ~100ms
    /// returns false if the program should exit.
    pub fn update(&mut self, task_manager: &mut TaskManager) -> Result<bool> {
        self.keybinds = {
            let mut number_keybind = 0;
            self.tasks
                .current
                .iter()
                .chain(self.tasks.past.iter())
                .map(|task| {
                    let keybind = task.ty.keybind().unwrap_or_else(|| {
                        number_keybind += 1;
                        (b'0' + number_keybind as u8) as char
                    });
                    (keybind, task.ty.clone())
                })
                .collect()
        };
        let now = Local::now();
        let now_std = Instant::now();
        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            match ev {
                Event::Key(
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    },
                ) => {
                    // Quit
                    return Ok(false);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(key),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    ..
                }) => {
                    if let Some(task_type) = self.keybinds.remove(&key) {
                        task_manager.complete_tasks(&task_type, now);
                        self.task_animations.push_back(task_type);
                    }
                }
                _ => info!("Unused event: {ev:?}"),
            }
        }
        let new_tasks = task_manager.tasks(now)?;
        // This is ugly but uhhh err ummm uhh... Look! Over there! The Good Year blimp!
        let notify_tasks = new_tasks
            .current
            .iter()
            .filter(|task| !self.tasks.current.contains(task))
            .map(|task| task.ty.clone());
        self.notify_tasks(notify_tasks, false)?;
        let priority_notify_tasks = new_tasks
            .past
            .iter()
            .filter(|task| !self.tasks.past.contains(task))
            .map(|task| task.ty.clone());
        self.notify_tasks(priority_notify_tasks, true)?;
        self.tasks = new_tasks;

        if let Some((_task_type, end_time)) = &self.current_task_animation {
            if *end_time > now_std {
                self.current_task_animation = None;
            }
        }
        if self.current_task_animation.is_none() && !self.task_animations.is_empty() {
            if let Some(task_animation) = self.task_animations.pop_front() {
                self.current_task_animation =
                    Some((task_animation, now_std + self.task_animation_duration));
            }
        }

        let happiness = 1.0
            - self
                .tasks
                .past
                .iter()
                .map(|task| {
                    // This formula is not special its just a random thing I came up with
                    (((now - task.when).num_seconds() as f32 - self.task_timeout.as_secs_f32())
                        / self.task_timeout_max.as_secs_f32())
                    .sqrt()
                })
                .sum::<f32>()
                .clamp(0.0, 1.0);
        self.mood = if happiness > 0.6 { "Happy" } else { "Sad" };

        let screen_size = terminal::window_size()?;
        self.lil_guy.update(
            happiness,
            self.current_task_animation.as_ref().map(|ta| &ta.0),
            (
                0i32..screen_size.columns as i32 - 4,
                0i32..screen_size.rows as i32 - 12.max(self.keybinds.len() as i32 + 2),
            ),
        )?;
        Ok(true)
    }
    /// Render the interface
    pub fn render(&self, writer: &mut impl Write) -> Result<()> {
        let screen_size = terminal::window_size()?;
        let text_height = 12.max(self.keybinds.len() as i32 + 2);
        queue!(writer, Clear(ClearType::All))?;
        queue!(
            writer,
            MoveTo(10, 2),
            Print(format!("{} is {}.", self.char_name, self.mood))
        )?;
        self.lil_guy
            .render(writer, (2, screen_size.rows as i32 - text_height))?;
        queue!(
            writer,
            MoveTo(3, screen_size.rows - text_height as u16),
            Print("=".repeat(screen_size.columns as usize - 6))
        )?;
        for (i, (keybind, task_type)) in self.keybinds.iter().enumerate() {
            queue!(
                writer,
                MoveTo(10, i as u16 + screen_size.rows - text_height as u16 + 1),
                Print(format!(
                    " - {} Press '{}' to {}.",
                    task_type,
                    keybind,
                    task_type.verb()
                ))
            )?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Send a notification and play a sound for a task
    fn notify_tasks(&self, tasks: impl Iterator<Item = TaskType>, is_priority: bool) -> Result<()> {
        for task in tasks {
            notify_rust::Notification::new()
                .summary(&format!("{}", task))
                .appname(NOTIFY_APPNAME)
                .timeout(Duration::from_secs(60))
                .hint(Hint::Urgency(if is_priority {
                    Urgency::Critical
                } else {
                    Urgency::Normal
                }))
                .show()?;
        }

        Ok(())
    }
}

impl Drop for InterfaceState {
    /// Finialize the interface, reset the terminal state
    fn drop(&mut self) {
        let _ = execute!(
            std::io::stdout(),
            Clear(ClearType::All),
            LeaveAlternateScreen,
            cursor::Show,
        );
        let _ = terminal::disable_raw_mode();
    }
}
