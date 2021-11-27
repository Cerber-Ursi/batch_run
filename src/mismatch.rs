use std::{convert::TryFrom, error::Error, process::Output};
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct LocalOutput {
    status: i32,
    stdout: String,
    stderr: String,
}
impl TryFrom<Output> for LocalOutput {
    // TODO make some real error and propagate it
    type Error = Box<dyn Error>;
    fn try_from(input: Output) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            status: input.status.code().ok_or("No status code")?,
            stdout: String::from_utf8_lossy(&input.stdout).to_string(),
            stderr: String::from_utf8_lossy(&input.stderr).to_string(),
        })
    }
}

#[derive(Debug)]
struct SingleMismatch<T = String> where T: Clone {
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
