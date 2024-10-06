/*
This file is part of Tuigotchi Health.

Tuigotchi Health is free software: you can redistribute it and/or
modify it under the terms of the GNU General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

Tuigotchi Health is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with Tuigotchi Health. If not, see
<https://www.gnu.org/licenses/>.
*/

use std::{
    collections::{BTreeMap, VecDeque},
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

use chrono::Local;
use color_eyre::Result;
use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Print, StyledContent, Stylize},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use lil_guy::LilGuyState;
use log::info;
#[cfg(target_os = "linux")]
use notify_rust::NotificationHandle;
use notify_rust::{Hint, Urgency};
#[cfg(not(target_os = "linux"))]
type NotificationHandle = ();
use playback_rs::{Player, Song};
use rand::{self, seq::SliceRandom};

use crate::{
    config::Config,
    task::TaskType,
    task_manager::{TaskManager, Tasks},
};

mod lil_guy;

const NOTIFY_APPNAME: &str = "tuigotchi-health";

pub struct InterfaceState {
    lil_guy: LilGuyState,
    tasks: Tasks,
    keybinds: BTreeMap<char, TaskType>,
    task_timeout: Duration,
    task_timeout_max: Duration,
    task_animations: VecDeque<TaskType>,
    current_task_animation: Option<(TaskType, Instant)>,
    task_animation_duration: Duration,
    mood: StyledContent<&'static str>,
    char_name: String,
    temp_icon_path: PathBuf,
    notifications: Vec<(TaskType, Option<NotificationHandle>)>,
    temp_meow_paths: Vec<PathBuf>,
    player: Player,
    text_colour: crossterm::style::Color,
    task_colour: crossterm::style::Color,
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
        let temp_icon_path = std::env::temp_dir().join("__kitty_notification_icon.png");
        std::fs::write(&temp_icon_path, include_bytes!("kitty_icon.png"))?;
        let temp_meow1_path = std::env::temp_dir().join("__meow1.wav");
        std::fs::write(&temp_meow1_path, include_bytes!("sounds/meow1.wav"))?;
        let temp_meow2_path = std::env::temp_dir().join("__meow2.wav");
        std::fs::write(&temp_meow2_path, include_bytes!("sounds/meow2.wav"))?;
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
            mood: "".with(style::Color::Grey),
            char_name: conf.character_name().to_string(),
            temp_icon_path,
            notifications: Vec::new(),
            temp_meow_paths: vec![temp_meow1_path, temp_meow2_path],
            player: Player::new(None)?,
            text_colour: conf.text_colour,
            task_colour: conf.task_colour,
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
                        // This would be so much nicer if retain was still drain_filter...
                        self.notifications.retain_mut(|(ty, notif)| {
                            if ty == &task_type {
                                if let Some(notif) = notif.take() {
                                    #[cfg(target_os = "linux")]
                                    notif.close();
                                }
                                false
                            } else {
                                true
                            }
                        });
                        self.task_animations.push_back(task_type);
                    }
                }
                _ => info!("Unused event: {ev:?}"),
            }
        }
        let new_tasks = task_manager.tasks(now)?;
        // This is ugly but uhhh err ummm uhh... Look! Over there! The Good Year blimp!
        let notify_tasks: Vec<_> = new_tasks
            .current
            .iter()
            .filter(|task| !self.tasks.current.contains(task))
            .map(|task| task.ty.clone())
            .collect();
        self.notify_tasks(notify_tasks.into_iter(), false)?;
        let priority_notify_tasks: Vec<_> = new_tasks
            .past
            .iter()
            .filter(|task| !self.tasks.past.contains(task))
            .map(|task| task.ty.clone())
            .collect();
        self.notify_tasks(priority_notify_tasks.into_iter(), true)?;
        self.tasks = new_tasks;

        if let Some((_task_type, end_time)) = &self.current_task_animation {
            if *end_time < now_std {
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
                    .max(0.0)
                    .sqrt()
                })
                .sum::<f32>()
                .clamp(0.0, 1.0);
        self.mood = match happiness {
            ..=0.1 => "Very Sad".with(style::Color::DarkRed),
            0.1..=0.4 => "Sad".with(style::Color::DarkMagenta),
            0.4..=0.6 => "Neutral".with(style::Color::Grey),
            0.6..=0.9 => "Happy".with(style::Color::Blue),
            0.9.. => "Very Happy".with(style::Color::Green),
            _ => "Unknown".with(style::Color::Magenta),
        };

        let screen_size = terminal::size()?;
        self.lil_guy.update(
            happiness,
            self.current_task_animation.as_ref().map(|ta| &ta.0),
            (
                0i32..screen_size.0 as i32 - 4,
                0i32..screen_size.1 as i32 - 12.max(self.keybinds.len() as i32 + 2),
            ),
            &self.tasks.past,
        )?;
        Ok(true)
    }
    /// Render the interface
    pub fn render(&self, writer: &mut impl Write) -> Result<()> {
        let screen_size = terminal::size()?;
        let text_height = 12.max(self.keybinds.len() as i32 + 2);
        queue!(writer, Clear(ClearType::All))?;
        queue!(
            writer,
            MoveTo(10, 2),
            Print(format!("{} is ", self.char_name,).with(self.text_colour)),
            Print(self.mood),
            Print(".".with(self.text_colour)),
        )?;
        self.lil_guy
            .render(writer, (2, screen_size.1 as i32 - text_height))?;
        queue!(
            writer,
            MoveTo(3, screen_size.1 - text_height as u16),
            Print(
                "=".repeat(screen_size.0 as usize - 6)
                    .with(self.text_colour)
            )
        )?;
        for (i, (keybind, task_type)) in self.keybinds.iter().enumerate() {
            queue!(
                writer,
                MoveTo(10, i as u16 + screen_size.1 - text_height as u16 + 1),
                Print(" - ".with(self.text_colour)),
                Print(task_type.to_string().with(self.task_colour)),
                Print(" Press '".with(self.text_colour)),
                Print(keybind.to_string().with(self.task_colour)),
                Print(format!("' to {}.", task_type.verb()).with(self.text_colour)),
            )?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Send a notification and play a sound for a task
    fn notify_tasks(
        &mut self,
        tasks: impl Iterator<Item = TaskType>,
        is_priority: bool,
    ) -> Result<()> {
        let mut was_task = false;

        for task in tasks {
            let mut notif = notify_rust::Notification::new()
                .summary(&format!("{}", task))
                .appname(NOTIFY_APPNAME)
                .timeout(Duration::from_secs(60))
                .icon(&self.temp_icon_path.to_string_lossy())
                .finalize();
            #[cfg(target_os = "linux")]
            let notif = notif.hint(Hint::Urgency(if is_priority {
                Urgency::Critical
            } else {
                Urgency::Normal
            }));
            self.notifications.push((task.clone(), Some(notif.show()?)));

            was_task = true;
        }

        if was_task {
            let song = Song::from_file(
                self.temp_meow_paths
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
                None,
            )?;
            self.player.play_song_next(&song, None)?;
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
