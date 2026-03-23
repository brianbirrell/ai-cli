use std::{
    io::{IsTerminal, Write},
    sync::mpsc,
    time::Duration,
};

const TICK_MS: u64 = 100;
const FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Activity spinner that writes to stderr.
///
/// The spinner is only displayed when stderr is connected to a terminal
/// (i.e. not piped or redirected) and progress output has not been
/// suppressed via `--no-progress`.
pub struct Spinner {
    /// Dropping or taking the sender signals the spinner thread to stop.
    cancel_tx: Option<mpsc::SyncSender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Spinner {
    /// Create a new spinner showing `message`.
    ///
    /// The spinner is a no-op when `enabled` is `false` or when stderr is
    /// not a terminal, so output piped to a file or another process is
    /// never disturbed.
    pub fn new(message: &str, enabled: bool) -> Self {
        if !enabled || !std::io::stderr().is_terminal() {
            return Self {
                cancel_tx: None,
                thread: None,
            };
        }

        // capacity=1 so the implicit drop-send in finish_and_clear never blocks
        let (tx, rx) = mpsc::sync_channel::<()>(1);
        let msg = message.to_string();

        let handle = std::thread::spawn(move || {
            let stderr = std::io::stderr();
            let mut idx = 0usize;

            loop {
                let frame = FRAMES[idx % FRAMES.len()];
                {
                    let mut err = stderr.lock();
                    let _ = write!(err, "\r{} {}", frame, msg);
                    let _ = err.flush();
                }
                idx += 1;

                // Wait for the next tick or a cancel/disconnect signal.
                match rx.recv_timeout(Duration::from_millis(TICK_MS)) {
                    Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                }
            }

            // Erase the spinner line so subsequent output starts cleanly.
            {
                let mut err = stderr.lock();
                let _ = write!(err, "\r{}\r", " ".repeat(80));
                let _ = err.flush();
            }
        });

        Self {
            cancel_tx: Some(tx),
            thread: Some(handle),
        }
    }

    /// Stop the spinner and clear it from the terminal.
    ///
    /// **Blocks** until the spinner thread has exited, which guarantees the
    /// terminal line is fully erased before this call returns. Safe to call
    /// multiple times — subsequent calls are no-ops.
    pub fn finish_and_clear(&mut self) {
        // Dropping the sender causes recv_timeout in the thread to return
        // `Disconnected`, which causes it to break, clear the line, and exit.
        drop(self.cancel_tx.take());
        if let Some(handle) = self.thread.take() {
            // Joining ensures the terminal clear has happened before we return.
            let _ = handle.join();
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
        let mut spinner = Spinner::new("working...", false);
        spinner.finish_and_clear();
    }

    #[test]
    fn test_spinner_no_terminal() {
        // In CI / test environments stderr is not a terminal, so the
        // internal thread should not be spawned regardless of `enabled`.
        let mut spinner = Spinner::new("working...", true);
        spinner.finish_and_clear();
    }

    #[test]
    fn test_spinner_finish_twice() {
        // finish_and_clear should be safe to call multiple times.
        let mut spinner = Spinner::new("working...", false);
        spinner.finish_and_clear();
        spinner.finish_and_clear();
    }
}
