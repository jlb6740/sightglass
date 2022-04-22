//! Experimental module to interpret Dockerfile commands in a temporary directory. WARNING: this
//! module provides none of Docker's isolation and has limited functionality.
//!
//! This is an "escape hatch" to reuse the existing Docker infrastructure in Sightglass but allow
//! for building artifacts directly on a machine. To do so, a limited set of [`Instruction`]s are
//! parsed from `Dockerfile`s and executed in a temporary directory on the system. Some effort has
//! been made to match the semantics of the [Dockerfile reference], but complex `Dockerfile`s may
//! run into issues like:
//!  - Only the `RUN`, `WORKDIR`, `COPY`/`ADD`, and `ARG`/`ENV` operations are supported; any other
//!    `Dockerfile` operations are ignored.
//!  - The execution of these operations is not isolated/containerized! The commands will affect the
//!    machine directly (some effort has been taken to constrain paths within the execution
//!    directory but this should not be relied upon).
//!  - Certain operations may not exactly match what Docker does: e.g., `RUN mv /some/absolute/path
//!    ...` will reference the true absolute path, not a path within the temp directory.
//!
//! [Dockerfile reference]: https://docs.docker.com/engine/reference/builder
use anyhow::bail;
use log::{debug, error, trace};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, path};
use thiserror::Error;

use crate::DockerBuildArgs;

/// List the instructions that this module can actually interpret.
pub enum Instruction<'a> {
    Run(Vec<&'a str>),
    Workdir(&'a str),
    Copy(&'a str, &'a str),
    Arg(&'a str, &'a str),
}

impl<'a> Instruction<'a> {
    /// Execute a single `Dockerfile` instruction with the given context.
    pub fn execute(&self, context: &mut ExecContext) -> anyhow::Result<()> {
        match self {
            Instruction::Run(run) => {
                // Replace any absolute paths.
                let run = make_contextual_shell_command(run, context.base_dir);
                //let pretty = run.join(" ");
                debug!("RUN {:?}", run);

                let mut cmd = Command::new(&run[0]);
                cmd.args(&run[1..])
                    .current_dir(&context.current_dir)
                    .envs(context.env.0.iter().map(|(a, b)| (a.as_ref(), b.as_ref())));
                let out = cmd.output().expect("failed to execute");
                trace!("  stdout: {:?}", std::str::from_utf8(&out.stdout));
                trace!("  stderr: {:?}", std::str::from_utf8(&out.stderr));

                if !out.status.success() {
                    bail!("Failed to RUN: {:?}", run);
                }
            }
            Instruction::Workdir(path) => {
                context.current_dir =
                    contextualize(context.base_dir, &context.current_dir, &PathBuf::from(path));
                if !context.current_dir.is_dir() {
                    fs::create_dir_all(&context.current_dir)?;
                }
                debug!("WORKDIR {}", context.current_dir.display());
            }
            Instruction::Copy(from, to) => {
                let is_directory =
                    |p: &str| p.trim().ends_with(path::MAIN_SEPARATOR) || p.trim() == ".";

                let from = {
                    let path = normalize(&context.dockerfile_dir.join(from));
                    if is_directory(from) || path.is_dir() {
                        CopyLocation::Dir(path)
                    } else {
                        CopyLocation::File(path)
                    }
                };

                let to = {
                    let path = normalize(&contextualize(
                        context.base_dir,
                        &context.current_dir,
                        &PathBuf::from(to),
                    ));
                    // Note that we can't copy from a directory to a file, so if `from` is a
                    // directory, so should `to`.
                    if is_directory(to) || from.is_dir() {
                        CopyLocation::Dir(path)
                    } else {
                        CopyLocation::File(path)
                    }
                };

                debug!("COPY {:?} {:?}", from, to);
                copy(from, to)?;
            }
            Instruction::Arg(key, val) => {
                if !context.env.0.contains_key(*key) {
                    let unquoted_val = enquote::unquote(val).unwrap_or_else(|_| val.to_string());
                    context.env.set(key.to_string(), unquoted_val);
                    debug!("ARG {}={}", key, val);
                } else {
                    debug!("ARG skipped, key {} exists", key);
                }
            }
        }
        Ok(())
    }
}

pub struct ExecContext<'a> {
    pub(crate) dockerfile_dir: &'a Path,
    pub(crate) base_dir: &'a Path,
    pub(crate) current_dir: PathBuf,
    pub(crate) env: DockerBuildArgs<'a>,
}

