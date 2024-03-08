use std::{collections::HashMap, path::PathBuf};

use error::Error;

pub mod compiler;
pub mod error;
pub mod project;

pub const ENV_VAR_PREFIX: &str = "CR";
pub const PROJECT_FILENAME: &str = "C.toml";
pub const CONFIG_DIR_NAME: &str = "cretaceous";
pub const COMPILERS_FILENAME: &str = "compilers.toml";
pub const REPLACE_DEFAULT: &str = "%default";

pub trait UnusedKeys {
    fn unused_keys(&self) -> Vec<String>;
}

impl<T> UnusedKeys for Vec<T>
where
    T: UnusedKeys,
{
    fn unused_keys(&self) -> Vec<String> {
        self.iter()
            .flat_map(|t| t.unused_keys().into_iter())
            .collect()
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

pub fn get_env_or<S: AsRef<str>>(vars_in_parts: &[&[S]], or: &str) -> String {
    for var_parts in vars_in_parts {
        let mut name = String::new();
        for part in var_parts.iter() {
            name.push_str(&part.as_ref().to_uppercase());
        }

        // SAFETY: We are transforming the string to ascii
        unsafe {
            for byte in name.as_bytes_mut() {
                if !byte.is_ascii_alphanumeric() {
                    *byte = b'_';
                }
                debug_assert!(*byte == b'_' || byte.is_ascii_alphanumeric());
            }
        }

        tracing::trace!("Checking {}", name);
        if let Some(value) = std::env::var(&name).ok() {
            return if value.contains(REPLACE_DEFAULT) {
                let new_value = value.replace("%default", or);
                tracing::debug!(
                    "Using {}={:?} (substituted from {:?})",
                    name,
                    new_value,
                    value
                );
                new_value
            } else {
                tracing::debug!("Using {}={:?}", name, value);
                value
            };
        }
    }

    tracing::debug!("Using default: {:?}", or);
    or.into()
}

pub fn find_project_file_from_current_dir() -> Result<PathBuf, Error> {
    let mut dir = std::env::current_dir()?.canonicalize()?;

    loop {
        for item in dir.read_dir()? {
            let item = item?;
            if item.file_name().to_string_lossy() == crate::PROJECT_FILENAME {
                return Ok(item.path());
            }
        }

        if !dir.pop() {
            break;
        }
    }

    Err(Error::NoProject)
}

#[cfg(feature = "dev")]
pub fn config_dir() -> Option<PathBuf> {
    Some(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("dist")
            .canonicalize()
            .unwrap(),
    )
}

#[cfg(not(feature = "dev"))]
pub fn config_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(CONFIG_DIR_NAME))
}

pub fn compilers_file() -> Option<PathBuf> {
    Some(config_dir()?.join(COMPILERS_FILENAME))
}

pub fn default_compiler() -> Result<compiler::Compiler, Error> {
    let compilers_path = compilers_file().ok_or_else(|| Error::NoConfigDir)?;
    let compilers_str = std::fs::read_to_string(compilers_path.as_path())
        .map_err(|io| Error::file_io(io, compilers_path.as_path()))?;

    let compilers = toml::from_str::<HashMap<String, compiler::CompilerInner>>(&compilers_str)
        .map_err(|toml| Error::GenericToml {
            toml,
            path: compilers_path.display().to_string(),
        })?;

    #[cfg(target_os = "linux")]
    let default_compiler_name = "gcc";

    let mut filtered = compilers
        .into_iter()
        .map(|(name, inner)| compiler::Compiler { name, inner })
        .filter(|compiler| compiler.name == default_compiler_name)
        .collect::<Vec<_>>();

    match filtered.len() {
        0 => Err(Error::NoCompiler {
            name: default_compiler_name.into(),
        }),

        2.. => Err(Error::ManyCompilers {
            name: default_compiler_name.into(),
        }),

        1 => Ok(filtered.pop().unwrap()),
    }
}
