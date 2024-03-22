use argh::FromArgs;
use cretaceous::{
    error::Error as CrError,
    project::{Project, TargetType, UnresolvedProject},
    UnusedKeys,
};
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

    #[argh(positional, description = "build targets")]
    targets: Vec<String>,
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
                tracing::error!("Because of: {}", the_source);
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
    let parsed_project: UnresolvedProject =
        toml::from_str(&file).map_err(|toml| CrError::ReadProject {
            toml,
            path: project_file.display().to_string(),
        })?;
    tracing::debug!("Project meta: {:#?}", parsed_project.project);

    let unused = parsed_project.unused_keys();
    if !unused.is_empty() {
        tracing::warn!("Unused keys: {:?}", unused);
    }

    let project = parsed_project.resolve(project_dir)?;
    let compiler = cretaceous::default_compiler()?;
    tracing::debug!("Compiler: {:#?}", compiler);

    let targets = if args.targets.is_empty() {
        project.targets_in_order()?
    } else {
        project.targets_in_order_from(args.targets.iter().map(|name| name.as_str()))?
    };
    tracing::debug!("Targets: {:#?}", targets);

    for (_, target) in targets {
        tracing::info!("Compiling target {}", target.name);

        for source in target.sources.iter() {
            let mut include_paths = vec![target.path.as_path()];
            for need in target.needs.iter() {
                include_paths.push(
                    project
                        .target
                        .get(need.as_str())
                        .ok_or_else(|| {
                            CrError::Bug(format!("Resolved project had unknown target"))
                        })?
                        .path
                        .as_path(),
                );
            }

            compiler.compile(source, &include_paths, args.debug, args.verbose)?;
        }

        for target_type in target.type_.iter() {
            match target_type {
                TargetType::Static => {
                    // heehoo
                }

                TargetType::Dynamic => {
                    // haha
                }

                TargetType::Binary => {
                    // hue
                }
            }
        }
    }

    Ok(())
}
