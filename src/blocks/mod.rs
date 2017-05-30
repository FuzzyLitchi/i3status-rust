mod time;
mod template;
mod load;
mod memory;
mod cpu;
mod music;
mod battery;
mod custom;
mod disk_space;
mod pacman;
mod temperature;
mod toggle;
mod sound;
mod focused_window;

use self::time::*;
use self::template::*;
use self::music::*;
use self::cpu::*;
use self::load::*;
use self::memory::*;
use self::battery::*;
use self::custom::*;
use self::disk_space::*;
use self::pacman::*;
use self::sound::*;
use self::toggle::*;
use self::focused_window::*;
use self::temperature::*;

use super::block::Block;
use super::scheduler::Task;

extern crate serde_json;
extern crate dbus;

use serde_json::Value;
use std::sync::mpsc::Sender;

macro_rules! boxed ( { $b:expr } => { Box::new($b) as Box<Block> }; );

pub fn create_block(config: Value,
                    tx_update_request: Sender<Task>,
                    theme: Value)
                    -> Box<Block> {
    match config.clone()["block"].as_str().unwrap() {
        "time" => boxed!(Time::new(config, theme)),
        "template" => boxed!(Template::new(config, tx_update_request, theme)),
        "music" => boxed!(Music::new(config, tx_update_request, theme)),
        "load" => boxed!(Load::new(config, theme)),
        "memory" => boxed!(Memory::new(config, tx_update_request, theme)),
        "cpu" => boxed!(Cpu::new(config, theme)),
        "pacman" => boxed!(Pacman::new(config, theme)),
        "battery" => boxed!(Battery::new(config, theme)),
        "custom" => boxed!(Custom::new(config, tx_update_request, theme)),
        "disk_space" => boxed!(DiskSpace::new(config, theme)),
        "toggle" => boxed!(Toggle::new(config, theme)),
        "sound" => boxed!(Sound::new(config, theme)),
        "temperature" => boxed!(Temperature::new(config, theme)),
        "focused_window" => boxed!(FocusedWindow::new(config, tx_update_request, theme)),
        name => {
            panic!("Not a registered block: {}", name);
        }
    }
}
