mod gitignore;
mod license;

use std::path::PathBuf;

use anyhow::Result;

use crate::core::model::Config;

pub fn apply(config: &Config) -> Result<Vec<PathBuf>> {
    let mut changed = Vec::new();

    if let Some(path) = gitignore::write(config)? {
        changed.push(path);
    }

    if let Some(path) = license::write(config)? {
        changed.push(path);
    }

    Ok(changed)
}
