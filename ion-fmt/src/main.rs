//! `ion-fmt` command line interface.
//!
//! Commands:
//! - `format`: format files in place, or stdin to stdout when no paths are passed
//! - `check`: report files/stdin that need formatting via process exit code
//! - `stdout`: print formatted file content without rewriting files
//! - no command: defaults to `stdout` with stdin -> stdout

use clap::{Parser, Subcommand};
use ion_fmt::{format_file, format_str, write_formatted_file};
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    FormatInPlace,
    Check,
    Stdout,
}

#[derive(Debug, Parser)]
#[command(name = "ion-fmt", version, about = "Formats Ion files.")]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    /// Format files in place or stdin.
    Format {
        /// Ion file paths. Reads stdin when omitted.
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
    /// Check formatting without writing changes.
    Check {
        /// Ion file paths. Reads stdin when omitted.
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
    /// Print formatted output to stdout.
    Stdout {
        /// Ion file paths. Reads stdin when omitted.
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
}

/// Runs formatter against stdin for a selected mode.
fn run_with_stdin(mode: Mode) -> Result<i32, String> {
    let mut raw = String::new();
    io::stdin()
        .read_to_string(&mut raw)
        .map_err(|error| format!("Failed to read stdin: {error}"))?;

    let formatted = format_str(&raw).map_err(|error| format!("{error}"))?;
    let needs_formatting = formatted != raw;

    if mode == Mode::Check {
        if needs_formatting {
            eprintln!("stdin needs formatting");
            Ok(1)
        } else {
            Ok(0)
        }
    } else {
        print!("{formatted}");
        Ok(0)
    }
}

/// Runs formatter over file paths for a selected mode.
fn run_with_paths(paths: &[PathBuf], mode: Mode) -> i32 {
    let mut has_errors = false;
    let mut needs_formatting = false;

    for path in paths {
        match mode {
            Mode::Stdout => match format_file(path) {
                Ok(result) => {
                    if paths.len() > 1 {
                        println!("==> {} <==", path.display());
                    }
                    print!("{}", result.formatted);
                }
                Err(error) => {
                    has_errors = true;
                    eprintln!("{}: {error}", path.display());
                }
            },
            Mode::Check => match format_file(path) {
                Ok(result) => {
                    if result.changed {
                        needs_formatting = true;
                        eprintln!("needs formatting: {}", path.display());
                    }
                }
                Err(error) => {
                    has_errors = true;
                    eprintln!("{}: {error}", path.display());
                }
            },
            Mode::FormatInPlace => {
                if let Err(error) = write_formatted_file(path) {
                    has_errors = true;
                    eprintln!("{}: {error}", path.display());
                }
            }
        }
    }

    i32::from(has_errors || (mode == Mode::Check && needs_formatting))
}

/// Dispatches execution by command and processes stdin or paths as needed.
fn run_with_cli(parsed: &Cli) -> Result<i32, String> {
    match parsed.command.as_ref() {
        Some(CliCommand::Format { paths }) => {
            if paths.is_empty() {
                run_with_stdin(Mode::FormatInPlace)
            } else {
                Ok(run_with_paths(paths, Mode::FormatInPlace))
            }
        }
        Some(CliCommand::Check { paths }) => {
            if paths.is_empty() {
                run_with_stdin(Mode::Check)
            } else {
                Ok(run_with_paths(paths, Mode::Check))
            }
        }
        Some(CliCommand::Stdout { paths }) => {
            if paths.is_empty() {
                run_with_stdin(Mode::Stdout)
            } else {
                Ok(run_with_paths(paths, Mode::Stdout))
            }
        }
        None => run_with_stdin(Mode::Stdout),
    }
}

fn main() {
    let parsed = Cli::try_parse().unwrap_or_else(|error| error.exit());

    match run_with_cli(&parsed) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
