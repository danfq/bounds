use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use anyhow::{Context, Result};

use crate::core::model::{Config, GitignoreTemplate, WriteMode};

pub fn write(config: &Config) -> Result<Option<PathBuf>> {
    let path = config.target.join(".gitignore");

    match config.gitignore_mode {
        WriteMode::Skip => return Ok(None),

        WriteMode::Create => {
            let contents = render(&config.gitignores);

            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .with_context(|| format!("failed to create {}", path.display()))?;

            file.write_all(contents.as_bytes())?;
        }

        WriteMode::Replace => {
            fs::write(&path, render(&config.gitignores))
                .with_context(|| format!("failed to replace {}", path.display()))?;
        }

        WriteMode::Merge => {
            let existing = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;

            let merged = merge(&existing, &config.gitignores);

            fs::write(&path, merged)
                .with_context(|| format!("failed to update {}", path.display()))?;
        }
    }

    Ok(Some(path))
}

fn render(templates: &[GitignoreTemplate]) -> String {
    let mut output = String::new();

    for template in templates {
        output.push_str(&format!("# {}\n", template.name));
        output.push_str(template.source.as_str().trim());
        output.push_str("\n\n");
    }

    output
}

fn merge(existing: &str, templates: &[GitignoreTemplate]) -> String {
    let mut known_rules = existing
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect::<HashSet<_>>();

    let mut output = existing.trim_end().to_owned();

    for template in templates {
        let additions = template
            .source
            .as_str()
            .lines()
            .map(str::trim)
            .filter(|line| {
                !line.is_empty() && !line.starts_with('#') && known_rules.insert((*line).to_owned())
            })
            .collect::<Vec<_>>();

        if additions.is_empty() {
            continue;
        }

        if !output.is_empty() {
            if !output.ends_with('\n') {
                output.push_str("\n\n");
            } else if !output.ends_with("\n\n") {
                output.push('\n');
            }
        }

        output.push_str(&format!("# {}\n", template.name));

        for line in additions {
            output.push_str(line);
            output.push('\n');
        }
    }

    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{merge, render};
    use crate::core::model::GitignoreTemplate;

    fn template(name: &str, source: &str) -> GitignoreTemplate {
        GitignoreTemplate {
            name: name.to_owned(),
            source: source.to_owned(),
        }
    }

    #[test]
    fn render_labels_and_separates_templates() {
        let templates = [
            template("Rust", "\ntarget/\n"),
            template("Node", "node_modules/\n"),
        ];

        assert_eq!(
            render(&templates),
            "# Rust\ntarget/\n\n# Node\nnode_modules/\n\n"
        );
    }

    #[test]
    fn merge_keeps_existing_rules_and_adds_only_new_rules() {
        let templates = [
            template("Rust", "target/\nCargo.lock\n"),
            template("Node", "target/\nnode_modules/\n"),
        ];

        assert_eq!(
            merge("# Existing\ntarget/\n", &templates),
            "# Existing\ntarget/\n\n# Rust\nCargo.lock\n\n# Node\nnode_modules/\n"
        );
    }

    #[test]
    fn merge_into_empty_file_has_no_leading_blank_lines() {
        let templates = [template("Rust", "target/\n")];

        assert_eq!(merge("", &templates), "# Rust\ntarget/\n");
    }
}
