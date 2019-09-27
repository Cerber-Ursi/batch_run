use crate::mismatch::Mismatch;
use glob::{GlobError, PatternError};
use std::env;
use std::ffi::OsString;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct BatchResult(HashMap<String, EntryOutcome>);

#[derive(Debug)]
pub enum EntryOutcome {
    Warning(Warning),
    Error(Error),
}

impl From<Warning> for EntryOutcome {
    fn from(warn: Warning) -> Self {
        Self::Warning(warn)
    }
}

impl<T> From<T> for EntryOutcome where Error: From<T> {
    fn from(err: T) -> Self {
        Self::Error(err.into())
    }
}

// TODO: what are the contents?
#[derive(Debug)]
pub enum Warning {
    Wip(String),
    Overwritten(String),
}

#[derive(Debug)]
pub enum Error {
    Cargo(io::Error),
    CargoFail,
    Glob(GlobError),
    Io(io::Error),
    Mismatch(Mismatch),
    Open(PathBuf, io::Error),
    Pattern(PatternError),
    PkgName(env::VarError),
    ProjectDir,
    ReadStderr(io::Error),
    RunFailed(String),
    ShouldNotHaveCompiled,
    UpdateVar(OsString),
    WriteStderr(io::Error),
}

pub type Result<T> = std::result::Result<T, EntryOutcome>;

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            Cargo(e) => write!(f, "failed to execute cargo: {}", e),
            CargoFail => write!(f, "cargo reported an error"),
            Glob(e) => write!(f, "{}", e),
            Io(e) => write!(f, "{}", e),
            Mismatch(_) => write!(f, "compiler error does not match expected error"), // TODO
            Open(path, e) => write!(f, "{}: {}", path.display(), e),
            Pattern(e) => write!(f, "{}", e),
            PkgName(e) => write!(f, "failed to detect CARGO_PKG_NAME: {}", e),
            ProjectDir => write!(f, "failed to determine name of project dir"),
            ReadStderr(e) => write!(f, "failed to read stderr file: {}", e),
            RunFailed(_) => write!(f, "execution of the test case was unsuccessful"), // TODO
            ShouldNotHaveCompiled => {
                write!(f, "expected test case to fail to compile, but it succeeded")
            }
            UpdateVar(var) => write!(
                f,
                "unrecognized value of TRYBUILD: {:?}",
                var.to_string_lossy(),
            ),
            WriteStderr(e) => write!(f, "failed to write stderr file: {}", e),
        }
    }
}

impl Error {
    // TODO - is this necessary?
    pub fn already_printed(&self) -> bool {
        use self::Error::*;

        match self {
            CargoFail | Mismatch(_) | RunFailed(_) | ShouldNotHaveCompiled => true,
            _ => false,
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
