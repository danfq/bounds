use std::{env, time::Duration};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use ureq::Agent;

use crate::core::model::{LicenseSummary, LicenseTemplate};

const API_URL: &str = "https://api.github.com";
const API_VERSION: &str = "2026-03-10";

#[derive(Clone)]
pub struct GitHubClient {
    agent: Agent,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new() -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build();

        Self {
            agent: config.into(),
            token: env::var("GITHUB_TOKEN").ok(),
        }
    }

    pub fn licenses(&self) -> Result<Vec<LicenseSummary>> {
        self.get_json(&format!("{API_URL}/licenses?per_page=100"))
    }

    pub fn license(&self, license: &LicenseSummary) -> Result<LicenseTemplate> {
        self.get_json(&license.url)
    }

    fn get_json<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut request = self
            .agent
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", API_VERSION)
            .header("User-Agent", concat!("bounds/", env!("CARGO_PKG_VERSION")));

        let authorization = self.token.as_ref().map(|token| format!("Bearer {token}"));

        if let Some(authorization) = authorization.as_deref() {
            request = request.header("Authorization", authorization);
        }

        let mut response = request
            .call()
            .with_context(|| format!("GitHub request failed: {url}"))?;

        response
            .body_mut()
            .read_json::<T>()
            .with_context(|| format!("GitHub returned invalid data: {url}"))
    }
}
