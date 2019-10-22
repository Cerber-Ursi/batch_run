use itertools::{EitherOrBoth, Itertools};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, error::Error, process::Output};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct LocalOutput {
    status: i32,
    stdout: Vec<String>,
    stderr: Vec<String>,
}
impl LocalOutput {
    // This is an *extremely* hacky thing.
    // In fact, I'm ignoring every backslash in the output by replacing them with forward slashes,
    // so that the pathes, if the program writes them (either correctly or during panic) are
    // compared indepentently of the platform separator.
    // I'm not really sure if this is a way to go, but...
    pub fn matches(&self, other: &LocalOutput) -> bool {
        self.status == other.status
            && match_lines_with_backslashes(&self.stdout, &other.stdout)
            && match_lines_with_backslashes(&self.stderr, &other.stderr)
    }
}
impl TryFrom<Output> for LocalOutput {
    // TODO make some real error and propagate it
    type Error = Box<dyn Error>;
    fn try_from(input: Output) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            status: input.status.code().ok_or("No status code")?,
            stdout: bytes_to_lines(&input.stdout),
            stderr: bytes_to_lines(&input.stderr),
        })
    }
}
fn bytes_to_lines(input: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(input)
        .to_string()
        .replace("\r\n", "\n")
        .lines()
        .map(String::from)
        .collect()
}
fn match_lines_with_backslashes(left: &[String], right: &[String]) -> bool {
    left.iter().zip_longest(right).all(|pair| {
        if let EitherOrBoth::Both(left, right) = pair {
            match_with_backslashes(left, right)
        } else {
            false
        }
    })
}
fn match_with_backslashes(left: &str, right: &str) -> bool {
    left.replace('\\', "/") == right.replace('\\', "/")
}

#[derive(Debug)]
struct SingleMismatch<T = String>
where
    T: Clone,
{
    expected: T,
    actual: T,
}

#[derive(Debug)]
pub struct CompileFailMismatch(SingleMismatch);
#[derive(Debug)]
pub struct RunMismatch(SingleMismatch<LocalOutput>);

impl RunMismatch {
    pub fn new(expected: LocalOutput, actual: LocalOutput) -> Self {
        RunMismatch(SingleMismatch { expected, actual })
    }
}

impl CompileFailMismatch {
    pub fn new<S1: Into<String>, S2: Into<String>>(expected: S1, actual: S2) -> Self {
        CompileFailMismatch(SingleMismatch {
            expected: expected.into(),
            actual: actual.into(),
        })
    }
}
