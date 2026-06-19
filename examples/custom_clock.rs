//! Plugging in a custom [`Clock`] so the *policy* measures elapsed time against
//! a host-controlled source instead of the wall clock. Here the "sleep" simply
//! advances the same virtual clock, so the whole retry session is deterministic
//! and runs instantly — exactly the pattern you'd use in a `no_std` engine, a
//! simulation, or a consensus test.
//!
//! Run with:
//!
//! ```text
//! cargo run --example custom_clock
//! ```

use std::cell::Cell;
use std::ops::ControlFlow;
use std::time::Duration;

use tardigrade::{retry_sync, Clock, ExponentialBackoff, PolicyExt, RetryError};

/// A fully deterministic virtual clock measured in milliseconds.
struct VirtualClock {
    now_ms: Cell<u64>,
}

impl VirtualClock {
    fn new() -> Self {
        Self {
            now_ms: Cell::new(0),
        }
    }
    fn advance(&self, by: Duration) {
        self.now_ms.set(self.now_ms.get() + by.as_millis() as u64);
    }
}

impl Clock for VirtualClock {
    type Instant = u64;

    fn now(&self) -> u64 {
        self.now_ms.get()
    }

    fn duration_since(&self, earlier: u64, now: u64) -> Duration {
        Duration::from_millis(now.saturating_sub(earlier))
    }
}

fn main() {
    let clock = VirtualClock::new();

    // Give up once 2 seconds of *virtual* time have elapsed.
    let policy = ExponentialBackoff::new(Duration::from_millis(100), 2.0)
        .max_elapsed_time(&clock, Duration::from_secs(2));

    let mut attempt = 0u32;
    let outcome: Result<(), RetryError<u32>> = retry_sync(
        policy,
        || {
            attempt += 1;
            // This operation never succeeds, so we lean on the time budget.
            ControlFlow::Continue(attempt)
        },
        // "Sleeping" just moves virtual time forward — no real waiting.
        |delay| {
            clock.advance(delay);
            println!("slept {delay:?}, virtual t = {}ms", clock.now());
        },
    );

    match outcome {
        Ok(()) => unreachable!("operation never breaks"),
        Err(RetryError::Exhausted(last)) => {
            println!(
                "budget exhausted after {last} attempts at virtual t = {}ms",
                clock.now()
            );
        }
    }
}
