use std::path::Path;

pub fn gitignore_templates(target: &Path) -> Vec<&'static str> {
    let mut detected = Vec::new();

    if target.join("Cargo.toml").exists() {
        detected.push("rust");
    }

    if target.join("package.json").exists() {
        detected.push("node");
    }

    if [
        "astro.config.js",
        "astro.config.mjs",
        "astro.config.cjs",
        "astro.config.ts",
        "astro.config.mts",
        "astro.config.cts",
    ]
    .iter()
    .any(|config| target.join(config).exists())
    {
        detected.push("astro");
    }

    if target.join("go.mod").exists() {
        detected.push("go");
    }

    if target.join("requirements.txt").exists() || target.join("pyproject.toml").exists() {
        detected.push("python");
    }

    detected
}
