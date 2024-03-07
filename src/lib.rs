use std::{collections::HashMap, path::PathBuf};

pub mod error;
pub mod util;

pub const PROJECT_FILENAME: &str = "C.toml";

#[derive(macros::UnusedKeys, serde::Deserialize, Debug)]
pub struct Project {
    pub project: ProjectMeta,
    pub sources: Vec<SourceGroup>,

    #[serde(flatten)]
    #[unused]
    pub rest: HashMap<String, toml::Value>,
}

#[derive(macros::UnusedKeys, serde::Deserialize, Debug)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,

    #[serde(flatten)]
    #[unused]
    pub rest: HashMap<String, toml::Value>,
}

#[derive(macros::UnusedKeys, serde::Deserialize, Debug)]
pub struct SourceGroup {
    pub files: Vec<PathBuf>,

    #[serde(default = "Vec::new")]
    pub compile: Vec<String>,

    #[serde(default = "Vec::new")]
    pub link: Vec<String>,

    #[serde(flatten)]
    #[unused]
    pub rest: HashMap<String, toml::Value>,
}

pub trait UnusedKeys {
    fn unused_keys(&self) -> Vec<String>;
}

impl<T> UnusedKeys for Vec<T> where T: UnusedKeys {
    fn unused_keys(&self) -> Vec<String> {
        self.iter().flat_map(|t| t.unused_keys().into_iter()).collect()
    }
}

impl UnusedKeys for PathBuf {
    fn unused_keys(&self) -> Vec<String> {
        vec![]
    }
}

impl UnusedKeys for String {
    fn unused_keys(&self) -> Vec<String> {
        vec![]
    }
}
