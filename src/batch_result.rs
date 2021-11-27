use crate::mismatch::{CompileFailMismatch, RunMismatch};
use glob::{GlobError, PatternError};
use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub enum BatchRunResult {
    NoEntries,
    ResultsMap(HashMap<String, EntryResult>),
}
pub type BatchResult<T = BatchRunResult> = std::result::Result<T, BatchError>;

impl BatchRunResult {
    pub fn errors(&self) -> Option<Vec<(&String, &EntryFailed)>> {
        if let BatchRunResult::ResultsMap(map) = self {
            Some(
                map.iter()
                    .filter_map(|(file, res)| res.as_ref().err().map(|err| (file, err)))
                    .collect(),
            )
        } else {
            None
        }
    }
    pub fn all_ok(&self) -> bool {
        match self.errors() {
            Some(errors) => errors.len() == 0,
            None => true, // TODO configure?
        }
    }
    pub fn assert_all_ok(&self) {
        let errors = match self.errors() {
            Some(errors) => errors,
            None => return,
        };
        if errors.len() > 0 {
            for (file, err) in errors.into_iter() {
                eprintln!("{} => {}", file, err);
            }
            panic!("Assertion failed, see errors in stderr above");
        }
    }
}

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
    #[error("led to execute cargo: {0}")]
    Cargo(#[source] io::Error),
    #[error("General IO error: {0}")]
    Io(#[source] io::Error),
    #[error("Incorrect value of BATCH_RUN environmental variable: expected either \"Overwrite\" or \"Wip\", got {}", .0.to_string_lossy())]
    UpdateVar(OsString),
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
    // TODO - is it used/necessary?
    #[error("Incorrect glob pattern: {0}")]
    Pattern(#[source] PatternError),
    #[error("Error reading snapshot: {0}")]
    ReadExpected(#[source] io::Error),
    #[error("Cannot execute compiled binary: {0}")]
    RunFailed(#[source] io::Error),
    #[error("Error writing snapshot: {0}")]
    WriteExpected(#[source] io::Error),
}

pub type EntryResult<T = ()> = std::result::Result<T, EntryFailed>;

impl From<io::Error> for BatchError {
    fn from(err: io::Error) -> Self {
        BatchError::Io(err)
    }
}

impl From<GlobError> for EntryError {
    fn from(err: GlobError) -> Self {
        EntryError::Glob(err)
    }
}

impl From<PatternError> for EntryError {
    fn from(err: PatternError) -> Self {
        EntryError::Pattern(err)
    }
}

impl From<io::Error> for EntryError {
    fn from(err: io::Error) -> Self {
        EntryError::Io(err)
    }
}

impl<T: Into<EntryError>> From<T> for EntryFailed {
    fn from(input: T) -> Self {
        Self::Error(input.into())
    }
}
