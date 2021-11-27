use std::cell::RefCell;
use std::path::{Path};
use std::thread;
use crate::result::{BatchResult, BatchRunResult};
use crate::runner::Runner;
use crate::entry::{Entry, Expected};

#[derive(Debug)]
pub struct Batch {
    runner: RefCell<Runner>,
    has_run: bool,
}

impl Batch {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Batch {
            runner: RefCell::new(Runner::new()),
            has_run: false,
        }
    }

    pub fn run_match<P: AsRef<Path>>(&self, path: P) {
        self.runner.borrow_mut().add_entry(Entry::new(path, Expected::RunMatch));
    }

    pub fn compile_fail<P: AsRef<Path>>(&self, path: P) {
        self.runner.borrow_mut().add_entry(Entry::new(path, Expected::CompileFail));
    }

    pub fn run(self) -> BatchResult<BatchRunResult> {
        self.runner.borrow_mut().run()
    }
}

#[doc(hidden)]
impl Drop for Batch {
    fn drop(&mut self) {
        if !thread::panicking() && !self.has_run {
            self.runner
                .borrow_mut()
                .run()
                .map(|_| ())
                .unwrap_or_else(|err| println!("{}", err));
        }
    }
}
