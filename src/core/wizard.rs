use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Result, bail};
use cliclack::{confirm, input, multiselect, note, select, spinner};

use crate::core::{
    api::{github::GitHubClient, gitignore::GitignoreClient},
    detect,
    model::{Config, GitignoreTemplate, LicenseSummary, LicenseTemplate, WriteMode},
};

pub fn run(path: Option<PathBuf>) -> Result<Option<Config>> {
    let gitignore = GitignoreClient::new();
    let github = GitHubClient::new();
    let target = choose_target(path)?;

    if !target.is_dir() {
        bail!("{} is not a directory", target.display());
    }

    let files = multiselect("What should bounds configure?")
        .item(
            "gitignore",
            ".gitignore",
            "Ignore generated and local files",
        )
        .item("license", "LICENSE", "Define how the project may be used")
        .initial_values(vec!["gitignore", "license"])
        .interact()?;

    let mut gitignores = if files.contains(&"gitignore") {
        choose_gitignores(&gitignore, &target)?
    } else {
        Vec::new()
    };

    let mut license = if files.contains(&"license") {
        Some(choose_license(&github)?)
    } else {
        None
    };

    let gitignore_mode = choose_gitignore_mode(&target, !gitignores.is_empty())?;

    let license_mode = choose_license_mode(&target, license.is_some())?;

    if gitignore_mode == WriteMode::Skip {
        gitignores.clear();
    }

    if license_mode == WriteMode::Skip {
        license = None;
    }

    let author = match &license {
        Some(license) if license.needs_author() => Some(choose_author()?),

        _ => None,
    };

    let config = Config {
        target,
        gitignores,
        gitignore_mode,
        license,
        license_mode,
        author,
    };

    note("Bounds will apply", summary(&config))?;

    if !confirm("Continue?").initial_value(true).interact()? {
        return Ok(None);
    }

    Ok(Some(config))
}

fn choose_target(path: Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(path) => Ok(path),

        None => {
            let path: String = input("Where should bounds add the files?")
                .default_input(".")
                .interact()?;

            Ok(PathBuf::from(path))
        }
    }
}

fn choose_gitignores(gitignore: &GitignoreClient, target: &Path) -> Result<Vec<GitignoreTemplate>> {
    let templates = loading(
        "Fetching .gitignore templates...",
        "Templates fetched",
        || gitignore.templates(),
    )?;

    let detected = detect::gitignore_templates(target)
        .into_iter()
        .filter(|detected| templates.iter().any(|template| template.key == *detected))
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let items = templates
        .iter()
        .map(|template| (template.key.clone(), template.name.clone(), String::new()))
        .collect::<Vec<_>>();

    let selected = multiselect("Which .gitignore templates should be included?")
        .items(&items)
        .initial_values(detected)
        .filter_mode()
        .max_rows(12)
        .interact()?;

    Ok(selected
        .into_iter()
        .filter_map(|key| {
            templates
                .iter()
                .find(|template| template.key == key)
                .cloned()
        })
        .collect())
}

fn choose_license(github: &GitHubClient) -> Result<LicenseTemplate> {
    let licenses = loading("Fetching available licenses...", "Licenses fetched", || {
        github.licenses()
    })?;

    let items = licenses
        .into_iter()
        .map(|license| {
            let label = license.name.clone();
            let hint = license.spdx_id.clone();

            (license, label, hint)
        })
        .collect::<Vec<_>>();

    let selected: LicenseSummary = select("Choose a license")
        .items(&items)
        .filter_mode()
        .max_rows(12)
        .interact()?;

    let license = loading(
        format!("Downloading {}...", selected.name),
        "License downloaded",
        || github.license(&selected),
    )?;

    note(
        format!("{} ({})", license.name, license.spdx_id),
        &license.description,
    )?;

    Ok(license)
}

fn choose_gitignore_mode(target: &Path, enabled: bool) -> Result<WriteMode> {
    if !enabled {
        return Ok(WriteMode::Skip);
    }

    if !target.join(".gitignore").exists() {
        return Ok(WriteMode::Create);
    }

    Ok(select(".gitignore already exists. What should bounds do?")
        .item(
            WriteMode::Merge,
            "Merge",
            "Keep existing rules and add the selected templates",
        )
        .item(WriteMode::Replace, "Replace", "Delete the current contents")
        .item(WriteMode::Skip, "Leave unchanged", "Do not touch the file")
        .interact()?)
}

fn choose_license_mode(target: &Path, enabled: bool) -> Result<WriteMode> {
    if !enabled {
        return Ok(WriteMode::Skip);
    }

    if !target.join("LICENSE").exists() {
        return Ok(WriteMode::Create);
    }

    Ok(select("LICENSE already exists. What should bounds do?")
        .item(
            WriteMode::Skip,
            "Leave unchanged",
            "Keep the current license",
        )
        .item(
            WriteMode::Replace,
            "Replace",
            "Replace it with the selected license",
        )
        .interact()?)
}

fn choose_author() -> Result<String> {
    let detected = git_user_name().unwrap_or_default();
    let mut prompt = input("Copyright holder");

    if !detected.is_empty() {
        prompt = prompt.default_input(&detected);
    }

    Ok(prompt.interact()?)
}

fn git_user_name() -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let name = String::from_utf8(output.stdout).ok()?;
    let name = name.trim();

    (!name.is_empty()).then(|| name.to_owned())
}

fn summary(config: &Config) -> String {
    let mut lines = vec![format!("Directory: {}", config.target.display())];

    if !config.gitignores.is_empty() {
        let templates = config
            .gitignores
            .iter()
            .map(|template| template.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        lines.push(format!(".gitignore: {templates}"));
    }

    if let Some(license) = &config.license {
        lines.push(format!("LICENSE: {} ({})", license.name, license.spdx_id,));
    }

    if let Some(author) = &config.author {
        lines.push(format!("Copyright: {author}"));
    }

    lines.join("\n")
}

fn loading<T>(
    message: impl std::fmt::Display,
    finished: impl std::fmt::Display,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let spinner = spinner();
    spinner.start(message);

    match operation() {
        Ok(value) => {
            spinner.stop(finished);
            Ok(value)
        }

        Err(error) => {
            spinner.error("Request failed");
            Err(error)
        }
    }
}
