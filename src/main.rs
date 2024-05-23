mod cli;
mod config;
mod session_manager;

use anyhow::Ok;
use clap::Parser;
use tracing_log::AsTrace;

use cli::{Command, CLI};
use config::Config;
use session_manager::SessionManager;

fn main() -> anyhow::Result<()> {

    let cli = CLI::parse();

    let timer = time::format_description::parse("[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second]").expect("Cataplum");
    let time_offset = time::UtcOffset::current_local_offset().unwrap_or_else(|_| time::UtcOffset::UTC);
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(time_offset, timer);
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(false)
        .with_level(true)
        .with_timer(timer)
        .with_max_level(cli.verbose.log_level_filter().as_trace())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config: Config = Config::new(cli.config)?;

    let mut session_manager = SessionManager::new(config);

    match &cli.command {
        Command::List => {
            tracing::info!("Listing all `src` mailboxes:");
            for mailbox in session_manager.list_all() {
                tracing::info!("> {}", mailbox);
            }
            let _ = session_manager.logout();
        },
        Command::Sync => {
            session_manager.sync();
            let _ = session_manager.logout();
        }
    }

    Ok(())
}
