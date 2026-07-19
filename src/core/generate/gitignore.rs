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
    let mut known_rules = HashSet::new();
    let mut output = String::new();

    for template in templates {
        let additions = new_rules(&template.source, &mut known_rules);

        if additions.is_empty() {
            continue;
        }

        output.push_str(&format!("# {}\n", template.name));

        for line in additions {
            output.push_str(line);
            output.push('\n');
        }

        output.push('\n');
    }

    output
}

fn merge(existing: &str, templates: &[GitignoreTemplate]) -> String {
    let (mut output, mut known_rules) = compact_existing(existing);

    for template in templates {
        let additions = new_rules(&template.source, &mut known_rules);

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

fn compact_existing(existing: &str) -> (String, HashSet<String>) {
    let mut known_rules = HashSet::new();
    let mut output = String::new();

    for line in existing.trim_end().lines() {
        let line = line.trim_end();
        let trimmed = line.trim_start();

        if !trimmed.is_empty() && !trimmed.starts_with('#') && !known_rules.insert(line.to_owned())
        {
            continue;
        }

        output.push_str(line);
        output.push('\n');
    }

    (output.trim_end().to_owned(), known_rules)
}

fn new_rules<'a>(source: &'a str, known_rules: &mut HashSet<String>) -> Vec<&'a str> {
    source
        .lines()
        .map(str::trim_end)
        .filter(|line| {
            let trimmed = line.trim_start();

            !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && known_rules.insert((*line).to_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{merge, render};
    use crate::core::model::GitignoreTemplate;

    fn template(name: &str, source: &str) -> GitignoreTemplate {
        GitignoreTemplate {
            key: name.to_lowercase(),
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

    #[test]
    fn render_removes_comments_blank_lines_and_duplicate_rules() {
        let templates = [
            template(
                "Astro",
                "# Created by Toptal\n\n### Astro ###\ndist/\n.astro/\n.env\n",
            ),
            template("Node", "# dependencies\nnode_modules/\ndist/\n.env\n"),
        ];

        assert_eq!(
            render(&templates),
            "# Astro\ndist/\n.astro/\n.env\n\n# Node\nnode_modules/\n\n"
        );
    }

    #[test]
    fn render_preserves_escaped_hash_patterns() {
        let templates = [template("Example", "# comment\n\\#file\n")];

        assert_eq!(render(&templates), "# Example\n\\#file\n\n");
    }

    #[test]
    fn merge_removes_rules_already_repeated_in_existing_file() {
        let templates = [template("Astro", ".env\ndist/\n")];

        assert_eq!(
            merge("# Existing\n.env\nnode_modules/\n.env\n", &templates),
            "# Existing\n.env\nnode_modules/\n\n# Astro\ndist/\n"
        );
    }

    #[test]
    fn distinct_gitignore_patterns_are_not_fuzzily_deduplicated() {
        let templates = [template("Paths", "dist/\n/dist/\n**/dist/\n")];

        assert_eq!(render(&templates), "# Paths\ndist/\n/dist/\n**/dist/\n\n");
    }
}
