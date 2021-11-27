use std::process::Output;

#[derive(Debug)]
pub struct SingleMismatch<T = String> {
    expected: T,
    actual: T,
}

#[derive(Debug)]
pub struct CompileFailMismatch(SingleMismatch);
#[derive(Debug)]
pub struct RunPassMismatch(SingleMismatch<Output>);
#[derive(Debug)]
pub struct RunFailMismatch(SingleMismatch<Output>);

#[derive(Debug)]
pub enum Mismatch {
    CompileFail(CompileFailMismatch),
    RunPass(RunPassMismatch),
    RunFail(RunFailMismatch),
}

impl SingleMismatch {
    // FIXME
    pub fn new() -> Self {
        SingleMismatch {
            expected: String::from(""),
            actual: String::from(""),
        }
    }
}
