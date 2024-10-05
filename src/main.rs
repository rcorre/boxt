use anyhow::Result;
use boxt::{config::Config, tui};
use clap::Parser;

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = CLI::parse();

    let xdg = xdg::BaseDirectories::with_prefix(APP_NAME)?;

    let config_path = xdg.get_config_file("config.toml");
    log::debug!("Reading config from {config_path:?}");

    let config = std::fs::read_to_string(config_path)?;
    log::trace!("Read config:\n {config:?}");

    let config: Config = toml::from_str(&config)?;
    log::trace!("Parsed config:\n {config:?}");

    tui::start(config, args.path)
}
