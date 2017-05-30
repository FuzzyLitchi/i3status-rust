// This is needed because apparently the large json! macro in the icons.rs file explodes at compile time...
#![recursion_limit="128"]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;
extern crate clap;
extern crate uuid;
extern crate regex;

#[macro_use]
pub mod util;
pub mod block;
pub mod blocks;
pub mod input;
pub mod icons;
pub mod themes;
mod config;
pub mod scheduler;
pub mod widget;
pub mod widgets;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use std::time::Duration;
use std::ops::DerefMut;

use block::Block;

use input::{process_events, I3barEvent};
use scheduler::{UpdateScheduler, Task};
use themes::get_theme;
use config::Config;
use icons::get_icons;

use self::clap::{Arg, App};

fn main() {
    let mut builder = App::new("i3status-rs")
        .version("0.1")
        .author("Kai Greshake <development@kai-greshake.de>, Contributors on GitHub: \\
                 https://github.com/greshake/i3status-rust/graphs/contributors")
        .about("Replacement for i3status for Linux, written in Rust")
        .arg(Arg::with_name("config")
            .value_name("CONFIG_FILE")
            .help("sets a json config file")
            .required(true)
            .index(1))
        .arg(Arg::with_name("theme")
            .help("which theme to use, can be a builtin theme or file.\nBuiltin themes: solarized-dark, plain")
            .default_value("plain")
            .short("t")
            .long("theme"))
        .arg(Arg::with_name("icons")
            .help("which icons to use, can be a builtin set or file.\nBuiltin sets: awesome, none (textual)")
            .default_value("none")
            .short("i")
            .long("icons"))
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .takes_value(false)
            .help("Prints debug information"))
        .arg(Arg::with_name("input-check-interval")
            .help("max. delay to react to clicking, in ms")
            .default_value("50"));

    if_debug!({
        builder = builder
        .arg(Arg::with_name("profile")
            .long("profile")
            .takes_value(true)
            .help("A block to be profiled. Analyze block.profile with pprof"))
        .arg(Arg::with_name("profile-runs")
            .long("profile-runs")
            .takes_value(true)
            .default_value("10000")
            .help("How many times to execute update when profiling."));;
    });

    let matches = builder.get_matches();

    // Load all arguments
    let input_check_interval = Duration::new(0,
                                             matches
                                                 .value_of("input-check-interval")
                                                 .unwrap()
                                                 .parse::<u32>()
                                                 .expect("Not a valid integer as interval") *
                                             1000000);

    // Merge the selected icons and color theme
    let icons = get_icons(matches.value_of("icons").unwrap());
    let mut theme = get_theme(matches.value_of("theme").unwrap()).expect("Not a valid theme!");
    theme["icons"] = icons;

    let (tx, rx_update_requests): (Sender<Task>, Receiver<Task>) = mpsc::channel();

    // Load the config file
    let config = Config::new(matches.value_of("config").unwrap(), &tx, &theme);

    let mut blocks: Vec<Box<Block>> = config.blocks;

    let order = blocks.iter().map(|x| String::from(x.id())).collect();

    let mut scheduler = UpdateScheduler::new(&blocks);

    let mut block_map: HashMap<String, &mut Block> = HashMap::new();

    for block in blocks.iter_mut() {
        block_map.insert(String::from(block.id()), (*block).deref_mut());
    }

    // Now we can start to run the i3bar protocol
    print!("{{\"version\": 1, \"click_events\": true}}\n[");

    // We wait for click events in a seperate thread, to avoid blocking to wait for stdin
    let (tx, rx_clicks): (Sender<I3barEvent>, Receiver<I3barEvent>) = mpsc::channel();
    process_events(tx);

    loop {
        // See if the user has clicked.
        while let Ok(event) = rx_clicks.try_recv() {
            for (_, block) in &mut block_map {
                block.click(&event);
            }
            util::print_blocks(&order, &block_map, &theme);
        }

        // Enqueue pending update requests
        while let Ok(request) = rx_update_requests.try_recv() {
            scheduler.schedule(request)
        }

        // This interval allows us to react to click events faster,
        // while still sleeping most of the time and not requiring all
        // Blocks to be Send.
        if let Some(ttnu) = scheduler.time_to_next_update() {
            if ttnu < input_check_interval {
                scheduler.do_scheduled_updates(&mut block_map);

                // redraw the blocks, state changed
                util::print_blocks(&order, &block_map, &theme);
            } else {
                thread::sleep(input_check_interval)
            }
        }
    }
}
