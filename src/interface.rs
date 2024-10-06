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
    collections::{BTreeMap, HashMap},
    io::Write,
    time::Duration,
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
use log::info;
use visual::LilGuyState;

use crate::{
    config::Config,
    task::TaskType,
    task_manager::{TaskManager, Tasks},
};

mod visual;

const NOTIFY_APPNAME: &'static str = "tamagotchi-health";

#[derive(Debug)]
pub struct InterfaceState {
    lil_guy: LilGuyState,
    tasks: Tasks,
    keybinds: BTreeMap<char, TaskType>,
    task_timeout: Duration,
    task_timeout_max: Duration,
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
                        ('0' as u8 + number_keybind as u8) as char
                    });
                    (keybind, task.ty.clone())
                })
                .collect()
        };
        let now = Local::now();
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
                    if let Some(task_type) = self.keybinds.get(&key) {
                        task_manager.complete_tasks(task_type, now);
                    }
                }
                _ => info!("Unused event: {ev:?}"),
            }
        }
        let new_tasks = task_manager.tasks(now)?;
        let notify_tasks = new_tasks
            .current
            .iter()
            .filter(|task| !self.tasks.current.contains(&task))
            .map(|task| task.ty.clone());
        self.notify_tasks(notify_tasks)?;
        self.tasks = new_tasks;
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

        let screen_size = terminal::window_size()?;
        self.lil_guy.update(
            happiness,
            None,
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
        self.lil_guy.render(
            writer,
            (
                2,
                screen_size.rows as i32 - text_height,
            ),
        )?;
        for (i, (keybind, task_type)) in self.keybinds.iter().enumerate() {
            queue!(
                writer,
                MoveTo(10, i as u16 + screen_size.rows - text_height as u16),
                Print(format!(
                    " - {}, Press '{}' to {}",
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
    fn notify_tasks<'a>(&self, tasks: impl Iterator<Item = TaskType>) -> Result<()> {
        for task in tasks {
            notify_rust::Notification::new()
                .summary(&*format!("{}", task))
                .appname(NOTIFY_APPNAME)
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