/// Contextualize `path`:
/// - if `path` is absolute, append it to `container_dir`
/// - if `path` is relative, append it to the `current_dir`.
pub fn contextualize(container_dir: &Path, current_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        // TODO handle Windows
        container_dir.join(
            path.strip_prefix("/")
                .expect("an absolute path should have a / prefix"),
        )
    } else {
        current_dir.join(path)
    }
}

/// Transform the command by:
/// - wrapping it in `sh -c ...`
/// - replacing any obvious absolute paths with their container-dir-relativized version (see
///   `contextualize`).
pub fn make_contextual_shell_command<'a>(
    original_command: &Vec<&'a str>,
    base_dir: &Path,
) -> Vec<String> {
    let mut contextualized_parts = Vec::new();
    for part in original_command {
        let part = if part.starts_with("/") {
            contextualize(base_dir, &PathBuf::new(), &PathBuf::from(part))
                .to_str()
                .expect("the command to be convertible to a string")
                .to_string()
        } else {
            part.to_string()
        };
        contextualized_parts.push(part)
    }
    vec!["sh".into(), "-c".into(), contextualized_parts.join(" ")]
}

/// Remove all relative parts (i.e., `.`, `..`) from a path. We cannot use `canonicalize` here
/// because that checks if files actually exist and we must be able to normalize "to be created"
/// files.
fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::from("/");
    for part in path.iter() {
        match part.to_str().expect("the path should be a valid string") {
            "." | "" => { /* Skip. */ }
            ".." => {
                let _ = normalized.pop();
            }
            p => normalized.push(p),
        }
    }
    normalized
}

/// Handle the various cases for copying directories and files to and from each other.
fn copy(from: CopyLocation, to: CopyLocation) -> anyhow::Result<()> {
    if let Dir(to) = &to {
        fs::create_dir_all(to)?;
    }

    use CopyLocation::*;
    let _ = match (from, to) {
        (File(from), File(to)) => fs::copy(from, to)?,
        (File(from), Dir(to)) => {
            let file_name = from.file_name().expect("the path should have a file name");
            let to = to.join(file_name);
            fs::copy(from, to)?
        }
        (Dir(from), Dir(to)) => copy_directory_recursively(from, to)?,
        (Dir(_), File(_)) => bail!("cannot copy a directory to a file location",),
    };
    Ok(())
}

fn copy_directory_recursively(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<u64> {
    fs::create_dir_all(&dst)?;
    let mut files_copied = 0;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            files_copied +=
                copy_directory_recursively(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            files_copied += 1;
        }
    }
    Ok(files_copied)
}

#[derive(Debug)]
enum CopyLocation {
    Dir(PathBuf),
    File(PathBuf),
}
impl CopyLocation {
    fn is_dir(&self) -> bool {
        match self {
            CopyLocation::Dir(..) => true,
            _ => false,
        }
    }
}

/// Parse a `Dockerfile` into a sequence of [`Instruction`]s. Note that unknown operators will be
/// silently ignored.
pub fn parse(contents: &str) -> Result<Vec<Instruction>, DockerInterpretError> {
    use DockerInterpretError::*;
    use Instruction::*;

    let mut instructions = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with("#") {
            // Skip comments.
            continue;
        }

        if let Some((command, rest)) = line.split_once(" ") {
            let rest = rest.trim();
            match command {
                "WORKDIR" => instructions.push(Workdir(rest)),
                "COPY" | "ADD" => {
                    if let Some((from, to)) = rest.split_once(" ") {
                        instructions.push(Copy(from.trim(), to.trim()))
                    } else {
                        return Err(BadParseCopy(rest.to_string()));
                    }
                }
                "RUN" => {
                    if rest.ends_with("\\") {
                        return Err(BadParseMultiline);
                    }
                    let pattern = Regex::new(r#"[\\""].+?[\\""]|[^ ]+"#)
                        .expect("this regex should not fail to compile");
                    let matches: Vec<&str> = pattern.find_iter(rest).map(|m| m.as_str()).collect();
                    instructions.push(Run(matches))
                }
                "ARG" | "ENV" => {
                    if let Some((key, val)) = rest.split_once("=") {
                        instructions.push(Arg(key.trim(), val.trim()))
                    } else {
                        return Err(BadParseArg(rest.to_string()));
                    }
                }
                _ => { /* Skip unknown command. */ }
            }
        } else {
            // Unknown line--no command.
        }
    }

    Ok(instructions)
}

/// Describe the ways this module can fail.
#[derive(Debug, Error)]
pub enum DockerInterpretError {
    #[error("COPY is limited to two parts, found: {0}")]
    BadParseCopy(String),
    #[error("RUN cannot execute multi-line commands")]
    BadParseMultiline,
    #[error("ARG must have two parts, found: {0}")]
    BadParseArg(String),
}
