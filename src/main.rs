use argh::FromArgs;
use black_dwarf::error::Error as BdError;
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

fn run() -> Result<(), BdError> {
    let arg_strings = std::env::args().collect::<Vec<_>>();
    let arg_strs = arg_strings.iter().map(String::as_str).collect::<Vec<_>>();
    let args = match Args::from_args(&arg_strs[0..1], &arg_strs[1..]) {
        Ok(args) => args,
        Err(exit) => {
            for line in exit.output.lines() {
                tracing::info!("{}", line);
            }

            if let Some(project_file) = black_dwarf::util::find_project_file_from_current_dir() {
                tracing::info!("The current project is {}", project_file.display());
            } else {
                tracing::info!("No project in current directory (or any parent directory)");
            }

            if exit.status.is_err() {
                return Err(BdError::CliError);
            } else {
                return Ok(());
            }
        }
    };

    let Some(project_file) = args
        .project
        .or_else(black_dwarf::util::find_project_file_from_current_dir)
    else {
        return Err(BdError::NoProject);
    };
    tracing::info!("Building project from {}", project_file.display());

    let file = std::fs::read_to_string(project_file.as_path())?;
    let bd: black_dwarf::Project = toml::from_str(&file).map_err(|err| {
        black_dwarf::error::ReadProject::from(err).with_filename(project_file.as_path())
    })?;
    tracing::debug!("Project: {:?}", bd);

    let unused = bd.unused_keys();
    if !unused.is_empty() {
        tracing::warn!("Unused keys: {:?}", unused);
    }

    Ok(())
}
