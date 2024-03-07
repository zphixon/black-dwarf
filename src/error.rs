use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid command: {0}")]
    Cli(String),

    #[error("I/O error: {0}")]
    GenericIo(#[from] std::io::Error),

    #[error("I/O error in {io}: {path}")]
    FileIo { io: std::io::Error, path: String },

    #[error("Could not read project file {path}: {toml}")]
    ReadProject { toml: toml::de::Error, path: String },

    #[error("Could not read TOML file {path}: {toml}")]
    GenericToml { toml: toml::de::Error, path: String },

    #[error("No project file in this directory or any parent")]
    NoProject,

    #[error("No config directory")]
    NoConfigDir,

    #[error("No compiler named {name}")]
    NoCompiler { name: String },

    #[error("Many compilers named {name}")]
    ManyCompilers { name: String },

    #[error("Compiler is broken: {why}")]
    CompilerBroken { why: String },
}

impl Error {
    pub fn with_filename(self, filename: &Path) -> Error {
        match self {
            Error::GenericIo(io) => Error::FileIo {
                io,
                path: format!("{}", filename.display()),
            },
            _ => self,
        }
    }
}
