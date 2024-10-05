use anyhow::Result;
use boxt::tui;
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = CLI::parse();
    tui::start(args.path)
}
