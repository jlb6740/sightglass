//! Provides helper functions for building Sightglass artifacts using Dockerfiles.
//!
//! The key use for Docker's use in Sightglass is to make the building of engines and benchmarks
//! reproducible. Though not perfect, Docker images are a conventional way of fixing environmental
//! factors that cause differences in build output. Sightglass expects built Docker images to place
//! their build output in specific locations; [Dockerfile::extract()] provides a single function for
//! building the image, starting it as a container, and extracting the files from those locations.
//!
//! In certain cases, Docker may not fit, e.g., when building Sightglass inside Docker itself.
//! [Dockerfile::interpret()] provides an experimental "escape hatch" to interpret a Dockerfile
//! inside a temporary directory. See its documentation for the limitations to this approach.
mod dockerfile;
mod engine;
mod interpret;

pub use dockerfile::Dockerfile;
pub use engine::DockerBuildArgs;
