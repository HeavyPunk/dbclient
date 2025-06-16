use clap::Parser;
use cli::Cli;
use config::Config;

mod cli;
mod config;
mod dbclient;
mod ui;
mod ui2;

fn main() {
    let args = Cli::parse();

    let config_content = std::fs::read_to_string(&args.config_path)
        .expect("Failed to read config file");

    let config: Config = toml::from_str(&config_content)
        .expect("Failed to parse config file");

    // ui::draw(&config);
    ui2::draw(config);
}

