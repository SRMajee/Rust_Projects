use std::future::Future;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use crate::task::Task;

/// Spawner sends new tasks into the executor's ready queue.
#[derive(Clone)]
pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    /// Spawns a future onto the executor.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender
            .send(task)
            .expect("failed to enqueue task: executor queue closed");
    }
}

/// A minimal single-threaded cooperative executor.
pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
    /// Creates a new Executor and its associated Spawner.
    pub fn new(capacity: usize) -> (Self, Spawner) {
        let (task_sender, ready_queue) = sync_channel(capacity);
        (Self { ready_queue }, Spawner { task_sender })
    }

    /// Runs the executor until all spawners are dropped and the queue is exhausted.
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            // Take the future out of the task mutex so we can poll it.
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Build a Waker using the standard library's Wake trait implementation on Task.
                let waker = Waker::from(Arc::clone(&task));
                let mut cx = Context::from_waker(&waker);

                // Poll the future. If it yields Pending, put it back for the next wakeup.
                if let Poll::Pending = future.as_mut().poll(&mut cx) {
                    *future_slot = Some(future);
                }
            }
        }
    }
}
