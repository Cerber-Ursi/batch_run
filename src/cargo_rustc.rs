use lazy_static::lazy_static;
use std::{
    env::consts::EXE_EXTENSION,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use crate::binary::BinaryBuilder;
use crate::error::{Error, Result};
use crate::rustflags;

lazy_static! {
    static ref TARGET_BIN: PathBuf = {
        let tmp: PathBuf = ["target", "batch", ""].into_iter().collect();
        tmp.with_extension(EXE_EXTENSION)
    };
}

include!(concat!(env!("OUT_DIR"), "/info.rs"));

fn raw_cargo() -> Command {
    Command::new(option_env!("CARGO").unwrap_or("cargo"))
}

fn rustc() -> Command {
    let mut cmd = Command::new(info::rustc());
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    cmd.args(&[
        "-o",
        TARGET_BIN.to_str().expect("Non-UTF-8 symbols in path"),
    ]);
    cmd
}

pub fn capture_build_command(bin_name: &str) -> Result<String> {
    let mut cmd = raw_cargo();
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));
    rustflags::set_env(&mut cmd);
    cmd.arg("build");
    if info::opt_level() == "release" {
        cmd.arg("--release");
    };
    cmd.arg("--bin")
        .arg(bin_name)
        .arg("--verbose")
        .output()
        .map_err(Error::Cargo)
        // .map(|out| { println!("Cargo output: \"{}\"", String::from_utf8(out.stderr).unwrap()); "rustc".to_owned() })
        .map(extract_build_command)
        .map(trim_build_command)
}

fn extract_build_command(out: Output) -> String {
    String::from_utf8(out.stderr)
        .expect("Cargo produced non-UTF-8 output")
        .lines()
        .filter(|line| line.trim_start().starts_with("Running `"))
        .last()
        .expect("No running command in cargo output")
        .to_owned()
}

fn trim_build_command(line: String) -> String {
    line.trim_start_matches("Running")
        .trim()
        .trim_matches('`')
        .to_owned()
}

pub fn build_test(builder: &BinaryBuilder, main: &Path, run: bool) -> Result<Output> {
    let mut cmd = rustc();
    builder.args_to_command(&mut cmd, main);
    cmd.arg(if run {
        "--emit=link"
    } else {
        "--emit=dep-info"
    });
    cmd.output().map_err(Error::Cargo)
}

pub fn run_test() -> Result<Output> {
    Command::new(&*TARGET_BIN)
        .output()
        .map_err(|_| Error::RunFailed)
}
