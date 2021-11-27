//! Normalization primitives for rustc output.
//!
//! Since we're using rustc directly (not cargo), it seems that there is little boilerplate
//! in the compilation output, but it still appears to be.
//! This module is designed to provide a way to remove the unnecessary lines,
//! so they won't appear in either the *.stderr files or in the processing output.

/// Possible normalizations of the rustc output, arranged from the least to the most preferable.
///
/// For now, there's only one, but we should keep this as a way to generalize later.
#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum Normalization {
    Basic,
}
use self::Normalization::*;

/// Helper struct for variations of normalized text.
///
/// When the output is passed through different normalizations, we must have a way to check
/// if some of them has provided the correct output (for backwards compatibility purposes).
/// Of course, we can use `Vec<String>` directly, but this struct provides several helper
/// methods to make the checks more smooth.
pub struct Variations {
    variations: Vec<String>,
}

impl Variations {
    /// Get the preferred variation which will be written in the .stderr file.
    pub fn preferred(&self) -> &str {
        self.variations.last().unwrap()
    }

    /// Check if any existing variation satisfies the provided predicate.
    pub fn any<F: FnMut(&str) -> bool>(&self, mut f: F) -> bool {
        self.variations.iter().any(|stderr| f(stderr))
    }
}

/// Generate the `Variations` object from the raw stderr output.
pub fn diagnostics(output: &[u8]) -> Variations {
    let from_bytes = String::from_utf8_lossy(output)
        .to_string()
        .replace("\r\n", "\n");

    let variations = [Basic].iter().map(|_| process(&from_bytes)).collect();

    Variations { variations }
}

fn process(original: &str) -> String {
    let mut normalized = String::new();

    for line in original.lines() {
        if let Some(line) = filter_map(line) {
            normalized += &line;
            if !normalized.ends_with("\n\n") {
                normalized.push('\n');
            }
        }
    }

    trim(normalized)
}

fn filter_map(line: &str) -> Option<String> {
    lazy_static::lazy_static! {
        static ref CUT_OUT: Vec<&'static str> = vec![
            "error: aborting due to",
            "For more information about this error, try `rustc --explain",
            "For more information about an error, try `rustc --explain",
            "Some errors have detailed explanations:",
        ];
    };
    // stripping out final compilation lines
    if CUT_OUT.iter().any(|prefix| line.trim().starts_with(prefix)) {
        None
    } else {
        Some(line.to_owned())
    }
}

/// Trim the bytes stream with minimal reallocations.
pub fn trim<S: AsRef<[u8]>>(output: S) -> String {
    let bytes = output.as_ref();
    let mut normalized = String::from_utf8_lossy(bytes).to_string();

    let len = normalized.trim_end().len();
    normalized.truncate(len);

    if !normalized.is_empty() {
        normalized.push('\n');
    }

    normalized
}
