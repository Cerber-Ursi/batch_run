use crate::result::{BatchResult, BatchRunResult};
use crate::binary::PreBinary;
use crate::config::Config;
use crate::entry::{Entry, expand_globs};
use crate::logging;

#[derive(Debug, Default)]
pub struct Runner {
    entries: Vec<Entry>,
}

impl Runner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

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
            Ok(BatchRunResult::NoEntries(Some(logging::no_entries()?)))
        } else {
            Ok(BatchRunResult::ResultsMap(
                entries
                    .into_iter()
                    .map(|entry| {
                        (
                            entry.path().display().to_string(),
                            entry.run(&builder, &cfg),
                        )
                    })
                    .collect(),
            ))
        }
    }
}
