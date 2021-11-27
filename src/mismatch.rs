#[derive(Debug)]
pub struct Mismatch {
    expected: String,
    actual: String,
}

impl Mismatch {
    // FIXME
    pub fn new() -> Self {
        Self {
            expected: String::from(""),
            actual: String::from(""),
        }
    }
}