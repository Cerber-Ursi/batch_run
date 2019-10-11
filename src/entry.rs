use std::fs::File;
use std::path::{Path, PathBuf};

use super::{Entry, Expected};
use crate::batch_result::{EntryError, EntryFailed, EntryResult};
use crate::binary::BinaryBuilder;
use crate::cargo_rustc;
use crate::config::Config;
use crate::message;
use crate::snapshot::{check_compile_fail, check_run_match};

impl Entry {
    fn run(&self, builder: &BinaryBuilder, cfg: &Config) -> EntryResult<()> {
        message::begin_test(self, true); // TODO
        check_exists(&self.path)?;

        let mut output =
            cargo_rustc::build_entry(builder, &self.path, self.expected.is_run_pass())?;

        let check = match self.expected {
            Expected::RunMatch => {
                // early exit if the entry has not compiled
                if !output.status.success() {
                    Err(EntryFailed::ShouldCompile(
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ))?;
                }
                output = cargo_rustc::run_entry()?;
                check_run_match
            }
            Expected::CompileFail => check_compile_fail,
        };
        check(&self.path, output, cfg.update_mode())
    }
}

fn check_exists(path: &Path) -> EntryResult<()> {
    if path.exists() {
        return Ok(());
    }
    match File::open(path) {
        Ok(_) => Ok(()),
        Err(err) => Err(EntryError::Open(path.to_owned(), err))?,
    }
}

#[derive(Debug)]
pub struct ExpandedEntry {
    raw_entry: Entry,
    error: Option<EntryFailed>,
}

pub(crate) fn expand_globs(tests: &[Entry]) -> Vec<ExpandedEntry> {
    fn glob(pattern: &str) -> EntryResult<Vec<PathBuf>> {
        let mut paths = glob::glob(pattern)?
            .map(|entry| entry.map_err(EntryFailed::from))
            .collect::<EntryResult<Vec<PathBuf>>>()?;
        paths.sort();
        Ok(paths)
    }

    let mut vec = Vec::new();

    for test in tests {
        let mut expanded = ExpandedEntry {
            raw_entry: test.clone(),
            error: None,
        };
        if let Some(utf8) = test.path.to_str() {
            if utf8.contains('*') {
                match glob(utf8) {
                    Ok(paths) => {
                        for path in paths {
                            vec.push(ExpandedEntry {
                                raw_entry: Entry {
                                    path,
                                    expected: expanded.raw_entry.expected,
                                },
                                error: None,
                            });
                        }
                        continue;
                    }
                    Err(error) => expanded.error = Some(error),
                }
            }
        }
        vec.push(expanded);
    }

    vec
}

impl ExpandedEntry {
    pub fn run(self, builder: &BinaryBuilder, cfg: &Config) -> EntryResult<()> {
        match self.error {
            None => self.raw_entry.run(builder, cfg),
            Some(error) => {
                let show_expected = false;
                message::begin_test(&self.raw_entry, show_expected);
                Err(error)
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.raw_entry.path
    }
}
