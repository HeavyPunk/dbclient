use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub config_path: std::path::PathBuf
}
