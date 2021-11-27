use crate::term;
use glob::{GlobError, PatternError};
use std::collections::HashMap;
use std::io;
use termcolor::Buffer;

pub mod error;
use error::*;

pub enum BatchRunResult {
    NoEntries(Option<Buffer>),
    ResultsMap(HashMap<String, EntryOutput>),
}
pub type BatchResult<T = BatchRunResult> = std::result::Result<T, BatchError>;

impl BatchRunResult {
    pub fn errors(&self) -> Option<Vec<(&String, &EntryFailed)>> {
        if let BatchRunResult::ResultsMap(map) = self {
            Some(
                map.iter()
                    .filter_map(|(file, res)| res.err().map(|err| (file, err)))
                    .collect(),
            )
        } else {
            None
        }
    }
    pub fn all_ok(&self) -> bool {
        match self.errors() {
            Some(errors) => errors.is_empty(),
            None => true, // TODO configure?
        }
    }
    pub fn assert_all_ok(&self) {
        let errors = match self.errors() {
            Some(errors) => errors,
            None => return,
        };
        if !errors.is_empty() {
            for (file, err) in errors.into_iter() {
                eprintln!("{} => {}", file, err);
            }
            panic!("Assertion failed, see errors in stderr above");
        }
    }
    pub fn print_all(&mut self) -> std::result::Result<(), PrintError> {
        match self {
            BatchRunResult::NoEntries(buf) => term::print(buf.take()),
            BatchRunResult::ResultsMap(map) => map.values_mut().map(EntryOutput::print).collect(),
        }
    }
}

pub type EntryResult<T = ()> = std::result::Result<T, EntryFailed>;
pub struct EntryOutput {
    res: EntryResult,
    buf: Option<Buffer>,
}
impl EntryOutput {
    pub(crate) fn new(res: EntryResult, buf: Buffer) -> Self {
        Self {
            res,
            buf: Some(buf),
        }
    }
    pub fn is_ok(&self) -> bool {
        self.res.is_ok()
    }
    pub fn err(&self) -> Option<&EntryFailed> {
        self.res.as_ref().err()
    }
    pub fn print(&mut self) -> std::result::Result<(), PrintError> {
        term::print(self.buf.take())
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
