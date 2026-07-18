mod core;

use crate::core::cli::Cli;
use anyhow::Result;
use clap::Parser;
use cliclack::{intro, outro, outro_cancel, spinner};

fn main() -> Result<()> {
    let cli = Cli::parse();

    intro("bounds")?;

    let Some(config) = core::wizard::run(cli.path)? else {
        outro_cancel("No changes were made.")?;
        return Ok(());
    };

    let spinner = spinner();
    spinner.start("Creating repository boundaries...");

    match core::generate::apply(&config) {
        Ok(files) => {
            spinner.stop("Repository boundaries created");

            let created = files
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n");

            outro(format!("Created or updated:\n{created}"))?;
            Ok(())
        }

        Err(error) => {
            spinner.error("Could not create repository boundaries");
            Err(error)
        }
    }
}
