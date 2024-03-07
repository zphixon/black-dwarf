use std::{collections::HashMap, path::PathBuf};

use error::Error;

pub mod compiler;
pub mod error;
pub mod project;

pub const ENV_VAR_PREFIX: &str = "CR";
pub const PROJECT_FILENAME: &str = "C.toml";
pub const CONFIG_DIR_NAME: &str = "cretaceous";
pub const COMPILERS_FILENAME: &str = "compilers.toml";

macro_rules! env_var {
    (
        $part1:expr $(, $part2:expr)* $(,)?
        ; $part3:expr $(, $part4:expr)* $(,)?
        $(; $part5:expr $(, $part6:expr)* $(,)?)* $(;)?
    ) => {
        env_var!($part1 $(, $part2)*);
        env_var!($part3 $(, $part4)*);
        $(env_var!($part5 $(, $part6)*);)*
    };

    (
        $part1:expr $(, $part2:expr)* $(,)?
    ) => {
        print!(stringify!($part1));
        $(print!(stringify!($part2));)*
        println!();
    };
}

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

pub fn get_env_or<const N: usize, S: AsRef<str>>(parts: [S; N], or: &str) -> Option<String> {
    assert!(!parts.is_empty());
    let mut name = String::from(ENV_VAR_PREFIX);
    for part in parts {
        name.push('_');
        name.push_str(&part.as_ref().to_uppercase());
    }
    std::env::var(&name).ok().or(Some(or.into()))
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
    Some(PathBuf::from(".").join("dist").canonicalize().unwrap())
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
        .map_err(Error::from)
        .map_err(|err| err.with_filename(compilers_path.as_path()))?;

    #[derive(serde::Deserialize)]
    struct CompilerList {
        compiler: Vec<compiler::Compiler>,
    }

    let CompilerList {
        compiler: compilers,
    } = toml::from_str(&compilers_str).map_err(|toml| Error::GenericToml {
        toml,
        path: format!("{}", compilers_path.display()),
    })?;

    #[cfg(target_os = "linux")]
    let default_compiler_name = "gcc";

    let mut filtered = compilers
        .into_iter()
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
