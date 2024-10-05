use anyhow::Result;
use clap::Parser;
use clart::tui;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {}

fn main() -> Result<()> {
    env_logger::init();
    let _args = CLI::parse();
    tui::start()
}
