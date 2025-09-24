mod cmd;
mod io;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Command-line interface for composing and inspecting Gordian Club editions.
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Manage Gordian Club editions",
    propagate_version = true,
    styles = clap::builder::Styles::styled()
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create the genesis edition for a single-publisher club.
    Init(cmd::init::CommandArgs),
    /// Operate on club editions.
    Edition(cmd::edition::CommandArgs),
    /// Manage permits for future editions.
    Permits(cmd::permits::CommandArgs),
    /// Work with encrypted club content.
    Content(cmd::content::CommandArgs),
}

fn main() -> Result<()> {
    // Calls envelope register_tags, which calls bc_components register_tags.
    provenance_mark::register_tags();

    let cli = Cli::parse();

    match cli.command {
        Command::Init(args) => cmd::init::exec(args),
        Command::Edition(args) => cmd::edition::exec(args),
        Command::Permits(args) => cmd::permits::exec(args),
        Command::Content(args) => cmd::content::exec(args),
    }
}
