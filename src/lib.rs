use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

enum WorkOrder {
    Job(Box<dyn FnOnce() + Send + 'static>),
    FinishWork,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<WorkOrder>,
}
impl ThreadPool {
    pub fn new(n: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let shared_receiver = Arc::new(Mutex::new(receiver));

        // Could potentially create 0 workers, but since it's only called from my binary.
        let workers = (0..n)
            .map(|i| Worker::new(i, Arc::clone(&shared_receiver)))
            .collect();

        ThreadPool { workers, sender }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender
            .send(WorkOrder::Job(Box::new(f)))
            .expect("channel in an ill state, can't send job");
    }
}
impl Drop for ThreadPool {
    // TODO There shouldn't be a possibility of a panic in drop, since this can cause a double panic, which immediately crashes tne program without cleanup process.
    fn drop(&mut self) {
        eprintln!("Dropping ThreadPool.");

        eprintln!("Sending FinishWork order for each worker.");
        for _ in &self.workers {
            self.sender
                .send(WorkOrder::FinishWork)
                .expect("channel in an ill state, can't send finish order");
        }

        // `Vec::drain` moves the workers out of the vector, which is needed for `JoinHandle::join()`.
        for worker in self.workers.drain(..) {
            eprintln!("Waiting for Worker {} to finish.", worker._id);
            worker.thread.join().unwrap();
        }

        eprintln!("Dropped ThreadPool.");
    }
}

struct Worker {
    _id: usize,
    thread: thread::JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<WorkOrder>>>) -> Worker {
        // Keeping the OS thread alive undefinitely with `loop`.
        let thread = thread::spawn(move || {
            loop {
                eprintln!("    Worker {id} trying to acquire lock for job receiver.");
                let receiver_guard = receiver.lock().expect("mutex in an ill state, can't lock");
                eprintln!("    Worker {id} acquired lock for job receiver.");

                let job = match receiver_guard
                    .recv()
                    .expect("WorkOrder sender should be available.")
                {
                    WorkOrder::FinishWork => {
                        eprintln!("    Worker {id} received FinishWork order: shutting down.");
                        break;
                    }
                    WorkOrder::Job(job) => {
                        eprintln!("    Worker {id} received job.");
                        job
                    }
                };

                // Free the lock before proceeding with the job. Otherwise there will be no concurrency because no other worker will be able to read the job receiver during this worker's work.
                // If, however, the `receiver.lock().unwrap()` was part of a let statement, then it would drop automatically at the end of the let statement.
                drop(receiver_guard);
                job();
                eprintln!("    Worker {id} finished job.");
            }
        });
        Worker { _id: id, thread }
    }
}
