use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};

/// Command-line interface for composing and inspecting Gordian Club editions.
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Manage Gordian Club editions",
    propagate_version = true
)]
#[command(styles = clap::builder::Styles::styled())]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create the genesis edition for a single-publisher club.
    Init(InitArgs),
    /// Operate on club editions.
    Edition(EditionArgs),
    /// Manage permits for future editions.
    Permits(PermitsArgs),
    /// Work with encrypted club content.
    Content(ContentArgs),
}

#[derive(Debug, Args)]
struct InitArgs {
    #[command(flatten)]
    compose: EditionComposeArgs,
}

#[derive(Debug, Subcommand)]
enum EditionCommand {
    /// Compose and sign an edition.
    Compose(EditionComposeArgs),
    /// Inspect and verify an edition.
    Inspect(EditionInspectArgs),
}

#[derive(Debug, Args)]
struct EditionArgs {
    #[command(subcommand)]
    command: EditionCommand,
}

#[derive(Debug, Args)]
struct EditionComposeArgs {
    /// Publisher's XID document UR (must include signing keys).
    #[arg(long, value_name = "UR", global = true)]
    publisher: String,
    /// Content envelope UR for this edition.
    #[arg(long, value_name = "UR")]
    content: String,
    /// Provenance mark UR bound to this edition.
    #[arg(long, value_name = "UR")]
    provenance: String,
    /// Permit descriptors (XID or public-keys UR).
    #[arg(long = "permit", value_name = "UR")]
    permits: Vec<String>,
    /// Optional SSKR specifications (e.g. "2of3").
    #[arg(long = "sskr", value_name = "SPEC")]
    sskr: Vec<String>,
    /// Previous edition UR to enforce provenance ordering.
    #[arg(long, value_name = "UR")]
    previous: Option<String>,
    /// Output directory for generated artifacts.
    #[arg(long, value_name = "PATH")]
    out_dir: Option<PathBuf>,
    /// Print a human-readable summary in addition to UR outputs.
    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Args)]
struct EditionInspectArgs {
    /// Edition UR to inspect.
    #[arg(long, value_name = "UR")]
    edition: String,
    /// Optional previous edition UR for provenance validation.
    #[arg(long, value_name = "UR")]
    previous: Option<String>,
    /// Emit summary details alongside normalized UR output.
    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Subcommand)]
enum PermitsCommand {
    /// Derive a public-key permit from recipient materials.
    Derive(PermitsDeriveArgs),
}

#[derive(Debug, Args)]
struct PermitsArgs {
    #[command(subcommand)]
    command: PermitsCommand,
}

#[derive(Debug, Args)]
struct PermitsDeriveArgs {
    /// Recipient descriptor (XID document or public-keys UR).
    #[arg(long, value_name = "UR")]
    recipient: Vec<String>,
    /// Optional label to annotate the permit holder.
    #[arg(long, value_name = "XID")]
    label: Option<String>,
}

#[derive(Debug, Subcommand)]
enum ContentCommand {
    /// Decrypt edition content using permits, SSKR shards, or raw keys.
    Decrypt(ContentDecryptArgs),
}

#[derive(Debug, Args)]
struct ContentArgs {
    #[command(subcommand)]
    command: ContentCommand,
}

#[derive(Debug, Args)]
struct ContentDecryptArgs {
    /// Edition UR containing the encrypted content.
    #[arg(long, value_name = "UR")]
    edition: String,
    /// Permit URs capable of unwrapping the content key.
    #[arg(long = "permit", value_name = "UR")]
    permits: Vec<String>,
    /// SSKR share URs for recovering the content key.
    #[arg(long = "sskr", value_name = "UR")]
    shards: Vec<String>,
    /// Symmetric key UR for decrypting the content directly.
    #[arg(long, value_name = "UR")]
    key: Option<String>,
    /// Emit decrypted envelope UR to stdout.
    #[arg(long)]
    emit_ur: bool,
}

fn main() -> Result<()> {
    bc_envelope::register_tags();
    provenance_mark::register_tags();

    let cli = Cli::parse();

    match cli.command {
        Command::Init(args) => handle_init(args),
        Command::Edition(args) => match args.command {
            EditionCommand::Compose(args) => handle_edition_compose(args),
            EditionCommand::Inspect(args) => handle_edition_inspect(args),
        },
        Command::Permits(args) => match args.command {
            PermitsCommand::Derive(args) => handle_permit_derive(args),
        },
        Command::Content(args) => match args.command {
            ContentCommand::Decrypt(args) => handle_content_decrypt(args),
        },
    }
}

fn handle_init(args: InitArgs) -> Result<()> {
    let _ = args;
    bail!("clubs-cli init is not implemented yet")
}

fn handle_edition_compose(args: EditionComposeArgs) -> Result<()> {
    let _ = args;
    bail!("clubs-cli edition compose is not implemented yet")
}

fn handle_edition_inspect(args: EditionInspectArgs) -> Result<()> {
    let _ = args;
    bail!("clubs-cli edition inspect is not implemented yet")
}

fn handle_permit_derive(args: PermitsDeriveArgs) -> Result<()> {
    let _ = args;
    bail!("clubs-cli permits derive is not implemented yet")
}

fn handle_content_decrypt(args: ContentDecryptArgs) -> Result<()> {
    let _ = args;
    bail!("clubs-cli content decrypt is not implemented yet")
}
