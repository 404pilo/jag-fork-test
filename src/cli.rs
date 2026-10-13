//! C-cli: option parsing, dispatch, and the exit code.

use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use crate::config::{self, ConfigError, Env, RealFs, TtyPrompter};
use crate::failure::Failure;

/// Grep with a model that ranks the matches by how well they answer your question.
#[derive(Parser, Debug)]
#[command(name = "jag", version, arg_required_else_help = true)]
struct Cli {
    /// Store your TypeSafe API key, with no echo
    #[arg(long, conflicts_with = "forget_key")]
    setup: bool,

    /// Remove the stored TypeSafe API key
    #[arg(long = "forget-key")]
    forget_key: bool,
}

/// Parse the command line, run the command, and give the exit code.
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let vars = |name: &str| std::env::var_os(name);
    let home = std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from);
    let env = Env { vars: &vars, home };
    let mut stdout = io::stdout();

    let result = if cli.setup {
        config::setup(
            &env,
            &RealFs,
            &mut TtyPrompter,
            &mut stdout,
            io::stdin().is_terminal(),
        )
    } else if cli.forget_key {
        config::forget_key(&env, &RealFs, &mut stdout)
    } else {
        Ok(())
    };
    finish(result)
}

/// Write a failure to the error stream and give its exit code (CV-1, CV-3).
fn finish(result: Result<(), ConfigError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let failure = Failure::from(error);
            eprint!("{}", failure.render());
            ExitCode::from(failure.exit_code)
        }
    }
}
