use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use anyhow::{Context, Result};
use jiff::Timestamp;

use crate::core::model::{Config, WriteMode};

pub fn write(config: &Config) -> Result<Option<PathBuf>> {
    let Some(license) = config.license.as_ref() else {
        return Ok(None);
    };

    if config.license_mode == WriteMode::Skip {
        return Ok(None);
    }

    let path = config.target.join("LICENSE");
    let year = Timestamp::now().in_tz("UTC")?.year().to_string();

    let contents = license
        .body
        .replace("[year]", &year)
        .replace("[yyyy]", &year)
        .replace("[fullname]", config.author.as_deref().unwrap_or_default())
        .replace(
            "[name of copyright owner]",
            config.author.as_deref().unwrap_or_default(),
        );

    match config.license_mode {
        WriteMode::Create => {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .with_context(|| format!("failed to create {}", path.display()))?;

            file.write_all(contents.as_bytes())?;
        }

        WriteMode::Replace => {
            fs::write(&path, contents)
                .with_context(|| format!("failed to replace {}", path.display()))?;
        }

        WriteMode::Merge | WriteMode::Skip => {
            return Ok(None);
        }
    }

    Ok(Some(path))
}
