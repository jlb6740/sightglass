//! Execute Docker commands as spawned processes. Use the `DOCKER` environment variable to change
//! the binary to use for this; the default is `"docker"`. In some future version (TODO) this module
//! be replaced by a crate interacting with the Docker API directly (e.g., `bollard`).
use log::{debug, error, info};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;
use std::{collections::HashMap, ffi::OsStr};
use std::{env, fmt, io, thread};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DockerEngineError>;

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

/// Container for the build arguments passed to `build`.
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
        Err(DockerEngineError::FailedExecution(lines.join("\n")))
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
        Err(DockerEngineError::FailedExecution(
            String::from_utf8(output.stdout).unwrap(),
        ))
    }
}

/// Container for the SHA256 digest that Docker uses for identifying objects.
#[derive(Debug)]
pub struct DockerId(String);
pub type ImageId = DockerId;
pub type ContainerId = DockerId;

impl From<String> for DockerId {
    fn from(s: String) -> Self {
        DockerId(s.trim().split_whitespace().last().unwrap().to_string())
    }
}

impl TryFrom<Vec<u8>> for DockerId {
    type Error = DockerEngineError;
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

/// Describe the ways this module can fail.
#[derive(Debug, Error)]
pub enum DockerEngineError {
    #[error("failed with IO error")]
    IoError(#[from] io::Error),
    #[error("failed to execute docker command: {0}")]
    FailedExecution(String),
    #[error("failed to parse an ID")]
    FailedParsingId(#[from] FromUtf8Error),
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
