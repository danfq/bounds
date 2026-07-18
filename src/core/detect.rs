use std::path::Path;

pub fn gitignore_templates(target: &Path) -> Vec<&'static str> {
    let mut detected = Vec::new();

    if target.join("Cargo.toml").exists() {
        detected.push("Rust");
    }

    if target.join("package.json").exists() {
        detected.push("Node");
    }

    if target.join("go.mod").exists() {
        detected.push("Go");
    }

    if target.join("requirements.txt").exists() || target.join("pyproject.toml").exists() {
        detected.push("Python");
    }

    detected
}
