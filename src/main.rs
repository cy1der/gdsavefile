use iced::{Application, Result, Settings};
use std::env::consts::OS;

mod app;
mod decoder;
mod encoder;

use app::App;

fn main() -> Result {
    if OS != "macos" {
        App::run(Settings::default())
    } else {
        println!("Unfortunately, this app does not work on MacOS because the save files are encrypted differently.\nBlame RobTop!");
        Ok(())
    }
}
