use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The audited QEMU releases: each stable release tag and its peeled commit,
/// independent of the tags a local tree happens to have.
pub const PINNED_VERSIONS: &str = include_str!("../qemu-versions.lock");

#[derive(Debug, PartialEq, Eq)]
pub struct PinnedVersion {
    pub tag: String,
    pub commit: String,
}

/// A release lockfile that does not list unique, ordered stable releases.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PinError {
    /// A line that is not a `tag commit` pair.
    #[error("Invalid release pin on line {line}")]
    Malformed { line: usize },
    /// A tag that is not a stable `v` release tag.
    #[error("Invalid release tag on line {line}")]
    Tag { line: usize },
    /// A release listed twice or after a later one.
    #[error("Release pins must be unique and ordered: {tag} on line {line}")]
    Order { line: usize, tag: String },
    /// A commit that is not 40 hexadecimal digits.
    #[error("Invalid commit pin for {tag} on line {line}")]
    Commit { line: usize, tag: String },
    /// A lockfile without any pin.
    #[error("Release lockfile is empty")]
    Empty,
}

/// A failure to read releases or their schemas from a QEMU git tree.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TreeError {
    /// The QEMU tree's directory cannot be used.
    #[error("Cannot use {} as a QEMU tree", path.display())]
    Repository {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// git could not be started.
    #[error("Failed to run git {command}")]
    Spawn {
        command: &'static str,
        #[source]
        source: std::io::Error,
    },
    /// git exited with an error.
    #[error("git {command} failed in {}: {stderr}", repo.display())]
    Failed {
        command: &'static str,
        repo: PathBuf,
        stderr: String,
    },
    /// git printed output that is not UTF-8.
    #[error("Invalid UTF-8 in git {command} output")]
    Utf8 {
        command: &'static str,
        #[source]
        source: std::string::FromUtf8Error,
    },
    /// A pinned release tag that the tree does not have.
    #[error("Missing QEMU release {tag}; fetch the release tags into the QEMU tree first")]
    MissingRelease { tag: String },
    /// A pinned release tag that points at another commit.
    #[error("QEMU release {tag} moved: expected {expected}, found {found}")]
    MovedRelease {
        tag: String,
        expected: String,
        found: String,
    },
    /// The directory for extracted schemas could not be created.
    #[error("Failed to create temp directory")]
    TempDir(#[source] std::io::Error),
    /// The schema archive of a release could not be read.
    #[error("Failed to read the schema archive of {tag}")]
    Archive {
        tag: String,
        #[source]
        source: std::io::Error,
    },
    /// An extracted schema file could not be written.
    #[error("Failed to unpack {}", path.display())]
    Unpack {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// The schema files of one release, extracted into a temporary directory that
/// is removed when this value is dropped.
#[derive(Debug)]
pub struct ExtractedSchemas(tempfile::TempDir);

impl ExtractedSchemas {
    /// The directory the release's `qapi`, `qga` and top-level schema files are in.
    pub fn path(&self) -> &Path {
        self.0.path()
    }
}

/// Read a release lockfile containing `tag peeled-commit` pairs.
pub fn parse_pinned_versions(input: &str) -> Result<Vec<PinnedVersion>, PinError> {
    let mut versions = Vec::new();
    let mut previous = None;
    for (index, value) in input.lines().enumerate() {
        let line = index + 1;
        let value = value.split('#').next().unwrap().trim();
        if value.is_empty() {
            continue;
        }
        let fields: Vec<_> = value.split_ascii_whitespace().collect();
        let [tag, commit] = fields[..] else {
            return Err(PinError::Malformed { line });
        };
        let version = parse_version_tag_full(tag)
            .filter(|_| tag.starts_with('v'))
            .ok_or(PinError::Tag { line })?;
        if previous.is_some_and(|previous| version <= previous) {
            return Err(PinError::Order {
                line,
                tag: tag.into(),
            });
        }
        if commit.len() != 40 || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(PinError::Commit {
                line,
                tag: tag.into(),
            });
        }
        previous = Some(version);
        versions.push(PinnedVersion {
            tag: tag.into(),
            commit: commit.to_ascii_lowercase(),
        });
    }
    if versions.is_empty() {
        return Err(PinError::Empty);
    }
    Ok(versions)
}

/// The variables that point git at another repository than the one it runs
/// in (`git rev-parse --local-env-vars`); a git hook, for one, exports them.
const REPOSITORY_ENV: &[&str] = &[
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

/// Run git in the QEMU tree `repo` and nowhere else: neither the environment
/// nor an enclosing repository, such as the superproject of an empty
/// submodule directory, can redirect it.
fn git(repo: &Path, command: &'static str, args: &[&str]) -> Result<Output, TreeError> {
    let repo_error = |source| TreeError::Repository {
        path: repo.into(),
        source,
    };
    let root = std::fs::canonicalize(repo).map_err(repo_error)?;
    let mut git = Command::new("git");
    for name in REPOSITORY_ENV {
        git.env_remove(name);
    }
    if let Some(parent) = root.parent() {
        git.env("GIT_CEILING_DIRECTORIES", parent);
    }
    git.arg(command)
        .args(args)
        .current_dir(&root)
        .output()
        .map_err(|source| TreeError::Spawn { command, source })
}

fn failed(repo: &Path, command: &'static str, output: &Output) -> TreeError {
    TreeError::Failed {
        command,
        repo: repo.into(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().into(),
    }
}

fn git_stdout(repo: &Path, command: &'static str, args: &[&str]) -> Result<Vec<u8>, TreeError> {
    let output = git(repo, command, args)?;
    if !output.status.success() {
        return Err(failed(repo, command, &output));
    }
    Ok(output.stdout)
}

fn git_text(repo: &Path, command: &'static str, args: &[&str]) -> Result<String, TreeError> {
    String::from_utf8(git_stdout(repo, command, args)?)
        .map_err(|source| TreeError::Utf8 { command, source })
}

/// Resolve the release tag `tag` to the object `peel` names in it (`commit`
/// or `tree`). A tag the tree lacks is a [`TreeError::MissingRelease`]; any
/// other failure keeps git's message.
fn resolve_release(repo: &Path, tag: &str, peel: &str) -> Result<String, TreeError> {
    let name = format!("refs/tags/{tag}^{{{peel}}}");
    let output = git(
        repo,
        "rev-parse",
        &["-q", "--verify", "--end-of-options", &name],
    )?;
    if output.status.code() == Some(1) && output.stderr.is_empty() {
        return Err(TreeError::MissingRelease { tag: tag.into() });
    }
    if !output.status.success() {
        return Err(failed(repo, "rev-parse", &output));
    }
    let id = String::from_utf8(output.stdout).map_err(|source| TreeError::Utf8 {
        command: "rev-parse",
        source,
    })?;
    Ok(id.trim().into())
}

/// Check that the tree has every pinned release tag, each at its pinned
/// commit, so that a missing or moved tag cannot change which schemas are
/// read.
pub fn verify_pinned_versions(repo: &Path, versions: &[PinnedVersion]) -> Result<(), TreeError> {
    for pin in versions {
        let found = resolve_release(repo, &pin.tag, "commit")?;
        if found != pin.commit {
            return Err(TreeError::MovedRelease {
                tag: pin.tag.clone(),
                expected: pin.commit.clone(),
                found,
            });
        }
    }
    Ok(())
}

/// Given a QEMU source root, or the schemas extracted from a release, find
/// the QMP schema entry point.
pub fn find_schema_entry(qemu_root: &Path) -> Option<PathBuf> {
    let qapi_dir = qemu_root.join("qapi").join("qapi-schema.json");
    if qapi_dir.exists() {
        return Some(qapi_dir);
    }
    let root_schema = qemu_root.join("qapi-schema.json");
    if root_schema.exists() {
        return Some(root_schema);
    }
    None
}

/// Parse a stable release tag, retaining its historical fourth component for
/// ordering. Components are decimal numbers without leading zeros.
pub fn parse_version_tag_full(tag: &str) -> Option<(u32, u32, u32, u32)> {
    let tag = tag.strip_prefix('v').unwrap_or(tag);
    let parts: Vec<_> = tag.split('.').collect();
    if !(2..=4).contains(&parts.len()) {
        return None;
    }
    let mut values = [0; 4];
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty()
            || !part.bytes().all(|byte| byte.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0'))
        {
            return None;
        }
        values[index] = part.parse().ok()?;
    }
    Some((values[0], values[1], values[2], values[3]))
}

/// Parse a release tag's three-part version, the form QMP greetings report.
pub fn parse_version_tag(tag: &str) -> Option<(u32, u32, u32)> {
    let (major, minor, patch, _) = parse_version_tag_full(tag)?;
    Some((major, minor, patch))
}

/// Extract the schema files of the release tag `tag` into a temporary
/// directory, without touching the tree's working copy. The tag is resolved
/// once, and the schema roots it has are archived in one operation. A release
/// without schema files gives an empty directory.
pub fn extract_schema_from_tag(qemu_repo: &Path, tag: &str) -> Result<ExtractedSchemas, TreeError> {
    let tree = resolve_release(qemu_repo, tag, "tree")?;
    let tmpdir = tempfile::tempdir().map_err(TreeError::TempDir)?;
    let roots = git_text(
        qemu_repo,
        "ls-tree",
        &[
            "--name-only",
            "-z",
            &tree,
            "--",
            "qapi-schema.json",
            "qapi-schema-guest.json",
            "qapi",
            "qga",
        ],
    )?;
    let paths: Vec<_> = roots.split('\0').filter(|path| !path.is_empty()).collect();
    if paths.is_empty() {
        return Ok(ExtractedSchemas(tmpdir));
    }
    let tar_data = git_stdout(
        qemu_repo,
        "archive",
        &[&[tree.as_str(), "--"], &paths[..]].concat(),
    )?;
    let archive_error = |source| TreeError::Archive {
        tag: tag.into(),
        source,
    };

    // The schema roots hold sources and build files too; keep the schemas.
    let mut archive = tar::Archive::new(&tar_data[..]);
    for entry in archive.entries().map_err(archive_error)? {
        let mut entry = entry.map_err(archive_error)?;
        let path = entry.path().map_err(archive_error)?.into_owned();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }

        let dest = tmpdir.path().join(&path);
        let unpack_error = |source| TreeError::Unpack {
            path: path.clone(),
            source,
        };
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(unpack_error)?;
        }
        entry.unpack(&dest).map_err(unpack_error)?;
    }

    Ok(ExtractedSchemas(tmpdir))
}

/// Given a QEMU source root, or the schemas extracted from a release, find
/// the guest agent's schema entry point.
pub fn find_qga_schema_entry(qemu_root: &Path) -> Option<PathBuf> {
    let qga = qemu_root.join("qga").join("qapi-schema.json");
    if qga.exists() {
        return Some(qga);
    }
    let legacy = qemu_root.join("qapi-schema-guest.json");
    legacy.exists().then_some(legacy)
}

/// Discover release tags from the QMP schema era (1.0 onward).
pub fn discover_version_tags(qemu_repo: &Path) -> Result<Vec<String>, TreeError> {
    discover_schema_era_tags(qemu_repo, (1, 0, 0))
}

/// Discover release tags from the guest-agent schema era (0.15 onward).
pub fn discover_qga_version_tags(qemu_repo: &Path) -> Result<Vec<String>, TreeError> {
    discover_schema_era_tags(qemu_repo, (0, 15, 0))
}

fn discover_schema_era_tags(
    qemu_repo: &Path,
    earliest: (u32, u32, u32),
) -> Result<Vec<String>, TreeError> {
    // Plumbing, so that no tag or column configuration changes the format.
    let refs = git_text(
        qemu_repo,
        "for-each-ref",
        &["--format=%(refname:strip=2)", "refs/tags/v*"],
    )?;
    let mut tags: Vec<String> = refs
        .lines()
        .filter(|tag| parse_version_tag(tag).is_some_and(|version| version >= earliest))
        .map(str::to_string)
        .collect();

    tags.sort_by_key(|tag| parse_version_tag_full(tag).unwrap());
    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_release_components_are_strict_and_preserve_subpatch_order() {
        assert_eq!(parse_version_tag("v2.8.1.1"), Some((2, 8, 1)));
        assert_eq!(parse_version_tag_full("v2.8.1.1"), Some((2, 8, 1, 1)));
        for invalid in [
            "v+2.8.1",
            "v2..1",
            "v2.8.1.x",
            "v2.8.1.1.1",
            "v2.8.1.4294967296",
            "v2.8.1-rc0",
            "v01.0",
            "v2.08.1",
        ] {
            assert!(parse_version_tag(invalid).is_none(), "{invalid}");
        }
        let commit = "0".repeat(40);
        let pins = format!("v2.8.1 {commit}\nv2.8.1.1 {commit}\nv2.8.2 {commit}\n");
        assert_eq!(parse_pinned_versions(&pins).unwrap().len(), 3);
        assert!(parse_pinned_versions(&format!("v2.8.1.1 {commit}\nv2.8.1 {commit}\n")).is_err());
        assert!(parse_pinned_versions(&format!("v2.8.1 {commit}\nv2.8.1.0 {commit}\n")).is_err());
    }

    #[test]
    fn pinned_releases_are_ordered_and_end_at_current_baseline() {
        let pins = parse_pinned_versions(PINNED_VERSIONS).unwrap();
        assert_eq!(pins.first().unwrap().tag, "v0.15.0");
        assert_eq!(pins.last().unwrap().tag, "v11.1.3");
    }

    #[test]
    fn invalid_release_pins_are_rejected() {
        let tag = |tag: &str| tag.to_string();
        for (input, error) in [
            ("", PinError::Empty),
            ("# only a comment\n", PinError::Empty),
            ("v11.1.3", PinError::Malformed { line: 1 }),
            ("\nv11.1.3 a b", PinError::Malformed { line: 2 }),
            ("v11.2.0-rc0 deadbeef", PinError::Tag { line: 1 }),
            ("11.1.3 deadbeef", PinError::Tag { line: 1 }),
            (
                "v11.1.3\u{a0}005c61c0ef6f6ae615283440b4bb7be51b2c6ddd",
                PinError::Malformed { line: 1 },
            ),
            (
                "# pins\nv11.1.3 deadbeef",
                PinError::Commit {
                    line: 2,
                    tag: tag("v11.1.3"),
                },
            ),
        ] {
            assert_eq!(parse_pinned_versions(input), Err(error), "{input:?}");
        }
        let pin = "v11.1.3 005c61c0ef6f6ae615283440b4bb7be51b2c6ddd\n";
        assert_eq!(
            parse_pinned_versions(&pin.repeat(2)),
            Err(PinError::Order {
                line: 2,
                tag: tag("v11.1.3")
            })
        );
    }
}
