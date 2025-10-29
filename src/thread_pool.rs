/**
This was gotten from the rust-lang book (https://doc.rust-lang.org/book/ch21-03-graceful-shutdown-and-cleanup.html)
and the modifications came from this youtube video (https://youtu.be/qjx8vutWaUQ?si=VTBgvFDAnTaaa8ZQ)
*/
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct WorkerPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl WorkerPool {
    /// Create new ThreadPool
    ///
    /// The size is the number of threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is 0
    pub fn new(size: usize) -> WorkerPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel(); // communicate between threads

        // creating shared ownership of the receiver
        // `Arc` for thread safe ownership
        // `Mutex` for thread safe mutability
        let receiver: Arc<Mutex<mpsc::Receiver<Message>>> = Arc::new(Mutex::new(receiver));

        let mut workers: Vec<Worker> = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        WorkerPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .send(Message::NewJob(job))
            .expect("should send job")
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender
                .send(Message::Terminate)
                .expect("sender should send terminate message")
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("error from thread");
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread: thread::JoinHandle<()> = thread::spawn(move || loop {
            let message: Message = receiver
                .lock()
                .expect("should get lock on receiver")
                .recv()
                .expect("should get value from receiver");

            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
                    break;
                }
            }
        });

        Worker {
            thread: Some(thread),
        }
    }
}
