use indicatif::{ProgressBar, ProgressStyle};
use std::io::IsTerminal;
use std::time::Duration;

/// Activity spinner that writes to stderr.
///
/// The spinner is only displayed when stderr is connected to a terminal
/// (i.e. not piped or redirected) and progress output has not been
/// suppressed via `--no-progress`.
pub struct Spinner {
    bar: Option<ProgressBar>,
}

impl Spinner {
    /// Create a new spinner showing `message`.
    ///
    /// The spinner is a no-op when `enabled` is `false` or when stderr is
    /// not a terminal, so output piped to a file or another process is
    /// never disturbed.
    pub fn new(message: &str, enabled: bool) -> Self {
        if enabled && std::io::stderr().is_terminal() {
            let bar = ProgressBar::new_spinner();
            bar.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner()),
            );
            bar.set_message(message.to_string());
            bar.enable_steady_tick(Duration::from_millis(100));
            Self { bar: Some(bar) }
        } else {
            Self { bar: None }
        }
    }

    /// Remove the spinner from the terminal.
    ///
    /// Safe to call multiple times; subsequent calls are no-ops.
    pub fn finish_and_clear(&self) {
        if let Some(bar) = &self.bar {
            bar.finish_and_clear();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.finish_and_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_disabled() {
        // When disabled the spinner is a no-op and should not panic.
        let spinner = Spinner::new("working...", false);
        spinner.finish_and_clear();
    }

    #[test]
    fn test_spinner_no_terminal() {
        // In CI / test environments stderr is not a terminal, so the
        // internal ProgressBar should be None regardless of `enabled`.
        let spinner = Spinner::new("working...", true);
        // We cannot assert stderr.is_terminal() here (it depends on the
        // runner), but we can verify the public API doesn't panic.
        spinner.finish_and_clear();
    }
}
