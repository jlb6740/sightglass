use anyhow::Result;
use log::info;
use pretty_env_logger;
use sightglass_build::{Artifact, Dockerfile, WasmBenchmark};
use std::env;
use std::path::PathBuf;

// This example tests the crate functionality from end to end.
#[test]
#[ignore]
fn build_benchmark_with_emcc() -> Result<()> {
    pretty_env_logger::init();

    // Build a Wasm benchmark using its Dockerfile.
    let dockerfile = Dockerfile::from(PathBuf::from(
        "./tests/build-benchmark-with-emcc/Dockerfile",
    ));
    let destination_wasm = env::temp_dir().join("benchmark.wasm");
    dockerfile.extract(WasmBenchmark::source(), &destination_wasm, None)?;
    let wasmfile = WasmBenchmark::from(destination_wasm);

    // Verify that the benchmark is a valid one.
    assert!(wasmfile.is_valid().is_ok());

    // Construct the artifact metadata.
    let artifact = Artifact::from(dockerfile, wasmfile);
    info!("Artifact created: {}", serde_json::to_string(&artifact)?);

    Ok(())
}

#[test]
fn interpret_dockerfile() -> Result<()> {
    pretty_env_logger::init();

    // Interpret a Dockerfile in a temp directory and extract a file.
    let dockerfile = Dockerfile::from(PathBuf::from("./tests/interpret-dockerfile/Dockerfile"));
    let source = PathBuf::from("/example.txt");
    let destination = env::temp_dir().join("example.txt");
    dockerfile.interpret(source, destination, None)?;

    Ok(())
}
