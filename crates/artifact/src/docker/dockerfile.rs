//! Execute Docker commands as spawned processes. This module provides an abstraction, [Dockerfile], for building a
//! Wasm benchmark in a single step.
//!
//! Use the `DOCKER` environment variable to change the binary to use for this; the default is
//! `"docker"`.
use crate::docker::interpret::contextualize;

use super::engine::*;
use super::interpret::parse;
use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::{env, fmt, fs, process};

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

    /// TODO
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
        fs::create_dir(&container_dir).context("failed to create container directory")?;

        // Parse the Dockerfile.
        let dockerfile_contents =
            fs::read_to_string(&self.0).context("failed to read Dockerfile")?;
        let instructions = parse(&dockerfile_contents)?;

        // Execute each recognized instruction in the container directory.
        let mut cwd = container_dir.clone();
        let dockerfile_dir = self
            .parent_dir()
            .canonicalize()
            .context("failed while canonicalizing Dockerfile directory")?;
        for i in instructions {
            cwd = i.execute(&dockerfile_dir, &container_dir, cwd)?;
        }

        // Extract the `source` file to `destination`.
        let source = contextualize(&container_dir, &cwd, source.as_ref());
        debug!(
            "Extracting: {} -> {}",
            source.display(),
            destination.as_ref().display()
        );
        fs::copy(source, destination).context("failed to extract file from temporary directory")?;

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
