//! Deterministic jitter with a seeded PRNG. Two policies built from the *same*
//! seed produce the *same* delay sequence — reproducible chaos, ideal for P2P
//! simulations and consensus tests — while still spreading peers out to avoid a
//! thundering herd.
//!
//! Run with:
//!
//! ```text
//! cargo run --example jitter
//! ```

use std::time::Duration;

use tardigrade::{BackoffPolicy, ExponentialBackoff, SplitMix64};

fn delays_for_seed(seed: u64) -> Vec<Duration> {
    // ±50% symmetric jitter around an exponentially growing interval.
    let mut policy = ExponentialBackoff::new(Duration::from_millis(100), 2.0)
        .with_jitter(SplitMix64::new(seed), 0.5);
    (0..6).map(|_| policy.next_delay().unwrap()).collect()
}

fn main() {
    let seed = 0xC0FF_EE00;

    let run_a = delays_for_seed(seed);
    let run_b = delays_for_seed(seed);
    let run_c = delays_for_seed(seed + 1);

    println!("seed {seed:#x}, run A: {run_a:?}");
    println!("seed {seed:#x}, run B: {run_b:?}");
    println!("seed {:#x}, run C: {run_c:?}", seed + 1);

    assert_eq!(run_a, run_b, "same seed must reproduce the same jitter");
    assert_ne!(run_a, run_c, "different seed should differ");

    println!("\nNominal (no-jitter) intervals would have been: 100, 200, 400, 800, 1600, 3200 ms");
    println!("Reproducibility verified: same seed -> identical sequence.");
}
