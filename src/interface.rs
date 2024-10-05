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

use std::{io::Write, time::Duration};

use chrono::Local;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::info;
use visual::LilGuyState;

use crate::{
    config::Config,
    task::{Task, TaskType},
    task_manager::{TaskManager, Tasks, TASK_THRESHOLD},
};

mod visual;

const NOTIFY_APPNAME: &'static str = "tamagotchi-health";

#[derive(Debug)]
pub struct InterfaceState {
    lil_guy: LilGuyState,
    tasks: Tasks,
}

impl InterfaceState {
    pub fn new(conf: &Config) -> Result<Self> {
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))?;
        terminal::enable_raw_mode()?;
        Ok(InterfaceState {
            lil_guy: LilGuyState::new(conf.character)?,
            tasks: Tasks::default(),
        })
    }
    /// Update the state of the interface, will run every ~100ms
    /// returns false if the program should exit.
    pub fn update(&mut self, task_manager: &TaskManager) -> Result<bool> {
        if event::poll(Duration::from_millis(100))? {
            info!("Hi!");
            let ev = event::read()?;
            match ev {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    // Quit
                    return Ok(false);
                }
                _ => info!("Unused event: {ev:?}"),
            }
        }
        let now = Local::now();
        let new_tasks = task_manager.get_tasks(now)?;
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
                    ((now - task.when - TASK_THRESHOLD).num_seconds() as f32 / 60.0).sqrt()
                })
                .sum::<f32>()
                .clamp(0.0, 1.0);
        self.lil_guy.update(happiness, None, (0i32..20, 0i32..10));
        Ok(true)
    }
    pub fn render(&self, writer: &mut impl Write) -> Result<()> {
        queue!(writer, Clear(ClearType::All))?;
        self.lil_guy.render(writer, (10, 10))?;
        writer.flush()?;
        Ok(())
    }

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
    fn drop(&mut self) {
        let _ = execute!(
            std::io::stdout(),
            Clear(ClearType::All),
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}
