use super::Runner;
use crate::batch_result::{BatchResult, BatchRunResult};
use crate::binary::PreBinary;
use crate::config::Config;
use crate::entry::expand_globs;
//use crate::message;

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

        if entries.is_empty() {
//            message::no_tests_enabled();
            Ok(BatchRunResult::NoEntries)
        } else {
            // for entry in entries {
            //     if let Err(err) = entry.run(&builder, &cfg) {
            //         failures += 1;
            //         message::test_fail(err);
            //     }
            // }
            Ok(BatchRunResult::ResultsMap(entries.into_iter().map(|entry| {
                (entry.path().display().to_string(), entry.run(&builder, &cfg))
            }).collect()))
        }

        // print!("\n\n");

        // if failures > 0 {
        //     // Err(Error::Batch(format!(
        //     //     "{} of {} tests failed",
        //     //     failures, len
        //     // )))?;
        //     // TODO
        // }
    }
}
