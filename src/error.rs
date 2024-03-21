use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid command: {0}")]
    Cli(String),

    #[error("I/O error: {0}")]
    GenericIo(#[from] std::io::Error),

    #[error("I/O error: {path}: {io}")]
    FileIo { io: std::io::Error, path: String },

    #[error("Could not read project file {path}: {toml}")]
    ReadProject { toml: toml::de::Error, path: String },

    #[error("Could not read TOML file {path}: {toml}")]
    GenericToml { toml: toml::de::Error, path: String },

    #[error("No project file in this directory or any parent")]
    NoProject,

    #[error("Project file is not in a directory")]
    NoProjectDir,

    #[error("No config directory")]
    NoConfigDir,

    #[error("Substitution was not valid: {0}")]
    UnknownSubstitution(String),

    #[error("File does not have a name: {0}")]
    NoFilename(String),

    #[error("No compiler named {name}")]
    NoCompiler { name: String },

    #[error("Many compilers named {name}")]
    ManyCompilers { name: String },

    #[error("Compiler is broken: {why}")]
    CompilerBroken { why: String },

    #[error("Could not run compiler: {0}")]
    CouldNotRunCompiler(#[from] subprocess::PopenError),

    #[error("Compilation failed")]
    CompilationFailed,

    #[error("No such build target: {0}")]
    NoSuchBuildTarget(String),
}

impl Error {
    pub fn file_io<P: AsRef<Path>>(io: std::io::Error, path: P) -> Error {
        Error::FileIo {
            io,
            path: format!("{}", path.as_ref().display()),
        }
    }
}
