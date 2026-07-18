use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "bounds",
    version,
    about = "Add a .gitignore and LICENSE to a repository"
)]
pub struct Cli {
    /// Repository to configure
    pub path: Option<PathBuf>,
}
