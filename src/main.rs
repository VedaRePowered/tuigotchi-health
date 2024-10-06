#![feature(btree_cursors)]
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

use std::{fs::File, io::BufWriter};

use color_eyre::{eyre::OptionExt, Result};
use config::Config;
use interface::InterfaceState;
use log::{error, warn};
use simplelog::WriteLogger;
use task_manager::TaskManager;

mod config;
mod interface;
mod task;
mod task_manager;

fn not_main() -> Result<()> {
    color_eyre::install()?;

    WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        File::create("log.txt").unwrap(),
    )?;

    let dirs =
        directories::ProjectDirs::from("ca.vedapowered", "Trans Girlies", "Tamagotchi Health")
            .ok_or_eyre("Failed to load config dir!")?;
    let mut config = Config::load_config(dirs.config_dir())?;
    let mut task_manager = TaskManager::new(&mut config)?;
    let mut interface = InterfaceState::new(&config)?;
    let mut stdout = BufWriter::new(std::io::stdout());
    while interface.update(&mut task_manager)? {
        if let Err(e) = interface.render(&mut stdout) {
            warn!("Rendering error: {e}");
        }
        // Do other updates and stuff
    }
    Ok(())
}

fn main() {
    if let Err(e) = not_main() {
        error!("AAAAAAAAAAAAAAAAAAAAA We crashed: {e}");
    }
}
