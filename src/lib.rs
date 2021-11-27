//! Batch Run
//! =========
//!
//! [![Latest Version](https://img.shields.io/crates/v/batch_run.svg)](https://crates.io/crates/batch_run)
//! [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/batch_run)
//!
//! `batch_run` is a runner for a set of Rust source files, based on [dtolnay's `trybuild`](https://github.com/dtolnay/trybuild).
//! It can be useful when you have a bunch of Rust sources which are not complex enough to be
//! packed into dedicated crates, but which are (by their meaning) not just integration test cases.
//! It also checks for output correctness, either on compile-time (for `compile_fail` cases)
//! or at runtime (for `run_pass` cases).
//!
//! ```toml
//! [dependencies]
//! batch_run = "1.0"
//! ```
//!
//! *Compiler support: requires rustc 1.31+*
//!
//! <br>
//!
//! ## Compile-fail cases
//!
//! A minimal batch_run setup looks like this:
//!
//! ```rust
//! fn main() {
//!     let b = batch_run::Batch::new();
//!     b.compile_fail("batches/ui/*.rs");
//!     match b.run() {
//!         Ok(()) => {},
//!         Err(err) => println!("{:?}", err)
//!     };
//! }
//! ```
//!
//! This program will individually compile each of the
//! source files matching the glob pattern, expect them to fail to compile, and
//! assert that the compiler's error message matches an adjacently named _*.stderr_
//! file containing the expected output (same file name as the test except with a
//! different extension). If it doesn't match, the program will print the error message
//! with expected vs actual compiler output.
//!
//! Dependencies listed under `[dependencies]` in the project's Cargo.toml are
//! accessible from within the batch.
//!
//! A compile\_fail case that fails to fail to compile is also a failure.
//!
//! <br>
//!
//! ## Run-pass cases
//!
//! In the run_pass cases, we not only check that the code compiles, but also actually run it
//! and match the stdout/stderr output with the corresponding _*.stdout_/_*.stderr_ files.
//!
//! You can mix compile_fail and run_pass cases in one batch:
//!
//! ```rust
//! fn main() {
//!     let t = batch_run::Batch::new();
//!     t.run_pass("batches/01-parse-header.rs");
//!     t.run_pass("batches/02-parse-body.rs");
//!     t.compile_fail("batches/03-expand-four-errors.rs");
//!     t.run_pass("batches/04-paste-ident.rs");
//!     t.run_pass("batches/05-repeat-section.rs");
//! }
//! ```
//!
//! <br>
//!
//! ## Details
//!
//! That's the entire API for now.
//!
//! <br>
//!
//! ## Workflow
//!
//! (TODO)
//!

#[macro_use]
mod term;

mod batch_result;
mod binary;
mod cargo_rustc;
mod config;
mod message;
mod mismatch;
mod normalize;
mod run;
mod rustflags;

use crate::batch_result::BatchResult;
use crate::batch_result::BatchRunResult;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
// use std::thread;

#[derive(Debug)]
pub struct Batch {
    runner: RefCell<Runner>,
}

#[derive(Debug)]
struct Runner {
    entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
struct Entry {
    path: PathBuf,
    expected: Expected,
}

#[derive(Copy, Clone, Debug)]
enum Expected {
    RunPass,
    RunFail,
    CompileFail,
}

impl Expected {
    pub fn is_run_pass(&self) -> bool {
        use Expected::*;
        match self {
            RunPass => true,
            RunFail => true,
            CompileFail => false,
        }
    }
}

impl Batch {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Batch {
            runner: RefCell::new(Runner {
                entries: Vec::new(),
            }),
        }
    }

    pub fn run_pass<P: AsRef<Path>>(&self, path: P) {
        self.runner.borrow_mut().entries.push(Entry {
            path: path.as_ref().to_owned(),
            expected: Expected::RunPass,
        });
    }

    pub fn compile_fail<P: AsRef<Path>>(&self, path: P) {
        self.runner.borrow_mut().entries.push(Entry {
            path: path.as_ref().to_owned(),
            expected: Expected::CompileFail,
        });
    }

    // TODO error type
    pub fn run(self) -> BatchResult<BatchRunResult> {
        self.runner.borrow_mut().run()
    }
}

// #[doc(hidden)]
// impl Drop for Batch {
//     fn drop(&mut self) {
//         if !thread::panicking() {
//             self.runner
//                 .borrow_mut()
//                 .run()
//                 .unwrap_or_else(|err| println!("{}", err));
//         }
//     }
// }
