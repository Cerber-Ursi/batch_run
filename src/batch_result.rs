use crate::mismatch::{CompileFailMismatch, RunMismatch};
use glob::{GlobError, PatternError};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::{self, Display};
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
            Some(map.iter().filter_map(|(file, res)| res.as_ref().err().map(|err| (file, err))).collect())
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
    // TODO - include error message?
    #[error("Entry should compile, but compilation failed")]
    ShouldCompile,
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
//    #[error("Cargo has not finished successfully")]
    Cargo(#[source] io::Error),
//    #[error("General IO error")]
    Io(#[source] io::Error),
//    #[error("Incorrect value of BATCH_RUN environmental variable: expected either \"Overwrite\" or \"Wip\", got {0}")]
    UpdateVar(OsString),
}

#[derive(Debug, Error)]
pub enum EntryError {
//    #[error("Rustc has not finished successfully")]
    Rustc(#[source] io::Error),
    // TODO - is it used/necessary?
//    #[error("Cargo failed to run")]
    CargoFail,
//    #[error("Incorrect glob provided")]
    Glob(#[source] GlobError),
//    #[error("General IO error")]
    Io(#[source] io::Error),
//    #[error("Unable to open provided path: {0}")]
    Open(PathBuf, #[source] io::Error),
    // TODO - is it used/necessary?
//    #[error("Incorrect pattern")]
    Pattern(#[source] PatternError),
//    #[error("Error reading snapshot")]
    ReadExpected(#[source] io::Error),
//    #[error("Cannot execute compiled binary")]
    RunFailed(#[source] io::Error),
//    #[error("Error writing snapshot")]
    WriteExpected(#[source] io::Error),
}

pub type EntryResult<T = ()> = std::result::Result<T, EntryFailed>;

impl Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::EntryError::*;

        match self {
            Rustc(e) => write!(f, "failed to execute rustc: {}", e),
            CargoFail => write!(f, "cargo reported an error"),
            Glob(e) => write!(f, "{}", e),
            Io(e) => write!(f, "{}", e),
            Open(path, e) => write!(f, "{}: {}", path.display(), e),
            Pattern(e) => write!(f, "{}", e),
            ReadExpected(e) => write!(f, "failed to read stderr file: {}", e),
            RunFailed(_) => unimplemented!(),
            WriteExpected(e) => write!(f, "failed to write stderr file: {}", e),
        }
    }
}

impl Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BatchError::*;

        match self {
            Cargo(e) => write!(f, "failed to execute cargo: {}", e),
            UpdateVar(var) => write!(
                f,
                "unrecognized value of BATCH_RUN: {:?}",
                var.to_string_lossy(),
            ),
            Io(e) => write!(f, "{}", e),
        }
    }
}

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
