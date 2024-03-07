use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid command: {0}")]
    CliError(String),

    #[error("I/O")]
    Io(#[from] std::io::Error),

    #[error("Could not read project file: {toml}")]
    ReadProject {
        toml: toml::de::Error,
        filename: String,
    },

    #[error("No project file in this directory or any parent")]
    NoProject,
}

#[derive(Debug, thiserror::Error)]
#[error("")]
pub struct ReadProject(#[from] toml::de::Error);

impl ReadProject {
    pub fn with_filename(self, filename: &Path) -> Error {
        Error::ReadProject {
            toml: self.0,
            filename: format!("{}", filename.display()),
        }
    }
}
