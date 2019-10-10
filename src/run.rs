use std::fs::{self, File};
use std::path::{Path, PathBuf};

use super::{Entry, Expected, Runner};
use crate::batch_result::{BatchResult, BatchRunResult, EntryError, EntryFailed, EntryResult};
use crate::binary::{BinaryBuilder, PreBinary};
use crate::cargo_rustc;
use crate::config::Config;
use crate::message::{self, Fail, Warn};
use crate::mismatch::SingleMismatch;
use crate::normalize::{self, Variations};

impl Runner {
    pub fn run(&mut self) -> BatchResult<BatchRunResult> {
        let config = Config::from_env()?;
        self.run_with_config(config)
    }
    pub fn run_with_config(&mut self, cfg: Config) -> BatchResult<BatchRunResult> {
        let cwd = std::env::current_dir()?;
        std::env::set_current_dir(
            std::env::var_os("CARGO_MANIFEST_DIR").expect("Couldn't get manifest dir"),
        )?;
        let res = self.run_impl(cfg);
        std::env::set_current_dir(cwd)?;
        res
    }
    fn run_impl(&mut self, cfg: Config) -> BatchResult<BatchRunResult> {
        let binary = PreBinary::new()?;

        let entries = expand_globs(&self.entries);

        let builder = binary.into_builder()?;

        print!("\n\n");

        let len = entries.len();
        let mut failures = 0;

        if entries.is_empty() {
            message::no_tests_enabled();
        } else {
            for entry in entries {
                if let Err(err) = entry.run_on(&builder) {
                    failures += 1;
                    message::test_fail(err);
                }
            }
        }

        print!("\n\n");

        if failures > 0 {
            // Err(Error::Batch(format!(
            //     "{} of {} tests failed",
            //     failures, len
            // )))?;
            // TODO
        }
        Ok(BatchRunResult::NoEntries) // FIXME
    }
}

impl Entry {
    fn run(&self, builder: &BinaryBuilder) -> EntryResult<()> {
        message::begin_test(self, true); // TODO
        check_exists(&self.path)?;

        let output = cargo_rustc::build_entry(builder, &self.path, self.expected.is_run_pass())?;
        let success = output.status.success();
        let stdout = output.stdout;
        let stderr = normalize::diagnostics(output.stderr);

        let check = match self.expected {
            Expected::RunPass => Entry::check_pass,
            Expected::CompileFail => Entry::check_compile_fail,
            Expected::RunFail => unimplemented!(),
        };

        check(self, success, stdout, stderr)
    }

    fn check_pass(
        &self,
        success: bool,
        build_stdout: Vec<u8>,
        variations: Variations,
    ) -> EntryResult<()> {
        let preferred = variations.preferred();
        if !success {
            message::failed_to_build(preferred);
            Err(EntryFailed::ShouldCompile)?;
        }

        let mut output = cargo_rustc::run_entry()?;
        output.stdout.splice(..0, build_stdout);
        message::output(preferred, &output);
        if output.status.success() {
            Ok(())
        } else {
            Err(EntryFailed::ShouldCompile)? // TODO
        }
    }

    fn check_compile_fail(
        &self,
        success: bool,
        build_stdout: Vec<u8>,
        variations: Variations,
    ) -> EntryResult<()> {
        let preferred = variations.preferred();

        if success {
            message::should_not_have_compiled();
            message::fail_output(Fail, &build_stdout);
            message::warnings(preferred);
            Err(EntryFailed::ShouldNotCompile)?;
        }

        let stderr_path = self.path.with_extension("stderr");

        if !stderr_path.exists() {
            // match project.update {
            //     Update::Wip => {
            //         let wip_dir = Path::new("wip");
            //         fs::create_dir_all(wip_dir)?;
            //         let gitignore_path = wip_dir.join(".gitignore");
            //         fs::write(gitignore_path, "*\n")?;
            //         let stderr_name = stderr_path
            //             .file_name()
            //             .unwrap_or_else(|| OsStr::new("test.stderr"));
            //         let wip_path = wip_dir.join(stderr_name);
            //         message::write_stderr_wip(&wip_path, &stderr_path, preferred);
            //         fs::write(wip_path, preferred).map_err(Error::WriteStderr)?;
            //     }
            //     Update::Overwrite => {
            //         message::overwrite_stderr(&stderr_path, preferred);
            //         fs::write(stderr_path, preferred).map_err(Error::WriteStderr)?;
            //     }
            // }
            message::fail_output(Warn, &build_stdout);
            return Ok(());
        }

        let expected = fs::read_to_string(&stderr_path)
            .map_err(EntryError::ReadExpected)?
            .replace("\r\n", "\n");

        if variations.any(|stderr| expected == stderr) {
            message::ok();
            return Ok(());
        }

        // match project.update {
        //     Update::Wip => {
        message::mismatch(&expected, preferred);
        Err(EntryFailed::Mismatch(unimplemented!()))
        //     }
        //     Update::Overwrite => {
        //         message::overwrite_stderr(&stderr_path, preferred);
        //         fs::write(stderr_path, preferred).map_err(Error::WriteStderr)?;
        //         Ok(())
        //     }
        // }
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
struct ExpandedEntry {
    raw_entry: Entry,
    error: Option<EntryFailed>,
}

fn expand_globs(tests: &[Entry]) -> Vec<ExpandedEntry> {
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
    fn run_on(self, builder: &BinaryBuilder) -> EntryResult<()> {
        match self.error {
            None => self.raw_entry.run(builder),
            Some(error) => {
                let show_expected = false;
                message::begin_test(&self.raw_entry, show_expected);
                Err(error)
            }
        }
    }
}
