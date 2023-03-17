use std::io::{self, Write};
use std::process::Command;

fn main() {
    println!("Building {}", env!("CARGO_PKG_NAME"));
    let output = Command::new("cc")
        .args([
            "-O3",
            "-Dmain=native_entry",
            "-fPIC",
            "-I.",
            "-L../../engines/native/",
            "-shared",
            "-o",
            "./target/benchmark.so",
            "benchmark.c",
            "-lengine",
        ])
        .output()
        .expect("failed to compile native benchmark");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let output = Command::new("ln")
        .args(["-f", "-s", "../stdout.expected", "./target/stdout.expected"])
        .output()
        .expect("failed to create symbolic link for stdout.expected");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let output = Command::new("ln")
        .args(["-f", "-s", "../stderr.expected", "./target/stderr.expected"])
        .output()
        .expect("failed to create symbolic link for stderr.expected");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let output = Command::new("ln")
        .args(["-f", "-s", "../default.m.input", "./target/default.m.input"])
        .output()
        .expect("failed to create symbolic link for default.m.input");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let output = Command::new("ln")
        .args(["-f", "-s", "../default.n.input", "./target/default.n.input"])
        .output()
        .expect("failed to create symbolic link for default.n.input");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}