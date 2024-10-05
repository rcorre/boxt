use anyhow::Result;
use boxt::tui;
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {}

fn main() -> Result<()> {
    env_logger::init();
    let _args = CLI::parse();
    tui::start()
}
