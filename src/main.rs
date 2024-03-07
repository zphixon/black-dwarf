use argh::FromArgs;
use cretaceous::{error::Error as CrError, UnusedKeys};
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
                return Err(CrError::CliError(exit.output.trim().into()));
            } else {
                tracing::info!(
                    "\n{}\n{}",
                    exit.output,
                    if let Some(project_file) =
                        cretaceous::util::find_project_file_from_current_dir()
                    {
                        format!("There is a project at {}", project_file.display())
                    } else {
                        "No project in current directory (or any parent directory)".into()
                    },
                );
                return Ok(());
            }
        }
    };

    let Some(project_file) = args
        .project
        .or_else(cretaceous::util::find_project_file_from_current_dir)
    else {
        return Err(CrError::NoProject);
    };
    tracing::info!("Building project from {}", project_file.display());

    let file = std::fs::read_to_string(project_file.as_path())?;
    let bd: cretaceous::Project = toml::from_str(&file).map_err(|err| {
        cretaceous::error::ReadProject::from(err).with_filename(project_file.as_path())
    })?;
    tracing::debug!("Project: {:?}", bd);

    let unused = bd.unused_keys();
    if !unused.is_empty() {
        tracing::warn!("Unused keys: {:?}", unused);
    }

    Ok(())
}
