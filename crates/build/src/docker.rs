//! Execute Docker commands as spawned processes. This module provides an abstraction, [Dockerfile], for building a
//! Wasm benchmark in a single step.
//!
//! Use the `DOCKER` environment variable to change the binary to use for this; the default is
//! `"docker"`.
use log::{debug, error, info, warn};
use regex::Regex;
use std::path::{self, Path};
use std::process::{self, Command, Stdio};
use std::string::FromUtf8Error;
use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};
use std::{collections::HashMap, ffi::OsStr};
use std::{convert::TryFrom, path::PathBuf};
use std::{env, fmt, fs, io};
use std::{
    io::{BufRead, BufReader, Write},
    thread,
};
use thiserror::Error;

/// Represents a Dockerfile that can build a Wasm benchmark.
#[derive(Clone, Debug)]
pub struct Dockerfile(PathBuf);

impl Dockerfile {
    /// Construct a new Dockerfile from its path.
    pub fn from(p: PathBuf) -> Self {
        Self(p)
    }

    /// Find the parent directory of the Dockerfile; this is useful for determining the default
    /// context directory.
    pub fn parent_dir(&self) -> PathBuf {
        self.0
            .parent()
            .expect("to exist within a parent directory")
            .to_path_buf()
    }

    /// Build the Dockerfile and extract the file placed at `source` inside the container to
    /// `destination` in the host. Optionally pass arguments to the build process (equivalent to
    /// `docker --arg ...`).
    pub fn extract<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        source: P1,
        destination: P2,
        args: Option<DockerBuildArgs>,
    ) -> Result<()> {
        info!("Building Dockerfile: {}", self.0.display());
        let image_id = build_image(&self.0, args)?;
        let container_id = create_container(&image_id)?;
        copy_file_from_container(&container_id, source.as_ref(), destination.as_ref())?;
        remove_container(&container_id)?;
        remove_image(&image_id)?;
        assert!(destination.as_ref().exists());
        Ok(())
    }

    ///
    pub fn interpret<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        source: P1,
        destination: P2,
        args: Option<DockerBuildArgs>,
    ) -> Result<()> {
        warn!(
            "Building Dockerfile outside of docker: {}",
            self.0.display()
        );

        // Create the directory within which to run the Dockerfile. Note that there is no real
        // isolation or protection of any kind: we simply run the Dockerfile as a script in this
        // directory.
        let container_dir = env::temp_dir().join(format!("sightglass-no-docker-{}", process::id()));
        fs::create_dir(&container_dir).unwrap();

        // Parse the Dockerfile.
        let dockerfile_contents = fs::read_to_string(&self.0)?;
        let instructions = parse(&dockerfile_contents)?;

        // Execute each recognized instruction in the container directory.
        let mut cwd = container_dir.clone();
        let dockerfile_dir = self.parent_dir().canonicalize().unwrap();
        for i in instructions {
            cwd = i.execute(&dockerfile_dir, &container_dir, cwd)?;
        }

        Ok(())
    }
}
impl Into<PathBuf> for Dockerfile {
    fn into(self) -> PathBuf {
        self.0
    }
}
/// Create a single-string identifier for the Dockerfile: `dockerfile@[hash of file bytes]`.
impl fmt::Display for Dockerfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bytes = fs::read(&self.0).expect("a readable file");
        let hash = blake3::hash(&bytes);
        write!(f, "dockerfile@{:?}", hash)
    }
}

pub struct DockerBuildArgs<'a>(HashMap<Cow<'a, str>, Cow<'a, str>>);
impl<'a> DockerBuildArgs<'a> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn set<S>(&mut self, key: S, value: S)
    where
        S: Into<Cow<'a, str>>,
    {
        self.0.insert(key.into(), value.into());
    }
}

pub type Result<T> = std::result::Result<T, DockerError>;

/// Build an image from a Dockerfile with the Dockerfile's parent directory as context.
pub fn build_image<P: AsRef<Path>>(
    dockerfile: P,
    args: Option<DockerBuildArgs<'_>>,
) -> Result<ImageId> {
    let context_dir = dockerfile
        .as_ref()
        .parent()
        .expect("a Dockerfile to exist within a parent directory");
    let tar_context_dir = tar_dir(context_dir)?;

    let mut command = Command::new(docker_binary());
    // Read the context from a tar directory,
    // https://docs.docker.com/engine/reference/commandline/build/#tarball-contexts
    command.arg("build").arg("-");

    if let Some(args) = args {
        for (k, v) in args.0 {
            command.arg("--build-arg").arg(format!("{}={}", k, v));
        }
    }

    execute_and_capture_last_line(command, Some(tar_context_dir))
}

