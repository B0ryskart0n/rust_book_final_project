use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: Sender<Box<dyn FnOnce() + Send + 'static>>,
}
impl ThreadPool {
    pub fn new(n: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let shared_receiver = Arc::new(Mutex::new(receiver));

        // Could potentially create 0 workers, but since it's only called from my binary.
        let workers = (0..n)
            .map(|i| Worker::new(i, Arc::clone(&shared_receiver)))
            .collect();

        ThreadPool {
            _workers: workers,
            sender,
        }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender
            .send(Box::new(f))
            .expect("channel in an ill state, can't send closure");
    }
}

struct Worker {
    _id: usize,
    _thread: thread::JoinHandle<()>,
}
impl Worker {
    fn new(_id: usize, receiver: Arc<Mutex<Receiver<Box<dyn FnOnce() + Send>>>>) -> Worker {
        // Keeping the OS thread alive undefinitely with `loop`.
        let _thread = thread::spawn(move || {
            loop {
                let job = receiver
                    .lock()
                    .expect("mutex in an ill state, can't lock")
                    .recv()
                    .expect("channel in an ill state, can't query receiver");

                job()
            }
        });
        Worker { _id, _thread }
    }
}
