//! The shape of each failure: a cause, a fix, and an exit code (CV-1, CV-3).

/// One failure that jag shows to the user.
///
/// The text never holds the API key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub cause: String,
    /// One line that the user can act on.
    pub fix: String,
    /// A code of IF-4.
    pub exit_code: u8,
}

impl Failure {
    /// The text for the error stream: `jag: <cause>` and then `<fix>`.
    pub fn render(&self) -> String {
        format!("jag: {}\n{}\n", self.cause, self.fix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_gives_cause_then_fix() {
        let failure = Failure {
            cause: "no key".into(),
            fix: "Run it again.".into(),
            exit_code: 3,
        };
        assert_eq!(failure.render(), "jag: no key\nRun it again.\n");
    }
}
