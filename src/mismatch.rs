use std::process::Output;

#[derive(Debug)]
struct SingleMismatch<T = String> where T: Clone {
    expected: T,
    actual: T,
}

#[derive(Debug)]
pub struct CompileFailMismatch(SingleMismatch);
#[derive(Debug)]
pub struct RunMismatch(SingleMismatch<Output>);

impl RunMismatch {
    pub fn new(expected: Output, actual: Output) -> Self {
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
