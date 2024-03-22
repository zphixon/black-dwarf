use crate::error::Error;
use std::{ops::Deref, path::Path};

#[derive(Debug)]
pub struct Compiler {
    pub name: String,
    pub inner: CompilerInner,
}

impl Deref for Compiler {
    type Target = CompilerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CompilerInner {
    pub compile_format: Vec<String>,
    pub command: String,
    pub verbose_flag: String,
    pub debug_flag: String,
    pub compile_only_flag: String,
    pub include_path_option: String,
    pub output_option: String,
    pub output_format: String,
}

impl Compiler {
    pub fn compile<S: AsRef<Path>>(
        &self,
        source: &Path,
        include_paths: &[S],
        debug: bool,
        verbose: bool,
    ) -> Result<(), Error> {
        if !source.is_absolute() {
            return Err(Error::Bug(format!(
                "Compiling non-absolute source file {}",
                source.display().to_string()
            )));
        }

        tracing::info!("Compiling {}", source.display());
        let source_file = source.display().to_string();

        let command_format = macros::env_var!(
            "compiler", source_file, "command_format";
            "compiler_command_format";
            &self.compile_format.join(" ")
        );

        let compiler_command = macros::env_var!(
            "compiler", source_file, "command";
            "compiler_command";
            self.command.as_str()
        );

        let compiler_verbose_flag = macros::env_var!(
            "compiler", source_file, "verbose_flag";
            "compiler_verbose_flag";
            self.verbose_flag.as_str()
        );

        let compiler_debug_flag = macros::env_var!(
            "compiler", source_file, "debug_flag";
            "compiler_debug_flag";
            self.debug_flag.as_str()
        );

        let compiler_include_path_option = macros::env_var!(
            "compiler", source_file, "include_path_option";
            "compiler_include_path_option";
            self.include_path_option.as_str()
        );

        let compiler_compile_only_flag = macros::env_var!(
            "compiler", source_file, "compile_only_flag";
            "compiler_compile_only_flag";
            self.compile_only_flag.as_str()
        );

        let compiler_output_option = macros::env_var!(
            "compiler", source_file, "output_option";
            "compiler_output_option";
            self.output_option.as_str()
        );

        let compiler_output_format = macros::env_var!(
            "compiler", source_file, "output_format";
            "compiler_output_format";
            self.output_format.as_str()
        );

        let include_path_separator = macros::env_var!(
            "compiler", source_file, "include_path_separator";
            "compiler_include_path_separator";
            ","
        );

        let compiler_include_paths = macros::env_var!(
            "compiler", source_file, "include_paths";
            "compiler_include_paths";
            &include_paths.iter().map(|s| s.as_ref().display().to_string()).collect::<Vec<_>>().join(&include_path_separator)
        );

        let mut command = Vec::<String>::new();
        for part in command_format.split(" ") {
            match part {
                "%command" => command.push(compiler_command.clone()),
                "%verbose_flag" if verbose => command.push(compiler_verbose_flag.clone()),
                "%verbose_flag" if !verbose => {}
                "%debug_flag" if debug => command.push(compiler_debug_flag.clone()),
                "%debug_flag" if !debug => {}
                "%compile_only_flag" => command.push(compiler_compile_only_flag.clone()),
                "%includes" => {
                    for path in compiler_include_paths.split(&include_path_separator) {
                        if path != "" {
                            command.push(compiler_include_path_option.clone());
                            command.push(path.into());
                        }
                    }
                }
                "%source" => command.push(source.display().to_string()),
                "%output_option" => command.push(compiler_output_option.clone()),
                "%output" => command.push(
                    compiler_output_format.replace(
                        "%source_basename",
                        &source
                            .with_file_name(source.file_stem().ok_or_else(|| {
                                tracing::error!("Cannot not compile file without filename");
                                Error::NoFilename(source.display().to_string())
                            })?)
                            .display()
                            .to_string(),
                    ),
                ),
                _ if part.starts_with("%") => return Err(Error::UnknownSubstitution(part.into())),
                _ => command.push(part.into()),
            }
        }

        tracing::info!("{:?}", command);
        let status = subprocess::Exec::cmd(&command[0])
            .args(&command[1..])
            .join()?;
        if !status.success() {
            Err(Error::CompilationFailed)
        } else {
            Ok(())
        }
    }

    pub fn link_static(&self) -> Result<(), Error> {
        todo!()
    }

    pub fn link_dynamic(&self) -> Result<(), Error> {
        todo!()
    }
}
