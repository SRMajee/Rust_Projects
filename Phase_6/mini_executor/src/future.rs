use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

/// Shared internal state between TimerFuture and the background thread.
struct TimerSharedState {
    completed: bool,
    waker: Option<Waker>,
}

/// A hand-rolled Future that completes after a specified duration.
/// Demonstrates the reactor pattern: background event source signaling a Waker.
pub struct TimerFuture {
    shared_state: Arc<Mutex<TimerSharedState>>,
}

impl TimerFuture {
    /// Creates a new `TimerFuture` that completes after `duration`.
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(TimerSharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = Arc::clone(&shared_state);
        thread::spawn(move || {
            thread::sleep(duration);
            let mut state = thread_shared_state.lock().unwrap();
            state.completed = true;
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        });

        Self { shared_state }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.shared_state.lock().unwrap();
        if state.completed {
            Poll::Ready(())
        } else {
            // Update waker with latest context waker so that the thread notifies the right context.
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// A hand-rolled Future that yields execution once cooperatively, re-scheduling itself.
pub struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            // Notify executor that we are ready to be polled again immediately,
            // but yield the current turn to let other tasks run.
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Helper function to yield control back to the executor once.
pub fn yield_now() -> YieldNow {
    YieldNow { yielded: false }
}
