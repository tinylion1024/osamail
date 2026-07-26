use std::{ffi::OsStr, process::ExitCode};

use clap::{Parser, error::ErrorKind};
use osamail::{automation::OsascriptRunner, cli::Cli, commands, error::OsaMailError, output};

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().collect();
    let json = json_requested(&arguments);
    let cli = match Cli::try_parse_from(arguments) {
        Ok(cli) => cli,
        Err(error) => {
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                return if error.print().is_ok() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                };
            }
            if json {
                let error = OsaMailError::InvalidArguments(error.to_string().trim().to_owned());
                if output::write_error(&mut std::io::stderr().lock(), true, &error).is_err() {
                    return ExitCode::FAILURE;
                }
            } else if error.print().is_err() {
                return ExitCode::FAILURE;
            }
            return ExitCode::from(2);
        }
    };
    let runner = OsascriptRunner::new();
    let result = commands::execute(
        &cli,
        &runner,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
    );

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if let Err(write_error) =
                output::write_error(&mut std::io::stderr().lock(), cli.json, &error)
            {
                eprintln!("error: failed to report OsaMail error: {write_error}");
            }
            ExitCode::from(error.exit_code())
        }
    }
}

fn json_requested(arguments: &[std::ffi::OsString]) -> bool {
    arguments
        .iter()
        .skip(1)
        .take_while(|argument| argument.as_os_str() != OsStr::new("--"))
        .any(|argument| argument.as_os_str() == OsStr::new("--json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_detection_stops_at_argument_separator() {
        let arguments = ["osamail", "search", "--", "--json"]
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        assert!(!json_requested(&arguments));
    }
}
