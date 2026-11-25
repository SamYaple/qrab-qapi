use qapi_codegen::unified::{Docs, GenerateError, Protocol, Releases, generate};
use qapi_codegen::versions::{
    PINNED_VERSIONS, PinError, TreeError, parse_pinned_versions, parse_version_tag,
    verify_pinned_versions,
};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
Usage: qapi-codegen [OPTIONS] --output-dir <DIR> [--] [QEMU_TREE]

Generate mod.rs, commands.rs, events.rs and metadata.rs of the QMP API,
or with --qga of the QEMU Guest Agent API, from every pinned QEMU
release. Nothing is written unless every release generates valid Rust.

QEMU_TREE is a QEMU git tree with each pinned release tag at its pinned
commit, vendor/qemu by default. Relative paths are relative to the
working directory.

Options:
  --output-dir <DIR>      Write the modules to DIR, creating it if needed
  --qga                   Generate the QEMU Guest Agent modules
  --versions-file <FILE>  Read the release pins from FILE instead of the
                          embedded qemu-versions.lock
  --schema-docs           Include QEMU's GPL-2.0-or-later schema
                          documentation, for local reading only
  -h, --help              Print this help

An option's value is the next argument or follows '=', as in
--output-dir=DIR. '--' ends the options.
";

/// Why the command line failed. Its source chain says why in detail.
#[derive(Debug, thiserror::Error)]
enum CliError {
    /// Arguments that are not a valid command line: exit status 2.
    #[error("{0}")]
    Usage(String),
    #[error("Failed to read the release lockfile {}", path.display())]
    ReadLockfile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Invalid release lockfile {lockfile}")]
    Lockfile {
        lockfile: String,
        #[source]
        source: PinError,
    },
    /// Pins that select no release with the protocol's schema, which would
    /// give empty modules.
    #[error(
        "Refusing to write empty {protocol} modules: {lockfile} pins no release from {first}, the first with a {protocol} schema"
    )]
    NoSchema {
        lockfile: String,
        protocol: &'static str,
        first: String,
    },
    #[error("Failed to check {} against the release pins of {lockfile}", repo.display())]
    Tree {
        repo: PathBuf,
        lockfile: String,
        #[source]
        source: TreeError,
    },
    #[error("Failed to generate the {protocol} modules from {}", repo.display())]
    Generate {
        protocol: &'static str,
        repo: PathBuf,
        #[source]
        source: Box<GenerateError>,
    },
    #[error(transparent)]
    Write(Box<GenerateError>),
}

#[derive(Debug, PartialEq, Eq)]
struct Options {
    repo: PathBuf,
    output_dir: PathBuf,
    versions_file: Option<PathBuf>,
    protocol: Protocol,
    docs: Docs,
}

/// Parse the arguments, or `None` for `--help`. Paths may be any OS string,
/// and options must be UTF-8.
fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Option<Options>, CliError> {
    let usage = |message: String| Err(CliError::Usage(message));
    let mut args = args.into_iter();
    let (mut repo, mut output_dir, mut versions_file) = (None, None, None);
    let (mut protocol, mut docs, mut options) = (Protocol::Qmp, Docs::Reference, true);
    while let Some(arg) = args.next() {
        let text = arg.to_string_lossy();
        if !options || !text.starts_with('-') {
            if repo.is_some() {
                return usage(format!("Unexpected argument {text}"));
            }
            repo = Some(PathBuf::from(arg));
            continue;
        }
        let (name, inline) = match text.split_once('=') {
            Some((name, value)) if name.starts_with("--") => (name, Some(value)),
            _ => (text.as_ref(), None),
        };
        match (name, inline) {
            ("--", None) => options = false,
            ("-h" | "--help", None) => return Ok(None),
            ("--qga", None) => protocol = Protocol::Qga,
            ("--schema-docs", None) => docs = Docs::Schema,
            ("--qga" | "--schema-docs" | "--help", Some(_)) => {
                return usage(format!("{name} takes no value"));
            }
            ("--output-dir" | "--versions-file", _) => {
                let value = match inline {
                    Some(_) if arg.to_str().is_none() => {
                        return usage(format!(
                            "The value of {name} is not UTF-8; give it as the next argument"
                        ));
                    }
                    Some(value) => OsString::from(value),
                    None => args.next().unwrap_or_default(),
                };
                if value.is_empty() {
                    return usage(format!("{name} requires a value"));
                }
                let slot = if name == "--output-dir" {
                    &mut output_dir
                } else {
                    &mut versions_file
                };
                if slot.replace(PathBuf::from(value)).is_some() {
                    return usage(format!("{name} is given twice"));
                }
            }
            _ => return usage(format!("Unknown option {text}")),
        }
    }
    let Some(output_dir) = output_dir else {
        return usage("--output-dir is required".into());
    };
    Ok(Some(Options {
        repo: repo.unwrap_or_else(|| "vendor/qemu".into()),
        output_dir,
        versions_file,
        protocol,
        docs,
    }))
}

fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), CliError> {
    let Some(options) = parse_args(args)? else {
        print!("{USAGE}");
        return Ok(());
    };
    let text;
    let (pins, lockfile) = match &options.versions_file {
        Some(path) => {
            text = std::fs::read_to_string(path).map_err(|source| CliError::ReadLockfile {
                path: path.clone(),
                source,
            })?;
            (text.as_str(), path.display().to_string())
        }
        None => (PINNED_VERSIONS, "the embedded qemu-versions.lock".into()),
    };
    let pins = parse_pinned_versions(pins).map_err(|source| CliError::Lockfile {
        lockfile: lockfile.clone(),
        source,
    })?;
    let protocol = options.protocol.name();
    let since = options.protocol.first_release();
    let first = match since {
        (major, minor, 0) => format!("{major}.{minor}"),
        (major, minor, micro) => format!("{major}.{minor}.{micro}"),
    };
    if !pins
        .iter()
        .any(|pin| parse_version_tag(&pin.tag).is_some_and(|version| version >= since))
    {
        return Err(CliError::NoSchema {
            lockfile,
            protocol,
            first,
        });
    }
    let start = std::time::Instant::now();
    verify_pinned_versions(&options.repo, &pins).map_err(|source| CliError::Tree {
        repo: options.repo.clone(),
        lockfile,
        source,
    })?;
    let tags: Vec<_> = pins.iter().map(|pin| pin.tag.as_str()).collect();
    let output = generate(
        &options.repo,
        Releases::Tags(&tags),
        options.protocol,
        options.docs,
    )
    .map_err(|source| CliError::Generate {
        protocol,
        repo: options.repo.clone(),
        source: Box::new(source),
    })?;
    for warning in &output.warnings {
        eprintln!("warning: {warning}");
    }
    output
        .write_to(&options.output_dir)
        .map_err(|error| CliError::Write(Box::new(error)))?;
    eprintln!(
        "Generated {} in {:.1}s",
        options.output_dir.display(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}

fn main() -> ExitCode {
    let Err(error) = run(std::env::args_os().skip(1)) else {
        return ExitCode::SUCCESS;
    };
    eprintln!("error: {error}");
    let mut source = std::error::Error::source(&error);
    while let Some(cause) = source {
        eprintln!("  caused by: {cause}");
        source = cause.source();
    }
    if let CliError::Usage(_) = error {
        eprint!("\n{USAGE}");
        ExitCode::from(2)
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(input: &[&str]) -> Result<Option<Options>, CliError> {
        parse_args(input.iter().map(OsString::from))
    }

    fn options(input: &[&str]) -> Options {
        args(input).unwrap().unwrap()
    }

    fn usage_error(input: &[&str]) -> String {
        match args(input) {
            Err(CliError::Usage(message)) => message,
            other => panic!("{input:?}: {other:?}"),
        }
    }

    #[test]
    fn rejects_incomplete_unknown_or_repeated_arguments() {
        for (input, message) in [
            (&[][..], "--output-dir is required"),
            (&["qemu", "--qga"], "--output-dir is required"),
            (&["--output-dir"], "--output-dir requires a value"),
            (&["--output-dir="], "--output-dir requires a value"),
            (&["--output-dir", ""], "--output-dir requires a value"),
            (
                &["--output-dir", "out", "--versions-file"],
                "--versions-file requires a value",
            ),
            (
                &["--output-dir", "out", "--unknown"],
                "Unknown option --unknown",
            ),
            (&["--output-dir", "out", "-q"], "Unknown option -q"),
            (&["--output-dir", "out", "-h=x"], "Unknown option -h=x"),
            (
                &["--output-dir", "out", "--qga=yes"],
                "--qga takes no value",
            ),
            (
                &["--output-dir", "out", "qemu", "other"],
                "Unexpected argument other",
            ),
            (
                &["--output-dir", "out", "--", "qemu", "other"],
                "Unexpected argument other",
            ),
            (
                &["--output-dir", "a", "--output-dir=b"],
                "--output-dir is given twice",
            ),
            (
                &[
                    "--output-dir",
                    "out",
                    "--versions-file",
                    "a",
                    "--versions-file",
                    "b",
                ],
                "--versions-file is given twice",
            ),
        ] {
            assert_eq!(usage_error(input), message, "{input:?}");
        }
    }

    #[test]
    fn values_are_the_next_argument_or_follow_an_equals_sign() {
        assert_eq!(
            options(&["--output-dir", "-out", "--versions-file", "--qga"]),
            Options {
                repo: "vendor/qemu".into(),
                output_dir: "-out".into(),
                versions_file: Some("--qga".into()),
                protocol: Protocol::Qmp,
                docs: Docs::Reference,
            }
        );
        assert_eq!(
            options(&["--output-dir=a=b", "--versions-file=-pins", "--", "--qga"]),
            Options {
                repo: "--qga".into(),
                output_dir: "a=b".into(),
                versions_file: Some("-pins".into()),
                protocol: Protocol::Qmp,
                docs: Docs::Reference,
            }
        );
        assert_eq!(
            options(&["--output-dir", "out", "--", "--"]).repo,
            PathBuf::from("--")
        );
    }

    #[cfg(unix)]
    #[test]
    fn paths_need_not_be_utf8() {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::OsStr::from_bytes(b"out\xff");
        let options = parse_args([OsString::from("--output-dir"), path.into(), path.into()])
            .unwrap()
            .unwrap();
        assert_eq!(options.output_dir, PathBuf::from(path));
        assert_eq!(options.repo, PathBuf::from(path));
        let inline = std::ffi::OsStr::from_bytes(b"--output-dir=out\xff");
        assert!(matches!(
            parse_args([inline.into()]),
            Err(CliError::Usage(message))
                if message == "The value of --output-dir is not UTF-8; give it as the next argument"
        ));
    }

    #[test]
    fn accepts_options_before_and_after_the_tree() {
        assert_eq!(
            options(&[
                "--qga",
                "--schema-docs",
                "--output-dir",
                "out",
                "qemu",
                "--versions-file",
                "pins",
            ]),
            Options {
                repo: "qemu".into(),
                output_dir: "out".into(),
                versions_file: Some("pins".into()),
                protocol: Protocol::Qga,
                docs: Docs::Schema,
            }
        );
    }

    #[test]
    fn help_wins_over_missing_arguments() {
        for input in [&["--help"][..], &["-h"], &["qemu", "--qga", "--help"]] {
            assert!(args(input).unwrap().is_none(), "{input:?}");
        }
        assert_eq!(usage_error(&["--", "--help"]), "--output-dir is required");
    }

    #[test]
    fn usage_fits_a_terminal_and_lists_every_option() {
        for line in USAGE.lines() {
            assert!(line.len() <= 76, "{line}");
        }
        for option in [
            "--output-dir <DIR>",
            "--qga",
            "--versions-file <FILE>",
            "--schema-docs",
            "-h, --help",
        ] {
            assert!(USAGE.contains(&format!("\n  {option} ")), "{option}");
        }
    }
}
