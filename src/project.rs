use crate::UnusedKeys;
use std::{collections::HashMap, path::PathBuf};

#[derive(macros::UnusedKeys, serde::Deserialize, Debug)]
pub struct Project {
    pub project: ProjectMeta,
    pub sources: Vec<SourceGroup>,
    pub targets: Vec<Target>,

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
    pub name: String,
    pub files: Vec<PathBuf>,

    #[serde(default = "Vec::new")]
    pub headers: Vec<String>,

    #[serde(default = "Vec::new")]
    pub libraries: Vec<String>,

    #[serde(flatten)]
    #[unused]
    pub rest: HashMap<String, toml::Value>,
}

#[derive(macros::UnusedKeys, serde::Deserialize, Debug)]
pub struct Target {
    pub name: String,
    pub sources: Vec<String>,

    #[serde(flatten)]
    #[unused]
    pub rest: HashMap<String, toml::Value>,
}
