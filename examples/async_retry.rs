//! `retry_async` is runtime-agnostic: it only speaks `core::future::Future`.
//! This example proves it by driving the retry loop with a ~20-line hand-rolled
//! executor and a sleep future that's ready immediately — no Tokio, async-std,
//! or any runtime dependency. In real code you'd pass `tokio::time::sleep`,
//! `embassy_time::Timer`, or a browser/WASM timer instead.
//!
//! Run with:
//!
//! ```text
//! cargo run --example async_retry
//! ```

use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::Duration;

use sisyphus::{retry_async, ExponentialBackoff, PolicyExt, RetryError};

/// A stand-in for a real async timer; ready on first poll.
async fn fake_sleep(delay: Duration) {
    println!("  (would await timer for {delay:?})");
}

async fn run() -> Result<u32, RetryError<&'static str>> {
    let policy = ExponentialBackoff::new(Duration::from_millis(10), 2.0).max_attempts(5);
    let mut attempt = 0u32;

    retry_async(
        policy,
        || {
            attempt += 1;
            async move {
                println!("async attempt {attempt}");
                if attempt < 4 {
                    ControlFlow::Continue("service unavailable")
                } else {
                    ControlFlow::Break(attempt)
                }
            }
        },
        // Swap this closure for your runtime's real sleep in production.
        fake_sleep,
    )
    .await
}

fn main() {
    match block_on(run()) {
        Ok(attempts) => println!("succeeded on attempt {attempts}"),
        Err(RetryError::Exhausted(last)) => println!("exhausted; last state: {last}"),
    }
}

/// A minimal `block_on` so this example needs zero async dependencies.
fn block_on<F: Future>(mut fut: F) -> F::Output {
    fn noop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);

    // Safety: the waker is a no-op and never dereferences its data pointer.
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    // Safety: `fut` lives on the stack for the duration of this function and is
    // never moved after being pinned.
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
    loop {
        if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
            return v;
        }
    }
}
