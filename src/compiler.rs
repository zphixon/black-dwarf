use crate::{
    error::Error,
    project::{Project, Target},
};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

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
    pub compile_command: String,
    pub compile_verbose_flag: String,
    pub compile_debug_flag: String,
    pub compile_only_flag: String,
    pub compile_include_path_option: String,
    pub compile_output_option: String,
    pub compile_output_format: String,

    pub dynamic_link_format: Vec<String>,
    pub binary_link_format: Vec<String>,
    pub link_command: String,
    pub dynamic_link_flag: String,
    pub link_verbose_flag: String,
    pub link_debug_flag: String,
    pub link_library_path_option: String,
    pub link_output_option: String,
    pub dynamic_link_output_format: String,
    pub link_option: String,

    pub archive_command: String,
    pub archive_format: Vec<String>,
    pub archive_output_format: String,
    pub archive_verbose_flag: String,
    pub archive_flag: String,
}

const PATH_SEPARATOR: &str = ",";

impl Compiler {
    fn short_source_path(&self, project: &Project, source_path: &Path) -> Result<String, Error> {
        Ok(source_path
            .strip_prefix(&project.dir)
            .map_err(|_| {
                Error::Bug(format!(
                    "Source file {} not in project dir {}",
                    source_path.display(),
                    project.dir.display(),
                ))
            })?
            .display()
            .to_string())
    }

