use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug)]
pub struct Config {
    pub target: PathBuf,

    pub gitignores: Vec<GitignoreTemplate>,
    pub gitignore_mode: WriteMode,

    pub license: Option<LicenseTemplate>,
    pub license_mode: WriteMode,

    pub author: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteMode {
    Create,
    Merge,
    Replace,
    Skip,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GitignoreTemplate {
    pub name: String,
    pub source: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct LicenseSummary {
    pub key: String,
    pub name: String,
    pub spdx_id: String,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LicenseTemplate {
    pub name: String,
    pub spdx_id: String,
    pub description: String,
    pub body: String,
}

impl LicenseTemplate {
    pub fn needs_author(&self) -> bool {
        self.body.contains("[fullname]") || self.body.contains("[name of copyright owner]")
    }
}

#[cfg(test)]
mod tests {
    use super::LicenseTemplate;

    fn license(body: &str) -> LicenseTemplate {
        LicenseTemplate {
            name: "Test license".to_owned(),
            spdx_id: "Test".to_owned(),
            description: "A license used by tests".to_owned(),
            body: body.to_owned(),
        }
    }

    #[test]
    fn detects_supported_author_placeholders() {
        assert!(license("Copyright [fullname]").needs_author());
        assert!(license("Copyright [name of copyright owner]").needs_author());
        assert!(!license("No copyright placeholder").needs_author());
    }
}
