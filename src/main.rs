use std::{thread, time::Duration};

use clap::Parser;
use cli::Cli;
use config::Config;
use ui3::model::Model;

mod cli;
mod config;
mod dbclient;
mod ui2;
mod ui3;

fn main() {
    let args = Cli::parse();

    let config_content = std::fs::read_to_string(&args.config_path)
        .expect("Failed to read config file");

    let config: Config = toml::from_str(&config_content)
        .expect("Failed to parse config file");

    // ui2::draw(config);
    Model::new(&config).main_loop();
}

