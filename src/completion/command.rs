//! # Generate completion command
//!
//! This module contains the [`clap`] command for generating
//! completion scripts to the standard output.

use std::io;

use clap::{value_parser, CommandFactory, Parser};
use clap_complete::Shell;
use color_eyre::eyre::Result;
use tracing::{info, instrument};

use crate::cli::Cli;

/// Print completion script for a shell to stdout.
///
/// This command allows you to generate completion script for a given
/// shell. The script is printed to the standard output. If you want
/// to write it to a file, just use unix redirection.
#[derive(Debug, Parser)]
pub struct GenerateCompletionCommand {
    /// Shell for which completion script should be generated for.
    #[arg(value_parser = value_parser!(Shell))]
    pub shell: Shell,
}

impl GenerateCompletionCommand {
    #[instrument(skip_all)]
    pub async fn execute(self) -> Result<()> {
        info!("executing generate completion command");

        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        clap_complete::generate(self.shell, &mut cmd, name, &mut io::stdout());

        Ok(())
    }
}
