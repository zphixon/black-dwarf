use argh::FromArgs;
use cretaceous::{error::Error as CrError, project::Project, UnusedKeys};
use std::path::PathBuf;

#[derive(argh::FromArgs)]
#[argh(description = "build tool xd")]
struct Args {
    #[argh(option, description = "project file")]
    project: Option<PathBuf>,
}

fn main() {
    tracing_subscriber::fmt::init();
    match run() {
        Ok(()) => {}
        Err(err) => {
            use std::error::Error;

            tracing::error!("{}", err);

            let mut source = err.source();
            while let Some(the_source) = source {
                tracing::error!("{}", the_source);
                source = the_source.source();
            }

            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), CrError> {
    let arg_strings = std::env::args().collect::<Vec<_>>();
    let arg_strs = arg_strings.iter().map(String::as_str).collect::<Vec<_>>();
    let args = match Args::from_args(&arg_strs[0..1], &arg_strs[1..]) {
        Ok(args) => args,
        Err(exit) => {
            if exit.status.is_err() {
                return Err(CrError::Cli(exit.output.trim().into()));
            } else {
                tracing::info!(
                    "\n{}\n{}",
                    exit.output,
                    if let Ok(project_file) = cretaceous::find_project_file_from_current_dir() {
                        format!("There is a project at {}", project_file.display())
                    } else {
                        "No project in current directory (or any parent directory)".into()
                    },
                );
                return Ok(());
            }
        }
    };

    let project_file = match args.project {
        Some(project_file) => project_file,
        None => cretaceous::find_project_file_from_current_dir()?,
    };

    tracing::info!("Building project from {}", project_file.display());

    let file = std::fs::read_to_string(project_file.as_path())?;
    let bd: Project = toml::from_str(&file).map_err(|toml| CrError::ReadProject {
        toml,
        path: format!("{}", project_file.display()),
    })?;
    tracing::debug!("Project: {:#?}", bd);

    let unused = bd.unused_keys();
    if !unused.is_empty() {
        tracing::warn!("Unused keys: {:?}", unused);
    }

    let compiler = cretaceous::default_compiler()?;
    tracing::debug!("Compiler: {:#?}", compiler);

    let name = &compiler.name;
    macros::env_var!("COMPILer", "PATH"; "COMPILER", name, "PATH");

    Ok(())
}
