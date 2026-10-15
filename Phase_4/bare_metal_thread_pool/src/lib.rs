use std::fmt;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// Type alias for an owned, unit-returning closure capable of transfer across threads.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Message sent through the channel to worker threads.
enum Message {
    NewJob(Job),
    Terminate,
}

/// Error type returned when ThreadPool creation fails.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PoolCreationError {
    ZeroThreads,
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroThreads => write!(f, "Thread pool must be initialized with at least 1 thread"),
        }
    }
}

impl std::error::Error for PoolCreationError {}

/// A bare-metal thread pool that dispatches jobs over an MPSC channel to worker threads.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}

impl ThreadPool {
    /// Creates a new `ThreadPool` with the specified number of threads.
    ///
    /// # Panics
    /// Panics if `size` is 0. Use [`ThreadPool::build`] for non-panicking initialization.
    pub fn new(size: usize) -> Self {
        Self::build(size).expect("ThreadPool size must be greater than zero")
    }

    /// Attempts to build a new `ThreadPool` with the specified number of threads.
    /// Returns `Err(PoolCreationError::ZeroThreads)` if `size` is 0.
    pub fn build(size: usize) -> Result<Self, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::ZeroThreads);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    /// Returns the number of worker threads in the pool.
    pub fn size(&self) -> usize {
        self.workers.len()
    }

    /// Dispatches a job `f` to an available worker thread in the pool.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Some(ref sender) = self.sender {
            let _ = sender.send(Message::NewJob(job));
        }
    }

    /// Shuts down the thread pool, sending terminate signals to all workers
    /// and waiting for their threads to exit.
    pub fn shutdown(&mut self) {
        if let Some(sender) = self.sender.take() {
            // Send a Terminate message for every worker.
            for _ in &self.workers {
                let _ = sender.send(Message::Terminate);
            }
            // Dropping sender closes the channel as well.
            drop(sender);
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// An individual worker holding a dedicated OS thread.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::Builder::new()
            .name(format!("worker-{}", id))
            .spawn(move || loop {
                let message = {
                    // Acquire lock and receive next message.
                    // Lock is dropped immediately upon exiting this block.
                    let rx = match receiver.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    rx.recv()
                };

                match message {
                    Ok(Message::NewJob(job)) => {
                        job();
                    }
                    Ok(Message::Terminate) | Err(_) => {
                        // Channel disconnected or terminate signal received: stop worker.
                        break;
                    }
                }
            })
            .expect("Failed to spawn worker thread");

        Worker {
            id,
            thread: Some(thread),
        }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> usize {
        self.id
    }
}
