use colored::Colorize;
use std::io::{self, Write};
use tokio::time::Instant;

/// Simple progress spinner for showing waiting status
pub struct ProgressSpinner {
    chars: Vec<char>,
    current: usize,
    start_time: Instant,
}

impl ProgressSpinner {
    pub fn new() -> Self {
        Self {
            chars: vec!['|', '/', '-', '\\'],
            current: 0,
            start_time: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs();
        print!(
            "\r🔗 Waiting for confirmation... {} ({}s)",
            self.chars[self.current].to_string().bright_blue(),
            elapsed
        );
        io::stdout().flush().unwrap();
        self.current = (self.current + 1) % self.chars.len();
    }
}
