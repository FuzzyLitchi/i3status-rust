extern crate serde_json;

use util::get_file;
use block::Block;
use blocks::create_block;
use scheduler::Task;
use std::sync::mpsc::Sender;

pub struct Config {
    pub blocks: Vec<Box<Block>>,
}

impl Config {
    pub fn new(file_name: &str, tx: &Sender<Task>, theme: &serde_json::Value) -> Config {
        let config = serde_json::from_str(get_file(file_name).as_str())
            .expect("Config file is not valid JSON!");

        match config {
            serde_json::Value::Array(b) => {
                let blocks = Vec::new();
                for block in b {
                    blocks.push(create_block(block,
                                             tx.clone(),
                                             theme.clone()))
                }
                Config {
                    blocks
                }
            },
            _ => panic!("Config file doesn't have an array as the outmost value and is therefor invalid!"),
        }
    }

}
