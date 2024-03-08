use argh::FromArgs;
use cretaceous::{error::Error as CrError, project::Project, UnusedKeys};
use std::path::PathBuf;

#[derive(argh::FromArgs)]
#[argh(description = "build tool xd")]
struct Args {
    #[argh(option, description = "project file")]
    project: Option<PathBuf>,

    #[argh(switch, description = "build with debug symbols")]
    debug: bool,

    #[argh(switch, description = "use verbose output")]
    verbose: bool,
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
        Some(project_file) => project_file
            .canonicalize()
            .map_err(|io| CrError::file_io(io, project_file.as_path()))?,
        None => cretaceous::find_project_file_from_current_dir()?,
    };
    let project_dir = project_file.parent().ok_or_else(|| CrError::NoProjectDir)?;

    tracing::info!("Building project from {}", project_file.display());

    let file = std::fs::read_to_string(project_file.as_path())?;
    let project: Project = toml::from_str(&file).map_err(|toml| CrError::ReadProject {
        toml,
        path: project_file.display().to_string(),
    })?;
    tracing::debug!("Project: {:#?}", project);

    let unused = project.unused_keys();
    if !unused.is_empty() {
        tracing::warn!("Unused keys: {:?}", unused);
    }

    let compiler = cretaceous::default_compiler()?;
    tracing::debug!("Compiler: {:#?}", compiler);

    for source_group in project.sources.iter() {
        for source in source_group.files.iter() {
            compiler.compile(
                project_dir,
                source.as_path(),
                &source_group.headers,
                args.debug,
                args.verbose,
            )?;
        }
    }

    Ok(())
}
