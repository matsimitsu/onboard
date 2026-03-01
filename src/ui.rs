use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicUsize, Ordering};

static LINE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Print an indented output line and track it for collapsing.
pub fn output(msg: &str) {
    println!("  {}", msg);
    LINE_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Reset the line counter and return the previous count.
pub fn take_line_count() -> usize {
    LINE_COUNT.swap(0, Ordering::Relaxed)
}

/// Collapse the step header + output lines and replace with a compact done line.
/// Only collapses in interactive terminals; in pipes, just prints "done".
pub fn complete_step(header: &str) {
    let lines = take_line_count();

    if std::io::stdout().is_terminal() {
        // Move cursor up past output + header, clear to end of screen
        let total = lines + 1;
        print!("\x1b[{}A\x1b[J", total);
        std::io::stdout().flush().ok();
        println!("{} - done", header);
    } else {
        println!("  done.");
    }
}
