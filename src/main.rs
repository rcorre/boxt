use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use clart::{tui, Document};

#[derive(Args)]
struct RenderArgs {
    path: PathBuf,
}

#[derive(Args)]
struct EditArgs {
    path: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    Render(RenderArgs),
    Edit(EditArgs),
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
        Command::Edit(args) => edit(args),
    }
}

fn render(args: RenderArgs) -> Result<()> {
    log::trace!("Loading document from: {:?}", args.path);
    let doc = Document::load(args.path)?;
    doc.draw(&mut std::io::stdout())?;
    Ok(())
}

fn edit(args: EditArgs) -> Result<()> {
    log::trace!("Loading document from: {:?}", args.path);
    let doc = Document::load(args.path.unwrap())?;
    tui::start(doc)
}
