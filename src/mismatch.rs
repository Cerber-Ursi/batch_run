use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, error::Error, process::Output};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct LocalOutput {
    status: i32,
    stdout: Vec<String>,
    stderr: Vec<String>,
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
