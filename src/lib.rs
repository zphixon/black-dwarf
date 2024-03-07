use std::{collections::HashMap, path::PathBuf};

pub mod error;
pub mod util;

pub const PROJECT_FILENAME: &str = "BD.toml";

#[derive(serde::Deserialize, Debug)]
pub struct Project {
    pub project: ProjectMeta,
    pub sources: Vec<SourceGroup>,

    #[serde(flatten)]
    pub rest: HashMap<String, toml::Value>,
}

impl Project {
    pub fn unused_keys(&self) -> Vec<String> {
        self.rest
            .keys()
            .map(|key| key.clone())
            .chain(
                self.project
                    .unused_keys()
                    .into_iter()
                    .map(|key| format!("project.{}", key)),
            )
            .chain(
                self.sources
                    .iter()
                    .flat_map(|source| source.unused_keys())
                    .map(|key| format!("[[sources]].{}", key)),
            )
            .collect()
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,

    #[serde(flatten)]
    pub rest: HashMap<String, toml::Value>,
}

impl ProjectMeta {
    pub fn unused_keys(&self) -> Vec<String> {
        self.rest.keys().map(|key| key.clone()).collect()
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct SourceGroup {
    pub files: Vec<PathBuf>,

    #[serde(default = "Vec::new")]
    pub compile: Vec<String>,

    #[serde(default = "Vec::new")]
    pub link: Vec<String>,

    #[serde(flatten)]
    pub rest: HashMap<String, toml::Value>,
}

impl SourceGroup {
    pub fn unused_keys(&self) -> Vec<String> {
        self.rest.keys().map(|key| key.clone()).collect()
    }
}