    pub fn compile_single_file<S: AsRef<Path>>(
        &self,
        project: &Project,
        source_path: &Path,
        include_paths: &[S],
        debug: bool,
        verbose: bool,
        dry_run: bool,
    ) -> Result<(), Error> {
        if !source_path.is_absolute() {
            return Err(Error::Bug(format!(
                "Compiling non-absolute source file {}",
                source_path.display().to_string()
            )));
        }

        let short_source_path = self.short_source_path(project, source_path)?;
        tracing::info!("Compiling {}", short_source_path);

        let command_format = self.resolve_compile_command_format(&short_source_path);
        let compiler_command = self.resolve_compile_command(&short_source_path);
        let compiler_verbose_flag = self.resolve_compiler_verbose_flag(&short_source_path);
        let compiler_debug_flag = self.resolve_compiler_debug_flag(&short_source_path);
        let compiler_include_path_option =
            self.resolve_compiler_include_path_option(&short_source_path);
        let compiler_compile_only_flag =
            self.resolve_compiler_compile_only_flag(&short_source_path);
        let compiler_output_option = self.resolve_compiler_output_option(&short_source_path);
        let compiler_include_paths = self.resolve_include_paths(&short_source_path, include_paths);

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
                    for path in compiler_include_paths.split(PATH_SEPARATOR) {
                        if path != "" {
                            command.push(compiler_include_path_option.clone());
                            command.push(path.into());
                        }
                    }
                }
                "%source" => command.push(source_path.display().to_string()),
                "%output_option" => command.push(compiler_output_option.clone()),
                "%output" => command.push(
                    self.compile_output_filename(&short_source_path, source_path)?
                        .display()
                        .to_string(),
                ),
                _ if part.starts_with("%") => return Err(Error::UnknownSubstitution(part.into())),
                _ => command.push(part.into()),
            }
        }

        tracing::info!("{:?}", command);
        if dry_run {
            tracing::debug!("Skipping due to --dry-run");
            return Ok(());
        }
        let status = subprocess::Exec::cmd(&command[0])
            .args(&command[1..])
            .join()?;
        if !status.success() {
            Err(Error::CompilationFailed)
        } else {
            Ok(())
        }
    }

    fn compile_output_filename(
        &self,
        short_source_path: &String,
        source_path: &Path,
    ) -> Result<PathBuf, Error> {
        // hnngn wh
        Ok(PathBuf::from(
            self.resolve_compiler_output_format(short_source_path)
                .replace(
                    "%source_basename",
                    &source_path
                        .with_file_name(source_path.file_stem().ok_or_else(|| {
                            tracing::error!("Cannot not compile file without filename");
                            Error::NoFilename(source_path.display().to_string())
                        })?)
                        .display()
                        .to_string(),
                ),
        ))
    }

    fn resolve_compile_command(&self, source_file: &String) -> String {
        macros::env_var!(
            doc "Command used to compile a source file"
            "compiler", source_file, "command";
            "compiler_command";
            self.compile_command.as_str()
        )
    }

    fn resolve_compile_command_format(&self, source_file: &String) -> String {
        macros::env_var!(
            doc "Format string used to build the command which will compile a source file"
            "compiler", source_file, "command_format";
            "compiler_command_format";
            &self.compile_format.join(" ")
        )
    }

    fn resolve_compiler_verbose_flag(&self, source_file: &String) -> String {
        macros::env_var!(
            doc "Flag which will cause the compiler to output verbose information"
            "compiler", source_file, "verbose_flag";
            "compiler_verbose_flag";
            self.compile_verbose_flag.as_str()
        )
    }

    fn resolve_compiler_debug_flag(&self, source_file: &String) -> String {
        macros::env_var!(
            doc "Flag which will cause the compiler to include debug symbols"
            "compiler", source_file, "debug_flag";
            "compiler_debug_flag";
            self.compile_debug_flag.as_str()
        )
    }

    fn resolve_compiler_include_path_option(&self, source_file: &String) -> String {
        macros::env_var!(
            doc "Option used to specify a path to search for header files"
            "compiler", source_file, "include_path_option";
            "compiler_include_path_option";
            self.compile_include_path_option.as_str()
        )
    }

    fn resolve_compiler_compile_only_flag(&self, source_file: &String) -> String {
        macros::env_var!(
            doc "Flag used to compile a source file without linking it"
            "compiler", source_file, "compile_only_flag";
            "compiler_compile_only_flag";
            self.compile_only_flag.as_str()
        )
    }

    fn resolve_compiler_output_option(&self, source_file: &String) -> String {
        macros::env_var!(
            doc "Option used to specify the output location of a compiled source file"
            "compiler", source_file, "output_option";
            "compiler_output_option";
            self.compile_output_option.as_str()
        )
    }

    fn resolve_compiler_output_format(&self, source_file: &String) -> String {
        macros::env_var!(
            doc "Format that a compiled source file should take"
            "compiler", source_file, "output_format";
            "compiler_output_format";
            self.compile_output_format.as_str()
        )
    }

    fn resolve_include_paths<S: AsRef<Path>>(
        &self,
        source_file: &String,
        include_paths: &[S],
    ) -> String {
        macros::env_var!(
            doc "Comma-separated list of paths to search for header files"
            "compiler", source_file, "include_paths";
            "compiler_include_paths";
            &include_paths
                .iter()
                .map(|s| s.as_ref().display().to_string())
                .collect::<Vec<_>>()
                .join(PATH_SEPARATOR)
        )
    }

    fn resolve_link_command(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Command used to link a dynamic library"
            "linker", target_name, "command";
            "linker_command";
            self.link_command.as_str()
        )
    }

    fn resolve_dynamic_link_command_format(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Format string used to build the command which will link a dynamic library"
            "dynamic_linker", target_name, "command_format";
            "dynamic_linker_command_format";
            &self.dynamic_link_format.join(" ")
        )
    }

    fn resolve_binary_link_command_format(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Format string used to build the command which will link a binary"
            "linker", target_name, "command_format";
            "linker_command_format";
            &self.binary_link_format.join(" ")
        )
    }

    fn resolve_linker_verbose_flag(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Flag which will cause the linker to output verbose information"
            "linker", target_name, "verbose_flag";
            "linker_verbose_flag";
            self.link_verbose_flag.as_str()
        )
    }

    fn resolve_linker_debug_flag(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Flag which will cause the linker to include debug symbols"
            "linker", target_name, "debug_flag";
            "linker_debug_flag";
            self.link_debug_flag.as_str()
        )
    }

    fn resolve_linker_link_path_option(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Option used to specify a path to search for library files"
            "linker", target_name, "library_path_option";
            "linker_library_path_option";
            self.link_library_path_option.as_str()
        )
    }

    fn resolve_linker_output_option(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Option used to specify the output location of a linked target"
            "linker", target_name, "output_option";
            "linker_output_option";
            self.link_output_option.as_str()
        )
    }

    fn resolve_linker_dynamic_output_format(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Format that a linked dynamic target should take"
            "linker", target_name, "dynamic_output_format";
            "linker_dynamic_output_format";
            self.dynamic_link_output_format.as_str()
        )
    }

    fn resolve_linker_dynamic_link_flag(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Flag that will cause the linker to output a dynamic library"
            "linker_dynamic", target_name, "link_flag";
            "linker_dynamic_link_flag";
            self.dynamic_link_flag.as_str()
        )
    }

    fn resolve_linker_link_option(&self, target_name: &String) -> String {
        macros::env_var!(
            doc "Option that will include a libary in a link command"
            "linker", target_name, "link_option";
            "linker_link_option";
            self.link_option.as_str()
        )
    }

    fn resolve_linker_paths<S: AsRef<Path>>(
        &self,
        target_name: &String,
        link_paths: &[S],
    ) -> String {
        macros::env_var!(
            doc "Comma-separated list of paths to search for library files"
            "linker", target_name, "link_paths";
            "linker_link_paths";
            &link_paths
                .iter()
                .map(|s| s.as_ref().display().to_string())
                .collect::<Vec<_>>()
                .join(PATH_SEPARATOR)
        )
    }

    pub fn compile_target(
        &self,
        project: &Project,
        target: &Target,
        debug: bool,
        verbose: bool,
        dry_run: bool,
    ) -> Result<(), Error> {
        for source in target.sources.iter() {
            let mut include_paths = vec![target.path.as_path()];
            for need in target.needs.iter() {
                include_paths.push(
                    project
                        .target
                        .get(need.as_str())
                        .ok_or_else(|| Error::Bug(format!("Resolved project had unknown target")))?
                        .path
                        .as_path(),
                );
            }

            self.compile_single_file(project, source, &include_paths, debug, verbose, dry_run)?;
        }

        Ok(())
    }

    fn resolve_archive_command(&self, target_name: &String) -> String {
        macros::env_var!(
            "archive", target_name, "command";
            "archive_command";
            self.archive_command.as_str()
        )
    }

    fn resolve_archive_format(&self, target_name: &String) -> String {
        macros::env_var!(
            "archive", target_name, "format";
            "archive_format";
            &self.archive_format.join(" ")
        )
    }

    fn resolve_archive_output_format(&self, target_name: &String) -> String {
        macros::env_var!(
            "archive", target_name, "output_format";
            "archive_output_format";
            self.archive_output_format.as_str()
        )
    }

    fn resolve_archive_verbose_flag(&self, target_name: &String) -> String {
        macros::env_var!(
            "archive", target_name, "verbose_flag";
            "archive_verbose_flag";
            self.archive_verbose_flag.as_str()
        )
    }

    fn resolve_archive_flag(&self, target_name: &String) -> String {
        macros::env_var!(
            "archive", target_name, "flag";
            "archive_flag";
            self.archive_flag.as_str()
        )
    }

    pub fn create_archive(
        &self,
        project: &Project,
        target: &Target,
        verbose: bool,
        dry_run: bool,
    ) -> Result<(), Error> {
        tracing::info!("Archiving target {}", target.name);

        let archive_command = self.resolve_archive_command(&target.name);
        let archive_format = self.resolve_archive_format(&target.name);
        let archive_output_format = self.resolve_archive_output_format(&target.name);
        let archive_verbose_flag = self.resolve_archive_verbose_flag(&target.name);
        let archive_flag = self.resolve_archive_flag(&target.name);

        //"%verbose_flag" if verbose => command.push(archive_verbose_flag.clone()), //"%verbose_flag" if !verbose => {}
        let mut replace_objects = String::new();
        for source_path in target.sources.iter() {
            let short_source_path = self.short_source_path(project, source_path)?;
            replace_objects.push_str(
                &self
                    .compile_output_filename(&short_source_path, source_path)?
                    .display()
                    .to_string(),
            );
            replace_objects.push(' ');
        }

        let command = archive_format
            .replace("%command", &archive_command)
            .replace("%objects", &replace_objects)
            .replace("%archive_flag", &archive_flag)
            .replace(
                "%output",
                &target
                    .path
                    .join(&archive_output_format.replace("%target", &target.name))
                    .display()
                    .to_string(),
            );

        let command = if verbose {
            command.replace("%verbose_flag", &archive_verbose_flag)
        } else {
            command.replace("%verbose_flag", "")
        };

        let command_vec = command.split_whitespace().collect::<Vec<_>>();

        tracing::info!("{:?}", command_vec);
        if dry_run {
            tracing::debug!("Skipping due to --dry-run");
            return Ok(());
        }
        let status = subprocess::Exec::cmd(&command_vec[0])
            .args(&command_vec[..])
            .join()?;
        if !status.success() {
            Err(Error::ArchiveFailed)
        } else {
            Ok(())
        }
    }

    pub fn link_dynamic(
        &self,
        project: &Project,
        target: &Target,
        verbose: bool,
        debug: bool,
        dry_run: bool,
    ) -> Result<(), Error> {
        tracing::info!("Linking dynamic target {}", target.name);

        let mut link_paths = vec![target.path.as_path()];
        for need in target.needs.iter() {
            link_paths.push(
                project
                    .target
                    .get(need.as_str())
                    .ok_or_else(|| Error::Bug(format!("Resolved project had unknown target")))?
                    .path
                    .as_path(),
            );
        }
        let link_paths = self.resolve_linker_paths(&target.name, &link_paths);

        let linker_command = self.resolve_link_command(&target.name);
        let linker_verbose_flag = self.resolve_linker_verbose_flag(&target.name);
        let linker_debug_flag = self.resolve_linker_debug_flag(&target.name);
        let linker_dynamic_link_flag = self.resolve_linker_dynamic_link_flag(&target.name);
        let linker_output_option = self.resolve_linker_output_option(&target.name);
        let linker_dynamic_output_format = self.resolve_linker_dynamic_output_format(&target.name);
        let link_path_option = self.resolve_linker_link_path_option(&target.name);
        let command_format = self.resolve_dynamic_link_command_format(&target.name);

        let mut command = Vec::<String>::new();
        for part in command_format.split(" ") {
            match part {
                "%command" => command.push(linker_command.clone()),
                "%verbose_flag" if verbose => command.push(linker_verbose_flag.clone()),
                "%verbose_flag" if !verbose => {}
                "%debug_flag" if debug => command.push(linker_debug_flag.clone()),
                "%debug_flag" if !debug => {}
                "%dynamic_link_flag" => command.push(linker_dynamic_link_flag.clone()),
                "%objects" => {
                    for source_path in target.sources.iter() {
                        let short_source_path = self.short_source_path(project, source_path)?;
                        command.push(
                            self.compile_output_filename(&short_source_path, source_path)?
                                .display()
                                .to_string(),
                        );
                    }
                }
                "%link_paths" => {
                    for path in link_paths.split(PATH_SEPARATOR) {
                        if path != "" {
                            command.push(link_path_option.clone());
                            command.push(path.into());
                        }
                    }
                }
                "%links" => {
                    for need in target.needs.iter() {
                        command.push(self.resolve_linker_link_option(need));
                        command.push(need.clone());
                    }
                }
                "%output_option" => command.push(linker_output_option.clone()),
                "%output" => {
                    command.push(
                        target
                            .path
                            .join(linker_dynamic_output_format.replace("%target", &target.name))
                            .display()
                            .to_string(),
                    );
                }
                _ if part.starts_with("%") => return Err(Error::UnknownSubstitution(part.into())),
                _ => command.push(part.into()),
            }
        }

        tracing::info!("{:?}", command);
        if dry_run {
            tracing::debug!("Skipping due to --dry-run");
            return Ok(());
        }
        let status = subprocess::Exec::cmd(&command[0])
            .args(&command[1..])
            .join()?;
        if !status.success() {
            Err(Error::LinkFailed)
        } else {
            Ok(())
        }
    }

    pub fn link_binary(
        &self,
        project: &Project,
        target: &Target,
        verbose: bool,
        debug: bool,
        dry_run: bool,
    ) -> Result<(), Error> {
        tracing::info!("Linking binary target {}", target.name);

        let mut link_paths = vec![target.path.as_path()];
        for need in target.needs.iter() {
            link_paths.push(
                project
                    .target
                    .get(need.as_str())
                    .ok_or_else(|| Error::Bug(format!("Resolved project had unknown target")))?
                    .path
                    .as_path(),
            );
        }
        let link_paths = self.resolve_linker_paths(&target.name, &link_paths);

        let linker_command = self.resolve_link_command(&target.name);
        let linker_verbose_flag = self.resolve_linker_verbose_flag(&target.name);
        let linker_debug_flag = self.resolve_linker_debug_flag(&target.name);
        let linker_output_option = self.resolve_linker_output_option(&target.name);
        let link_path_option = self.resolve_linker_link_path_option(&target.name);
        let command_format = self.resolve_binary_link_command_format(&target.name);

        let mut replace_objects = String::new();
        for source_path in target.sources.iter() {
            let short_source_path = self.short_source_path(project, source_path)?;
            replace_objects.push_str(
                &self
                    .compile_output_filename(&short_source_path, source_path)?
                    .display()
                    .to_string(),
            );
            replace_objects.push(' ');
        }

        let mut replace_link_paths = String::new();
        for path in link_paths.split(PATH_SEPARATOR) {
            if path != "" {
                replace_link_paths.push_str(&link_path_option);
                replace_link_paths.push_str(&path);
                replace_link_paths.push(' ');
            }
        }

        let mut replace_links = String::new();
        for need in target.needs.iter() {
            replace_links.push_str(&self.resolve_linker_link_option(need));
            replace_links.push_str(&need);
            replace_links.push(' ');
        }

        let command = command_format
            .replace("%command", &linker_command)
            .replace("%objects", &replace_objects)
            .replace("%link_paths", &replace_link_paths)
            .replace("%links", &replace_links)
            .replace("%output_option", &linker_output_option)
            .replace(
                "%output",
                &target.path.join(&target.name).display().to_string(),
            );

        let command = if verbose {
            command.replace("%verbose_flag", &linker_verbose_flag)
        } else {
            command.replace("%verbose_flag", "")
        };
        let command = if debug {
            command.replace("%debug_flag", &linker_debug_flag)
        } else {
            command.replace("%debug_flag", "")
        };

        let command_vec = command.split_whitespace().collect::<Vec<_>>();
        tracing::info!("{:?}", command_vec);
        if dry_run {
            tracing::debug!("Skipping due to --dry-run");
            return Ok(());
        }
        let status = subprocess::Exec::cmd(&command_vec[0])
            .args(&command_vec[1..])
            .join()?;
        if !status.success() {
            Err(Error::LinkFailed)
        } else {
            Ok(())
        }
    }
}
