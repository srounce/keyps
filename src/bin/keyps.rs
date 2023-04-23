use clap::{ArgAction, Parser};
use itertools::Itertools;
use keyps::{
    keyper::{Keyper, KeyperConfig},
    source::SourceIdentifier,
};
use log::{debug, error, info};
use std::{env, fs, path::PathBuf, process};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Verbosity level (eg. -vvv)
    #[arg(short, action = ArgAction::Count)]
    verbosity: u8,

    /// Path to authorized_keys file (eg. ./authorized_keys). This file must exist and be writable. 
    ///
    /// If not specified, an upward search for the closest available `.ssh/authorized_keys` file
    /// will be performed from the current working directory.
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// One or more sources with one of the following formats:
    ///
    /// * `github:<username>`
    ///
    /// * `gitlab:<username>`
    ///
    /// * `<https url>`
    #[arg(short, long = "source")]
    sources: Vec<SourceIdentifier>,

    /// Time in seconds to wait between polling sources
    #[arg(short, long, default_value = "10")]
    interval: u64,
}

#[derive(Debug, thiserror::Error)]
enum ApplicationError {
    #[error("Application encountered an unknown error: {0:#?}")]
    Unknown(Box<dyn std::error::Error>),

    #[error(r#"Unable to locate authorized_keys file at: "{path}""#)]
    AuthorizedKeysNotFound { path: PathBuf },

    #[error(r#"Unable to locate authorized_keys file at the following paths: {search_paths:#?}"#)]
    DefaultAuthorizedKeysNotFound { search_paths: Vec<PathBuf> },

    #[error(r#"Unable to read authorized_keys file at: {path}"#)]
    ReadAuthorizedKeysFailed { path: PathBuf },

    #[error(r#"No sources specified"#)]
    NoSources,

    // TODO: Improve the output of this error case
    #[error(r#"Invalid sources specified: {sources:#?}"#)]
    InvalidSources { sources: Vec<SourceIdentifier> },
}

impl ApplicationError {
    pub fn unknown<T>(original_error: T) -> ApplicationError
    where
        T: std::error::Error + 'static,
    {
        ApplicationError::Unknown(Box::new(original_error))
    }
}

fn main() {
    let args = Args::parse();

    match run(args) {
        // TODO: Better error message output/formatting
        Err(e) => {
            error!("{}", e.to_string());
            process::exit(1)
        }
        _ => {}
    };
}

fn run(args: Args) -> Result<(), ApplicationError> {
    env_logger::Builder::new()
        .filter_level(match args.verbosity {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Error,
            2 => log::LevelFilter::Warn,
            3 => log::LevelFilter::Info,
            4 => log::LevelFilter::Debug,
            5 | _ => log::LevelFilter::Trace,
        })
        .init();

    if args.sources.len() == 0 {
        return Err(ApplicationError::NoSources);
    }

    let source_list = args.sources.iter().map(|s| s.to_string()).join("\n");
    info!("Using sources:\n{}", source_list);

    let invalid_sources = args
        .sources
        .clone()
        .into_iter()
        .filter(|s| match s {
            SourceIdentifier::Invalid(_) => true,
            _ => false,
        })
        .collect::<Vec<_>>();
    if invalid_sources.len() > 0 {
        return Err(ApplicationError::InvalidSources {
            sources: invalid_sources,
        });
    }

    let authorized_keys_path = match args.file {
        Some(path) => path
            .canonicalize()
            .map_err(|_| ApplicationError::AuthorizedKeysNotFound { path }),
        None => env::current_dir()
            .map_err(|err| ApplicationError::unknown(err))
            .and_then(|path| find_up(path, ".ssh/authorized_keys")),
    }?;

    info!(
        r#"Using authorized_keys file "{}""#,
        authorized_keys_path.to_str().unwrap_or_default()
    );

    //let authorized_keys = fs::read_to_string(&authorized_keys_path).map_err(|_| {
    //ApplicationError::ReadAuthorizedKeysFailed {
    //path: authorized_keys_path.clone(),
    //}
    //})?;

    //debug!("Loaded keys:\n{authorized_keys}");

    let mut signals = {
        use signal_hook::consts::signal::*;
        use signal_hook::iterator::Signals;

        Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT]).map_err(ApplicationError::unknown)
    }?;

    let service = Keyper::start(KeyperConfig {
        sources: args.sources,
        file_path: authorized_keys_path,
        interval: args.interval,
    });

    'outer: loop {
        // TODO:
        //
        // loop
        // - for: each source
        // - - update source hashmap entry
        // - if: check for managed key entries block
        // - - true: overwrite block
        // - - false: append to file
        // - update authorized_keys file
        // - sleep X seconds
        //
        for signal in signals.pending() {
            use signal_hook::consts::signal::*;

            match signal {
                SIGHUP => {
                    // TODO: allow specifying a file based config and reloading
                    debug!("TODO: reload config");
                    // Reload configuration
                    // Reopen the log file
                }
                SIGTERM | SIGINT | SIGQUIT => {
                    service.stop().join();
                    break 'outer;
                }
                _ => unreachable!(),
            }
        }
    }

    // TODO:
    // cleanup
    debug!("Exiting");

    Ok(())
}

fn find_up<F>(starting_path: PathBuf, filename: F) -> Result<PathBuf, ApplicationError>
where
    F: Into<String> + Copy,
{
    let search_paths = starting_path
        .ancestors()
        .map(|p| p.join(filename.into()))
        .collect::<Vec<_>>();

    search_paths
        .iter()
        .find(|p| p.exists())
        .map(|p| p.clone())
        .ok_or_else(|| ApplicationError::DefaultAuthorizedKeysNotFound { search_paths })
}

//fn parse_authorized_keys() ->
