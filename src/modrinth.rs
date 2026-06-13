use crate::csv::CsvMeta;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;

const PROJECTS_URL: &str = "https://api.modrinth.com/v2/projects";
const VERSIONS_URL: &str = "https://api.modrinth.com/v2/project/%s/version";

#[derive(Deserialize, Debug)]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub client_side: String,
    pub server_side: String,
}

#[derive(Debug, Deserialize)]
pub struct ModVersion {
    pub id: String,
    pub version_number: String,
    pub version_type: String,
    pub dependencies: Vec<ModDependency>,
    pub files: Vec<VersionFile>,
}

#[derive(Debug, Deserialize)]
pub struct ModDependency {
    pub project_id: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionFile {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub size: i64,
}

#[derive(Debug, Deserialize)]
pub struct Hashes {
    pub sha1: String,
}

static HTTP_CLIENT: OnceLock<ureq::Agent> = OnceLock::new();

pub fn get_client() -> &'static ureq::Agent {
    HTTP_CLIENT.get_or_init(|| {
        let config = ureq::Agent::config_builder()
            .user_agent(format!(
                "{}/{} ({})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_REPOSITORY")
            ))
            .timeout_global(Some(Duration::from_secs(30)))
            .build();

        ureq::Agent::new_with_config(config)
    })
}

pub fn fetch_projects<T: AsRef<str> + Serialize>(ids: &[T]) -> anyhow::Result<Vec<Project>> {
    Ok(get_client()
        .get(PROJECTS_URL)
        .query("ids", &serde_json::to_string(ids).unwrap())
        .call()?
        .body_mut()
        .read_json::<Vec<Project>>()?)
}

pub fn fetch_versions_for_project(project_id: &str, meta: &CsvMeta) -> Vec<ModVersion> {
    let url = VERSIONS_URL.replace("%s", project_id);
    get_client()
        .get(&url)
        .query("loaders", format!("[\"{}\"]", meta.loader))
        .query("game_versions", format!("[\"{}\"]", meta.minecraft))
        .call()
        .unwrap()
        .body_mut()
        .read_json::<Vec<ModVersion>>()
        .unwrap()
}
