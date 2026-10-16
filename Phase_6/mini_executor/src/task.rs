use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::task::Wake;

/// Boxed top-level future type that can be scheduled across threads.
pub type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// A runnable unit of work scheduled onto the executor.
pub struct Task {
    /// Mutex-wrapped future. When running, the future is polled.
    /// If it returns `Poll::Pending`, it is put back into this slot.
    pub future: Mutex<Option<BoxFuture>>,

    /// Channel handle used by `wake()` to push this task back to the executor queue.
    pub task_sender: SyncSender<Arc<Task>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        // Re-enqueue this task to notify the executor that it is ready to be polled again.
        let _ = self.task_sender.send(Arc::clone(self));
    }
}