/// Create a container from a Docker image ID.
pub fn create_container(image: &ImageId) -> Result<ContainerId> {
    let mut command = Command::new(docker_binary());
    command.arg("create").arg(&image.0);
    execute_and_capture_last_line(command, None)
}

/// Copy a file from a Docker container.
pub fn copy_file_from_container<P: AsRef<OsStr>>(
    container: &ContainerId,
    from: P,
    to: P,
) -> Result<()> {
    let mut command = Command::new(docker_binary());
    let container_location = format!("{}:{}", &container.0, from.as_ref().to_string_lossy());
    command.arg("cp").arg(container_location).arg(to);
    execute_command(command)
}

/// Remove a Docker container.
pub fn remove_container(container: &ContainerId) -> Result<()> {
    let mut command = Command::new(docker_binary());
    command.arg("rm").arg(container);
    execute_command(command)
}

/// Remove a Docker image.
pub fn remove_image(image: &ImageId) -> Result<()> {
    let mut command = Command::new(docker_binary());
    command.arg("rmi").arg(image);
    execute_command(command)
}

// Retrieve the Docker binary to use; set the `DOCKER` environment variable to change the default
// `docker`.
fn docker_binary() -> String {
    env::var("DOCKER").unwrap_or("docker".to_string())
}

// Execute a Docker command and capture the last line as a Docker ID.
fn execute_and_capture_last_line(mut command: Command, input: Option<Vec<u8>>) -> Result<DockerId> {
    info!("Executing: {:?}", &command);
    command.stdout(Stdio::piped());
    if input.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command.spawn()?;
    // TODO pipe stderr to the same place somehow.

    // If we need to pipe input to stdin, do so in a separate thread to avoid deadlocking.
    if let Some(bytes) = input {
        let mut child_stdin = child.stdin.take().unwrap();
        thread::spawn(move || {
            child_stdin.write_all(&bytes).unwrap();
        });
    }

    // Capture all printed lines to the logger and the last one as the ID.
    let reader = BufReader::new(child.stdout.take().unwrap());
    let mut lines = Vec::new();
    reader.lines().filter_map(|l| l.ok()).for_each(|l| {
        debug!("{}", &l);
        lines.push(l);
    });
    let last_line = lines.last().cloned();

    // Check that the process executed successfuly.
    let status = child.wait()?;
    if status.success() && last_line.is_some() {
        let id = DockerId::from(last_line.unwrap());
        info!("Succeeded: {}", id);
        Ok(id)
    } else {
        error!("Failed: {:?}", child);
        Err(DockerError::FailedExecution(lines.join("\n")))
    }
}

// Execute a docker command, expecting a 0 exit code.
pub fn execute_command(mut command: Command) -> Result<()> {
    info!("Executing: {:?}", &command);
    let output = command.output()?;
    if output.status.success() {
        info!("Succeeded");
        Ok(())
    } else {
        error!("Failed: {:?}", output);
        Err(DockerError::FailedExecution(
            String::from_utf8(output.stdout).unwrap(),
        ))
    }
}

/// Describe the ways this module can fail.
#[derive(Debug, Error)]
pub enum DockerError {
    #[error("failed with IO error")]
    IoError(#[from] io::Error),
    #[error("failed to execute docker command: {0}")]
    FailedExecution(String),
    #[error("failed to parse an ID")]
    FailedParsingId(#[from] FromUtf8Error),
    #[error("failed to parse Dockerfile: {0}")]
    FailedParsingFile(String),
}

pub type ImageId = DockerId;
pub type ContainerId = DockerId;

/// Container for the SHA256 digest that Docker uses for identifying objects.
#[derive(Debug)]
pub struct DockerId(String);

impl From<String> for DockerId {
    fn from(s: String) -> Self {
        DockerId(s.trim().split_whitespace().last().unwrap().to_string())
    }
}

impl TryFrom<Vec<u8>> for DockerId {
    type Error = DockerError;
    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Ok(Self::from(String::from_utf8(value)?))
    }
}

impl Display for DockerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl AsRef<OsStr> for DockerId {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

/// Construct a TAR archive from a given path, returning the uncompressed bytes. This allows us to
/// use symlinks in Docker contexts, which otherwise are forbidden for security reasons; in our
/// case, the advantage of using symlinks--drastically less duplication of benchmark code--outweighs
/// any security impact.
///
/// Warning: this will store the entire Docker context in memory!
fn tar_dir<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    use tar::Builder;
    let mut bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut bytes);
        builder.follow_symlinks(true);
        builder.append_dir_all(".", path)?;
        builder.finish()?;
    }
    Ok(bytes)
}

