//! `ion-fmt` command line interface.
//!
//! Commands:
//! - `format`: format files in place, or stdin to stdout when no paths are passed
//! - `check`: report files/stdin that need formatting via process exit code
//! - `stdout`: print formatted file content without rewriting files
//! - no command: defaults to `stdout`; requires file paths or piped stdin
//!
//! `stdin` needs special handling:
//! - interactive terminal stdin should not be read implicitly, otherwise the CLI appears to hang
//! - piped or redirected stdin should be consumed as formatter input
//! - `IsTerminal` distinguishes those cases, but it does not tell us whether any bytes are ready

use clap::{Parser, Subcommand};
use ion_fmt::{
    DictionaryOptions, FieldStyle, FormatOptions, format_file_with_options,
    format_str_with_options, write_formatted_file_with_options,
};
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(feature = "dictionary-indexmap")]
const CLI_ABOUT: &str = "Formats Ion files.";
#[cfg(feature = "dictionary-indexmap")]
const CLI_LONG_ABOUT: &str = "Formats Ion files.\n\nBuild mode: dictionary-indexmap (section names and dictionary keys preserve insertion order).";

#[cfg(not(feature = "dictionary-indexmap"))]
const CLI_ABOUT: &str = "Formats Ion files.";
#[cfg(not(feature = "dictionary-indexmap"))]
const CLI_LONG_ABOUT: &str = "Formats Ion files.\n\nBuild mode: default dictionary backend (BTreeMap, section names and dictionary keys are sorted).";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    FormatInPlace,
    Check,
    Stdout,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputKind {
    Stdin,
    Paths,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CliExitCode {
    Success = 0,
    Failure = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StyleOption {
    DictionaryField(FieldStyle),
}

impl FromStr for StyleOption {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, value) = s
            .split_once('=')
            .ok_or_else(|| format!("Invalid `--style` value `{s}`. Expected `key=value`."))?;

        match key {
            "dictionary-field" => value.parse().map(Self::DictionaryField),
            _ => Err(format!(
                "Unsupported `--style` key `{key}`. Supported keys: `dictionary-field`."
            )),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "ion-fmt", version, about = CLI_ABOUT, long_about = CLI_LONG_ABOUT)]
struct Cli {
    /// Style options in `key=value` form (repeatable).
    ///
    /// Supported:
    /// - `dictionary-field=multiline` (default)
    /// - `dictionary-field=singleline`
    #[arg(long = "style", value_name = "KEY=VALUE", global = true)]
    styles: Vec<StyleOption>,

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

fn main() {
    let parsed = Cli::try_parse().unwrap_or_else(|error| error.exit());
    let stdin_is_terminal = stdin_is_terminal();

    match run_with_cli(&parsed, stdin_is_terminal) {
        Ok(code) => std::process::exit(code.value()),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

/// Returns whether stdin is connected to an interactive terminal.
///
/// `true` means reading stdin implicitly would block waiting for the user to
/// finish terminal input. `false` covers piped and redirected stdin, which is
/// safe to consume as formatter input.
#[must_use]
fn stdin_is_terminal() -> bool {
    io::stdin().is_terminal()
}

/// Dispatches execution by command and processes stdin or paths as needed.
///
/// A bare `ion-fmt` invocation maps to the `stdout` mode. Whether it then reads
/// stdin or returns an error depends on the terminal check in `resolve_input_kind`.
fn run_with_cli(parsed: &Cli, stdin_is_terminal: bool) -> Result<CliExitCode, String> {
    let options = parsed.format_options();

    match parsed.command.as_ref() {
        Some(CliCommand::Format { paths }) => {
            run_mode(paths, Mode::FormatInPlace, stdin_is_terminal, options)
        }
        Some(CliCommand::Check { paths }) => {
            run_mode(paths, Mode::Check, stdin_is_terminal, options)
        }
        Some(CliCommand::Stdout { paths }) => {
            run_mode(paths, Mode::Stdout, stdin_is_terminal, options)
        }
        None => run_mode(&[], Mode::Stdout, stdin_is_terminal, options),
    }
}

fn run_mode(
    paths: &[PathBuf],
    mode: Mode,
    stdin_is_terminal: bool,
    options: FormatOptions,
) -> Result<CliExitCode, String> {
    match resolve_input_kind(paths, stdin_is_terminal) {
        Some(InputKind::Stdin) => run_with_stdin(mode, options),
        Some(InputKind::Paths) => Ok(run_with_paths(paths, mode, options)),
        None => Err(missing_input_error(mode)),
    }
}

/// Resolves whether the command should read stdin or operate on explicit paths.
///
/// `IsTerminal` is used here only to distinguish interactive terminal stdin from
/// piped or redirected stdin. It does not mean "stdin currently has data".
///
/// This keeps `ion-fmt` from blocking on `read_to_string()` when the user runs
/// it directly in a terminal without any paths, while still allowing
/// `cat file.ion | ion-fmt` and `ion-fmt < file.ion`.
#[must_use]
fn resolve_input_kind(paths: &[PathBuf], stdin_is_terminal: bool) -> Option<InputKind> {
    if paths.is_empty() {
        if stdin_is_terminal {
            None
        } else {
            Some(InputKind::Stdin)
        }
    } else {
        Some(InputKind::Paths)
    }
}

/// Runs formatter against stdin for a selected mode.
fn run_with_stdin(mode: Mode, options: FormatOptions) -> Result<CliExitCode, String> {
    let mut raw = String::new();
    io::stdin()
        .read_to_string(&mut raw)
        .map_err(|error| format!("Failed to read stdin: {error}"))?;

    let formatted = format_str_with_options(&raw, options).map_err(|error| format!("{error}"))?;
    let needs_formatting = formatted != raw;

    if mode == Mode::Check {
        if needs_formatting {
            eprintln!("stdin needs formatting");
            Ok(CliExitCode::Failure)
        } else {
            Ok(CliExitCode::Success)
        }
    } else {
        print!("{formatted}");
        Ok(CliExitCode::Success)
    }
}

/// Runs formatter over file paths for a selected mode.
fn run_with_paths(paths: &[PathBuf], mode: Mode, options: FormatOptions) -> CliExitCode {
    let mut has_errors = false;
    let mut needs_formatting = false;

    for path in paths {
        match mode {
            Mode::Stdout => match format_file_with_options(path, options) {
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
            Mode::Check => match format_file_with_options(path, options) {
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
                if let Err(error) = write_formatted_file_with_options(path, options) {
                    has_errors = true;
                    eprintln!("{}: {error}", path.display());
                }
            }
        }
    }

    if has_errors || (mode == Mode::Check && needs_formatting) {
        CliExitCode::Failure
    } else {
        CliExitCode::Success
    }
}

#[must_use]
fn missing_input_error(mode: Mode) -> String {
    format!(
        "No input provided for `{}`. Pass one or more file paths or pipe Ion through stdin.",
        mode.command_name()
    )
}

impl Mode {
    #[must_use]
    fn command_name(self) -> &'static str {
        match self {
            Self::FormatInPlace => "format",
            Self::Check => "check",
            Self::Stdout => "stdout",
        }
    }
}

impl CliExitCode {
    #[must_use]
    fn value(self) -> i32 {
        self as i32
    }
}

impl Cli {
    #[must_use]
    fn format_options(&self) -> FormatOptions {
        let mut field = FieldStyle::Multiline;

        for option in &self.styles {
            match option {
                StyleOption::DictionaryField(next_style) => field = *next_style,
            }
        }

        FormatOptions {
            dictionary: DictionaryOptions { field },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Cli, CliCommand, CliExitCode, FieldStyle, InputKind, Mode, missing_input_error,
        resolve_input_kind, run_with_cli,
    };
    use clap::Parser;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;
    use std::sync::LazyLock;
    use test_case::test_case;

    #[derive(Debug)]
    struct ResolveInputKindTestCase {
        paths: Vec<PathBuf>,
        stdin_is_terminal: bool,
        expected: Option<InputKind>,
    }

    static PIPED_STDIN_CASE: LazyLock<ResolveInputKindTestCase> =
        LazyLock::new(|| ResolveInputKindTestCase {
            paths: vec![],
            stdin_is_terminal: false,
            expected: Some(InputKind::Stdin),
        });
    static INTERACTIVE_WITH_PATH_CASE: LazyLock<ResolveInputKindTestCase> =
        LazyLock::new(|| ResolveInputKindTestCase {
            paths: vec![PathBuf::from("sample.ion")],
            stdin_is_terminal: true,
            expected: Some(InputKind::Paths),
        });
    static INTERACTIVE_NO_INPUT_CASE: LazyLock<ResolveInputKindTestCase> =
        LazyLock::new(|| ResolveInputKindTestCase {
            paths: vec![],
            stdin_is_terminal: true,
            expected: None,
        });

    #[test_case(&*PIPED_STDIN_CASE; "uses stdin when data is piped")]
    #[test_case(&*INTERACTIVE_WITH_PATH_CASE; "uses paths when files are provided")]
    #[test_case(&*INTERACTIVE_NO_INPUT_CASE; "rejects interactive stdin without files")]
    fn resolve_input_kind_cases(case: &ResolveInputKindTestCase) {
        assert_eq!(
            case.expected,
            resolve_input_kind(&case.paths, case.stdin_is_terminal)
        );
    }

    #[derive(Debug, Eq, PartialEq)]
    enum RunWithCliExpectation {
        ExitCode(CliExitCode),
        Error(String),
    }

    #[derive(Debug)]
    struct RunWithCliTestCase {
        cli: Cli,
        stdin_is_terminal: bool,
        expected: RunWithCliExpectation,
    }

    static DEFAULT_COMMAND_MISSING_INPUT_CASE: LazyLock<RunWithCliTestCase> =
        LazyLock::new(|| RunWithCliTestCase {
            cli: Cli {
                styles: vec![],
                command: None,
            },
            stdin_is_terminal: true,
            expected: RunWithCliExpectation::Error(missing_input_error(Mode::Stdout)),
        });
    static FORMAT_COMMAND_MISSING_INPUT_CASE: LazyLock<RunWithCliTestCase> =
        LazyLock::new(|| RunWithCliTestCase {
            cli: Cli {
                styles: vec![],
                command: Some(CliCommand::Format { paths: vec![] }),
            },
            stdin_is_terminal: true,
            expected: RunWithCliExpectation::Error(missing_input_error(Mode::FormatInPlace)),
        });
    static CHECK_COMMAND_MISSING_INPUT_CASE: LazyLock<RunWithCliTestCase> =
        LazyLock::new(|| RunWithCliTestCase {
            cli: Cli {
                styles: vec![],
                command: Some(CliCommand::Check { paths: vec![] }),
            },
            stdin_is_terminal: true,
            expected: RunWithCliExpectation::Error(missing_input_error(Mode::Check)),
        });
    static STDOUT_COMMAND_MISSING_INPUT_CASE: LazyLock<RunWithCliTestCase> =
        LazyLock::new(|| RunWithCliTestCase {
            cli: Cli {
                styles: vec![],
                command: Some(CliCommand::Stdout { paths: vec![] }),
            },
            stdin_is_terminal: true,
            expected: RunWithCliExpectation::Error(missing_input_error(Mode::Stdout)),
        });
    static CHECK_FORMATTED_FILE_CASE: LazyLock<RunWithCliTestCase> =
        LazyLock::new(|| RunWithCliTestCase {
            cli: Cli {
                styles: vec![],
                command: Some(CliCommand::Check {
                    paths: vec![PathBuf::from("tests/readme/formatted.ion")],
                }),
            },
            stdin_is_terminal: true,
            expected: RunWithCliExpectation::ExitCode(CliExitCode::Success),
        });
    static CHECK_UNFORMATTED_FILE_CASE: LazyLock<RunWithCliTestCase> =
        LazyLock::new(|| RunWithCliTestCase {
            cli: Cli {
                styles: vec![],
                command: Some(CliCommand::Check {
                    paths: vec![PathBuf::from("tests/readme/unformatted.ion")],
                }),
            },
            stdin_is_terminal: true,
            expected: RunWithCliExpectation::ExitCode(CliExitCode::Failure),
        });

    #[test_case(&*DEFAULT_COMMAND_MISSING_INPUT_CASE; "default command missing input")]
    #[test_case(&*FORMAT_COMMAND_MISSING_INPUT_CASE; "format command missing input")]
    #[test_case(&*CHECK_COMMAND_MISSING_INPUT_CASE; "check command missing input")]
    #[test_case(&*STDOUT_COMMAND_MISSING_INPUT_CASE; "stdout command missing input")]
    #[test_case(&*CHECK_FORMATTED_FILE_CASE; "check command with formatted file")]
    #[test_case(&*CHECK_UNFORMATTED_FILE_CASE; "check command with unformatted file")]
    fn run_with_cli_cases(case: &RunWithCliTestCase) {
        let actual = match run_with_cli(&case.cli, case.stdin_is_terminal) {
            Ok(code) => RunWithCliExpectation::ExitCode(code),
            Err(error) => RunWithCliExpectation::Error(error),
        };

        assert_eq!(case.expected, actual);
    }

    #[derive(Debug)]
    struct ParseStyleCliTestCase {
        args: Vec<&'static str>,
        expected_style: Option<FieldStyle>,
        expected_error_fragment: Option<&'static str>,
    }

    static STYLE_DEFAULT_CASE: LazyLock<ParseStyleCliTestCase> =
        LazyLock::new(|| ParseStyleCliTestCase {
            args: vec!["ion-fmt"],
            expected_style: Some(FieldStyle::Multiline),
            expected_error_fragment: None,
        });
    static STYLE_SINGLELINE_CASE: LazyLock<ParseStyleCliTestCase> =
        LazyLock::new(|| ParseStyleCliTestCase {
            args: vec!["ion-fmt", "--style", "dictionary-field=singleline"],
            expected_style: Some(FieldStyle::Singleline),
            expected_error_fragment: None,
        });
    static STYLE_MULTILINE_CASE: LazyLock<ParseStyleCliTestCase> =
        LazyLock::new(|| ParseStyleCliTestCase {
            args: vec!["ion-fmt", "--style", "dictionary-field=multiline"],
            expected_style: Some(FieldStyle::Multiline),
            expected_error_fragment: None,
        });
    static STYLE_LAST_WINS_CASE: LazyLock<ParseStyleCliTestCase> =
        LazyLock::new(|| ParseStyleCliTestCase {
            args: vec![
                "ion-fmt",
                "--style",
                "dictionary-field=singleline",
                "--style",
                "dictionary-field=multiline",
            ],
            expected_style: Some(FieldStyle::Multiline),
            expected_error_fragment: None,
        });
    static STYLE_MISSING_VALUE_CASE: LazyLock<ParseStyleCliTestCase> =
        LazyLock::new(|| ParseStyleCliTestCase {
            args: vec!["ion-fmt", "--style", "dictionary-field"],
            expected_style: None,
            expected_error_fragment: Some("Expected `key=value`"),
        });
    static STYLE_UNKNOWN_KEY_CASE: LazyLock<ParseStyleCliTestCase> =
        LazyLock::new(|| ParseStyleCliTestCase {
            args: vec!["ion-fmt", "--style", "table-column=singleline"],
            expected_style: None,
            expected_error_fragment: Some("Supported keys: `dictionary-field`"),
        });
    static STYLE_UNKNOWN_VALUE_CASE: LazyLock<ParseStyleCliTestCase> =
        LazyLock::new(|| ParseStyleCliTestCase {
            args: vec!["ion-fmt", "--style", "dictionary-field=folded"],
            expected_style: None,
            expected_error_fragment: Some("Expected `singleline` or `multiline`"),
        });

    #[test_case(&*STYLE_DEFAULT_CASE; "default style is multiline")]
    #[test_case(&*STYLE_SINGLELINE_CASE; "parses explicit singleline")]
    #[test_case(&*STYLE_MULTILINE_CASE; "parses multiline")]
    #[test_case(&*STYLE_LAST_WINS_CASE; "last style wins when repeated")]
    #[test_case(&*STYLE_MISSING_VALUE_CASE; "rejects style missing value")]
    #[test_case(&*STYLE_UNKNOWN_KEY_CASE; "rejects unknown style key")]
    #[test_case(&*STYLE_UNKNOWN_VALUE_CASE; "rejects unknown dictionary-field style value")]
    fn parse_style_cli_cases(case: &ParseStyleCliTestCase) {
        match Cli::try_parse_from(case.args.clone()) {
            Ok(cli) => {
                assert_eq!(case.expected_error_fragment, None);
                assert_eq!(
                    case.expected_style,
                    Some(cli.format_options().dictionary.field)
                );
            }
            Err(error) => {
                let rendered = error.to_string();
                assert_eq!(case.expected_style, None);
                assert!(
                    case.expected_error_fragment
                        .is_some_and(|fragment| rendered.contains(fragment)),
                    "actual CLI parse error: {rendered}"
                );
            }
        }
    }
}
