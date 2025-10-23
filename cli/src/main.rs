use std::path::PathBuf;

use clap::Parser;

mod command;
mod ui;

use tracing::debug;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::ui::cli::CliUiHandler;

#[derive(Debug, Parser)]
#[clap(name = "Scruff", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    #[clap(about = "Clean a directory")]
    Clean {
        #[clap(help = "Path to clean")]
        path: PathBuf,
        #[clap(
            long = "dry-run",
            help = "Preview what would be deleted without actually deleting anything"
        )]
        dry_run: bool,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    // Initialize logging
    initialize_logging();

    debug!("Debug logging enabled.");

    let mut ui = CliUiHandler;

    match opt.command {
        Command::Clean { path, dry_run } => command::clean(&path, dry_run, &mut ui)?,
    };

    Ok(())
}

fn initialize_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default tracing subscriber failed!");
}
