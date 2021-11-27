use crate::mismatch::Mismatch;
use glob::{GlobError, PatternError};
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

pub struct BatchResult(HashMap<String, Result<()>>);

#[derive(Debug)]
pub enum EntryFailed {
    ShouldCompile,
    ShouldNotCompile,
    ExpectedNotExist(NoExpected),
    Mismatch(Mismatch),
    Error(Error),
}

#[derive(Debug)]
pub enum NoExpected {
    ToWip(String),
    Direct(String),
}

#[derive(Debug)]
pub enum Error {
    Cargo(io::Error),
    CargoFail,
    Glob(GlobError),
    Io(io::Error),
    Open(PathBuf, io::Error),
    Pattern(PatternError),
    ReadExpected(io::Error),
    RunFailed(io::Error),
    UpdateVar(OsString),
    WriteExpected(io::Error),
}

pub type Result<T> = std::result::Result<T, EntryFailed>;

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            Cargo(e) => write!(f, "failed to execute cargo: {}", e),
            CargoFail => write!(f, "cargo reported an error"),
            Glob(e) => write!(f, "{}", e),
            Io(e) => write!(f, "{}", e),
            Open(path, e) => write!(f, "{}: {}", path.display(), e),
            Pattern(e) => write!(f, "{}", e),
            ReadExpected(e) => write!(f, "failed to read stderr file: {}", e),
            RunFailed(_) => unimplemented!(),
            UpdateVar(var) => write!(
                f,
                "unrecognized value of BATCH_RUN: {:?}",
                var.to_string_lossy(),
            ),
            WriteExpected(e) => write!(f, "failed to write stderr file: {}", e),
        }
    }
}

impl From<GlobError> for Error {
    fn from(err: GlobError) -> Self {
        Error::Glob(err)
    }
}

impl From<PatternError> for Error {
    fn from(err: PatternError) -> Self {
        Error::Pattern(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl<T: Into<Error>> From<T> for EntryFailed {
    fn from(input: T) -> Self {
        Self::Error(input.into())
    }
}
