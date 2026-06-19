//! Using the built-in [`SystemClock`] (requires the `std` feature) to enforce a
//! real wall-clock time budget on retries.
//!
//! Run with:
//!
//! ```text
//! cargo run --example system_clock --features std
//! ```

use std::ops::ControlFlow;
use std::time::Duration;

use sisyphus::{retry_sync, Constant, PolicyExt, RetryError, SystemClock};

fn main() {
    // Poll every 20ms, but never spend more than 200ms of real time trying.
    let policy = Constant::new(Duration::from_millis(20))
        .max_elapsed_time(SystemClock, Duration::from_millis(200));

    let mut attempt = 0u32;
    let result: Result<(), RetryError<u32>> = retry_sync(
        policy,
        || {
            attempt += 1;
            ControlFlow::Continue(attempt) // never succeeds; rely on the budget
        },
        std::thread::sleep,
    );

    match result {
        Ok(()) => unreachable!(),
        Err(RetryError::Exhausted(last)) => {
            println!("real-time budget exhausted after {last} attempts");
        }
    }
}
