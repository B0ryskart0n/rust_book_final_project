use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Box<dyn FnOnce() + Send + 'static>>>,
}
impl ThreadPool {
    pub fn new(n: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let shared_receiver = Arc::new(Mutex::new(receiver));
        let sender = Some(sender);

        // Could potentially create 0 workers, but since it's only called from my binary.
        let workers = (0..n)
            .map(|i| Worker::new(i, Arc::clone(&shared_receiver)))
            .collect();

        ThreadPool { workers, sender }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender
            .as_ref()
            .unwrap()
            .send(Box::new(f))
            .expect("channel in an ill state, can't send closure");
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        eprintln!("Dropping ThreadPool.");
        // Move out the underlying `sender` and drop it. This will put the channel in an ill state.
        drop(self.sender.take());

        // `Vec::drain` moves the workers out of the vector, which is needed for `JoinHandle::join()`.
        for worker in self.workers.drain(..) {
            eprintln!("Waiting for Worker {} to finish.", worker._id);
            worker.thread.join().unwrap();
        }
    }
}

struct Worker {
    _id: usize,
    thread: thread::JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Box<dyn FnOnce() + Send>>>>) -> Worker {
        // Keeping the OS thread alive undefinitely with `loop`.
        let thread = thread::spawn(move || {
            loop {
                eprintln!("    Worker {id} trying to acquire lock for job receiver.");
                let receiver_guard = receiver.lock().expect("mutex in an ill state, can't lock");
                eprintln!("    Worker {id} acquired lock for job receiver.");

                let job = match receiver_guard.recv() {
                    Err(_) => {
                        eprintln!(
                            "    Worker {id} shutting down because the sender has been disconnected."
                        );
                        break;
                    }
                    Ok(job) => job,
                };

                // Free the lock before proceeding with the job. Otherwise there will be no concurrency because no other worker will be able to read the job receiver during this worker's work.
                // If, however, the `receiver.lock().unwrap()` was part of a let statement, then it would drop automatically at the end of the let statement.
                drop(receiver_guard);

                eprintln!("    Worker {id} received job.");
                job();
                eprintln!("    Worker {id} finished job.");
            }
        });
        Worker { _id: id, thread }
    }
}
