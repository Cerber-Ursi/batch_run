use crate::mismatch::{CompileFailMismatch, RunMismatch};
use glob::{GlobError, PatternError};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

pub enum BatchRunResult {
    NoEntries,
    ResultsMap(HashMap<String, EntryResult<()>>),
}
pub type BatchResult<T> = std::result::Result<T, BatchError>;

#[derive(Debug)]
pub enum EntryFailed {
    ShouldCompile,
    ShouldNotCompile,
    ExpectedNotExist(NoExpected),
    CompileFailMismatch(CompileFailMismatch),
    RunMismatch(RunMismatch),
    Error(EntryError),
}

#[derive(Debug)]
pub enum NoExpected {
    ToWip(String),
    Direct(String),
}

#[derive(Debug)]
pub enum BatchError {
    Cargo(io::Error),
    Io(io::Error),
    UpdateVar(OsString),
}

#[derive(Debug)]
pub enum EntryError {
    Rustc(io::Error),
    CargoFail,
    Glob(GlobError),
    Io(io::Error),
    Open(PathBuf, io::Error),
    Pattern(PatternError),
    ReadExpected(io::Error),
    RunFailed(io::Error),
    WriteExpected(io::Error),
}

pub type EntryResult<T> = std::result::Result<T, EntryFailed>;

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
