use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, InfoLevel};

/// Mail Synchronization Tool
#[derive(Parser)]
#[clap(name = "mail_sync", version)]
pub struct CLI {
    /// Path to Configuration File
    #[clap(short, long)]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}

#[derive(Subcommand)]
pub enum Command {
    /// List Source Mailboxes
    List,
    /// Synchronize Mailboxes
    Sync,
}
