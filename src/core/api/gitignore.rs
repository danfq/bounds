use std::{collections::HashMap, time::Duration};

use anyhow::{Context, Result};
use serde::Deserialize;
use ureq::Agent;

use crate::core::model::GitignoreTemplate;

const API_URL: &str = "https://www.toptal.com/developers/gitignore/api";

#[derive(Clone)]
pub struct GitignoreClient {
    agent: Agent,
}

#[derive(Deserialize)]
struct ApiTemplate {
    key: String,
    name: String,
    contents: String,
}

impl GitignoreClient {
    pub fn new() -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build();

        Self {
            agent: config.into(),
        }
    }

    pub fn templates(&self) -> Result<Vec<GitignoreTemplate>> {
        let url = format!("{API_URL}/list?format=json");
        let mut response = self
            .agent
            .get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", concat!("bounds/", env!("CARGO_PKG_VERSION")))
            .call()
            .with_context(|| format!(".gitignore template request failed: {url}"))?;

        let templates = response
            .body_mut()
            .read_json::<HashMap<String, ApiTemplate>>()
            .with_context(|| format!(".gitignore template service returned invalid data: {url}"))?;

        let mut templates = templates
            .into_values()
            .map(|template| GitignoreTemplate {
                key: template.key,
                name: template.name,
                source: template.contents,
            })
            .collect::<Vec<_>>();

        templates.sort_unstable_by_key(|template| template.name.to_lowercase());

        Ok(templates)
    }
}