fn parse(contents: &str) -> Result<Vec<Instruction>> {
    use Instruction::*;
    let mut instructions = Vec::new();
    // let multiline_accumulator = None;
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with("#") {
            // Skip comments.
        } else if let Some(("WORKDIR", dir)) = line.split_once(" ") {
            instructions.push(Workdir(dir.trim()))
        } else if let Some(("COPY", from_to)) = line.split_once(" ") {
            if let Some((from, to)) = from_to.trim().split_once(" ") {
                instructions.push(Copy(from.trim(), to.trim()))
            } else {
                let message = format!("COPY is limited to two parts, found: {}", from_to);
                return Err(DockerError::FailedParsingFile(message));
            }
        } else if let Some(("RUN", command)) = line.split_once(" ") {
            let pattern = Regex::new(r#"[\\""].+?[\\""]|[^ ]+"#).unwrap();
            let matches: Vec<&str> = pattern.find_iter(command).map(|m| m.as_str()).collect();
            instructions.push(Run(matches))
        } else if let Some(("ENV", key_var)) = line.split_once(" ") {
            if let Some((key, var)) = key_var.trim().split_once(" ") {
                instructions.push(Env(key.trim(), var.trim()))
            } else {
                let message = format!("ENV must have two parts, found: {}", key_var);
                return Err(DockerError::FailedParsingFile(message));
            }
        } else {
        }
    }
    Ok(instructions)
}

enum Instruction<'a> {
    Run(Vec<&'a str>),
    Workdir(&'a str),
    Copy(&'a str, &'a str),
    Env(&'a str, &'a str),
}
impl<'a> Instruction<'a> {
    fn execute(
        &self,
        dockerfile_dir: &Path,
        base_dir: &Path,
        mut current_dir: PathBuf,
    ) -> Result<PathBuf> {
        let restrict = |base: &Path, current: &Path, add: &Path| {
            if add.is_absolute() {
                // TODO handle Windows
                base.join(add.strip_prefix("/").unwrap())
            } else {
                current.join(add)
            }
        };
        let normalize = |path: &Path| -> PathBuf {
            let mut normalized = PathBuf::from("/");
            for part in path.iter() {
                match part.to_str().unwrap() {
                    "." | "" => { /* Skip. */ }
                    ".." => {
                        let _ = normalized.pop();
                    }
                    p => normalized.push(p),
                }
            }
            normalized
        };

        match self {
            Instruction::Run(run) => {
                let pretty = run.join(" ");
                debug!("RUN {:?}", pretty);
                let mut cmd = Command::new(&run[0]);
                cmd.args(&run[1..]).current_dir(&current_dir);
                let out = cmd.output().expect("failed to execute");
                debug!("> stdout: {:?}", std::str::from_utf8(&out.stdout));
                debug!("> stderr: {:?}", std::str::from_utf8(&out.stderr));
                if !out.status.success() {
                    return Err(DockerError::FailedExecution(format!(
                        "Failed to run command: {}",
                        pretty
                    )));
                }
            }
            Instruction::Workdir(path) => {
                current_dir = restrict(base_dir, &current_dir, &PathBuf::from(path));
                if !current_dir.is_dir() {
                    fs::create_dir_all(&current_dir).unwrap();
                }
                debug!("WORKDIR {}", current_dir.display());
            }
            Instruction::Copy(from, to) => {
                let from = dockerfile_dir.join(from);
                let is_directory =
                    |p: &str| p.trim().ends_with(path::MAIN_SEPARATOR) || p.trim() == ".";
                let mut to_ = normalize(&restrict(base_dir, &current_dir, &PathBuf::from(to)));
                let to = if is_directory(to) {
                    if !to_.is_dir() {
                        fs::create_dir_all(&to_).unwrap();
                    }
                    to_.push(from.file_name().unwrap());
                    to_
                } else {
                    let parent = to_.parent().unwrap();
                    if !parent.is_dir() {
                        fs::create_dir_all(&to_).unwrap();
                    }
                    to_
                };

                debug!("COPY {} {}", from.display(), to.display());
                fs::copy(from, to).unwrap();
            }
            Instruction::Env(..) => todo!(),
        }
        Ok(current_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tar_directory_will_materialize_symlinks() {
        use std::path::Path;
        use tar::Archive;
        let bytes = tar_dir("./tests").unwrap();
        let mut archive = Archive::new(&bytes[..]);
        let linked_file = archive
            .entries()
            .unwrap()
            .into_iter()
            .find(|e| e.as_ref().unwrap().path().unwrap() == Path::new("sightglass.h"))
            .unwrap()
            .unwrap();
        assert!(linked_file.size() > 100);
    }
}
