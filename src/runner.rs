use crate::binary::BUILDER;
use crate::config::Config;
use crate::entry::{expand_globs, Entry};
use crate::logging;
use crate::result::{BatchResult, BatchRunResult};

use termcolor::{StandardStream, WriteColor};

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

    pub fn run(&mut self) -> BatchResult<BatchRunResult<StandardStream>> {
        let config = Config::from_env()?;
        self.run_with_config(config)
    }
    pub fn run_with_config<W: WriteColor>(
        &mut self,
        cfg: Config<W>,
    ) -> BatchResult<BatchRunResult<W>> {
        let cwd = std::env::current_dir()?;
        std::env::set_current_dir(
            std::env::var_os("CARGO_MANIFEST_DIR").expect("Couldn't get manifest dir"),
        )?;
        let res = self.run_impl(cfg);
        std::env::set_current_dir(cwd)?;
        res
    }
    fn run_impl<W: WriteColor>(&mut self, cfg: Config<W>) -> BatchResult<BatchRunResult<W>> {
        let entries = expand_globs(&self.entries, &cfg.writer());

        let builder = &*BUILDER;

        print!("\n\n");

        if entries.is_empty() {
            let mut log = cfg.writer().build();
            logging::no_entries(&mut log)?;
            Ok(BatchRunResult::NoEntries(Some(log)))
        } else {
            Ok(BatchRunResult::ResultsMap(
                entries
                    .into_iter()
                    .map(|entry| (entry.path().display().to_string(), entry.run(builder, &cfg)))
                    .collect(),
            ))
        }
    }
}
