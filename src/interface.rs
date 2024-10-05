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

use std::time::Duration;

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::info;
use visual::LilGuyState;

use crate::config::Config;

mod visual;

#[derive(Debug)]
pub struct InterfaceState {
    lil_guy: LilGuyState,
}

impl InterfaceState {
    pub fn new(conf: &Config) -> Result<Self> {
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))?;
        terminal::enable_raw_mode()?;
        Ok(InterfaceState {
            lil_guy: LilGuyState::new(conf.character)?,
        })
    }
    /// Update the state of the interface, will run every ~100ms
    /// returns false if the program should exit.
    pub fn update(&mut self) -> Result<bool> {
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
        Ok(true)
    }
}

impl Drop for InterfaceState {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), Clear(ClearType::All), LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
