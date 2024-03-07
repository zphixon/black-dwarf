use std::path::Path;

use crate::get_env_or;

#[derive(serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Compiler {
    pub name: String,
    pub path: Option<String>,
    pub include_path_format: String,
    pub link_path_format: String,
}

impl Compiler {
    pub fn compile(&self, source: &Path, headers: &[&str]) {
        let name = &self.name;
    }
}
