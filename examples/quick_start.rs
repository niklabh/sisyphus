//! The canonical sync use case: retry a flaky operation with exponential
//! backoff, bounded by a max delay and a max number of attempts, sleeping the
//! real thread between tries.
//!
//! Run with:
//!
//! ```text
//! cargo run --example quick_start
//! ```

use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use sisyphus::{retry_sync, ExponentialBackoff, PolicyExt, RetryError};

/// A fake service that fails a few times before succeeding.
fn flaky_connect(attempt: u32) -> Result<&'static str, &'static str> {
    if attempt < 4 {
        Err("connection refused")
    } else {
        Ok("connected")
    }
}

fn main() {
    let policy = ExponentialBackoff::new(Duration::from_millis(50), 2.0)
        .with_max_delay(Duration::from_secs(1))
        .max_attempts(6);

    let mut attempt = 0u32;
    let started = Instant::now();

    let result: Result<&str, RetryError<&str>> = retry_sync(
        policy,
        || {
            attempt += 1;
            match flaky_connect(attempt) {
                Ok(ok) => ControlFlow::Break(ok),
                Err(transient) => {
                    println!("attempt {attempt} failed: {transient}");
                    ControlFlow::Continue(transient)
                }
            }
        },
        // The execution side is entirely yours; here we block the OS thread.
        |delay| {
            println!("  backing off for {delay:?}");
            std::thread::sleep(delay);
        },
    );

    match result {
        Ok(msg) => println!(
            "success after {attempt} attempts in {:?}: {msg}",
            started.elapsed()
        ),
        Err(RetryError::Exhausted(last)) => {
            println!("gave up after {attempt} attempts; last error: {last}");
        }
    }
}
