use crate::mismatch::{CompileFailMismatch, RunMismatch};
use glob::{GlobError, PatternError};
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EntryFailed {
    #[error("Entry should compile, but compilation failed; error message:\n{0}")]
    ShouldCompile(String),
    #[error("Entry should not compile, but it compiled successfully")]
    ShouldNotCompile,
    #[error("There's no expected output for entry. {0}")]
    ExpectedNotExist(#[source] NoExpected),
    #[error("Compiler error mismatch")]
    CompileFailMismatch(CompileFailMismatch),
    #[error("Runtime output mismatch")]
    RunMismatch(RunMismatch),
    #[error("Internal error")]
    Error(#[source] EntryError),
}

#[derive(Debug, Error)]
pub enum NoExpected {
    #[error("Output written to WIP folder")]
    ToWip(String),
    #[error("Output written directly to snapshot")]
    Direct(String),
}

#[derive(Debug, Error)]
pub enum BatchError {
    #[error("Failed to execute cargo: {0}")]
    Cargo(#[source] io::Error),
    #[error("Configuration error: {0}")]
    ConfigError(#[source] ConfigError),
    #[error("General IO error: {0}")]
    Io(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Incorrect value of BATCH_RUN environmental variable: expected either \"Overwrite\" or \"Wip\", got {}", .0.to_string_lossy())]
    UpdateEnvVar(OsString),
}

#[derive(Debug, Error)]
pub enum EntryError {
    #[error("Failed to execute rustc: {0}")]
    Rustc(#[source] io::Error),
    // TODO - is it used/necessary?
    #[error("Cargo reported an error")]
    CargoFail,
    #[error("Error executing glob: {0}")]
    Glob(#[source] GlobError),
    #[error("General IO error: {0}")]
    Io(#[source] io::Error),
    #[error("Unable to open provided path: {}, error: {}", .0.display(), .1)]
    Open(PathBuf, #[source] io::Error),
    #[error("Incorrect glob pattern: {0}")]
    Pattern(#[source] PatternError),
    #[error("Error reading snapshot: {0}")]
    ReadExpected(#[source] io::Error),
    #[error("Cannot execute compiled binary: {0}")]
    RunFailed(#[source] io::Error),
    #[error("Error writing snapshot: {0}")]
    WriteExpected(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum PrintError {
    #[error("The internal buffer was already printed")]
    AlreadyPrinted,
    #[error("I/O error while printing: {0}")]
    Io(#[source] std::io::Error),
}