use std::{
    ffi::OsStr,
    io::IsTerminal,
    process::ExitCode,
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

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
    let progress = DelayedProgress::start(&cli);
    let result = commands::execute(
        &cli,
        &runner,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
    );
    progress.finish();

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

struct DelayedProgress {
    cancel: Option<mpsc::Sender<()>>,
    worker: Option<thread::JoinHandle<()>>,
}

impl DelayedProgress {
    fn start(cli: &Cli) -> Self {
        if cli.json || cli.quiet || !std::io::stderr().is_terminal() {
            return Self::disabled();
        }
        let Some(message) = cli.command.progress_message() else {
            return Self::disabled();
        };

        let (cancel, receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            if matches!(
                receiver.recv_timeout(Duration::from_secs(1)),
                Err(RecvTimeoutError::Timeout)
            ) {
                eprintln!("{message}");
            }
        });
        Self {
            cancel: Some(cancel),
            worker: Some(worker),
        }
    }

    const fn disabled() -> Self {
        Self {
            cancel: None,
            worker: None,
        }
    }

    fn finish(mut self) {
        if let Some(cancel) = self.cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
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

    #[test]
    fn disabled_progress_finishes_cleanly() {
        DelayedProgress::disabled().finish();
    }
}
