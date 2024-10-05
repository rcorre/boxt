use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use clart::Document;

#[derive(Args)]
struct RenderArgs {
    path: PathBuf,
}

#[derive(Subcommand)]
enum Command {
    Render(RenderArgs),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = CLI::parse();

    match args.command {
        Command::Render(args) => render(args),
    }
}

fn render(args: RenderArgs) -> Result<()> {
    let s = fs::read_to_string(args.path)?;
    log::trace!("Parsing:\n{s}");
    let doc: Document = toml::from_str(&s)?;
    log::trace!("Parsing:\n{s}");
    doc.draw(&mut std::io::stdout())?;
    Ok(())
}
