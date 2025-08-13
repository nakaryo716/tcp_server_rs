use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Task>>,
}

impl ThreadPool {
    pub fn new(num: u32) -> Self {
        if num == 0 {
            panic!("number of worker thread should more than 0");
        }

        let (sender, receiver) = mpsc::channel::<Task>();
        let arc_receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(num as usize);

        for id in 0..num {
            let arc_receiver = Arc::clone(&arc_receiver);
            workers.push(Worker::generate(id, arc_receiver));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self.sender.as_ref() {
            Some(sender) => sender.send(Task::new(f)).unwrap(),
            _ => unreachable!(),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            worker.handle.join().unwrap();
        }
    }
}

struct Worker {
    #[allow(unused)]
    id: u32,
    handle: JoinHandle<()>,
}

impl Worker {
    fn generate(id: u32, receiver: Arc<Mutex<Receiver<Task>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                let task_result;
                {
                    let gurad = receiver.lock().unwrap();
                    task_result = gurad.recv();
                }

                match task_result {
                    Ok(task) => task.0(),
                    Err(e) => {
                        eprintln!("{e:?}");
                        break;
                    }
                }
            }
        });
        Worker { id, handle }
    }
}

struct Task(Box<dyn FnOnce() + Send + 'static>);

impl Task {
    fn new<F>(f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Task(Box::new(f))
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::{Arc, Mutex}, time};

    use crate::thread_pool::ThreadPool;

    #[test]
    fn test_blocking_task() {
        let pool = ThreadPool::new(4);

        let data = Arc::new(Mutex::new(0));
        for _ in 0..4 {
            let data = data.clone();
            pool.spawn(move || {
                task(data);
            });
        }

        std::thread::sleep(time::Duration::from_millis(6));
        assert_eq!(*data.lock().unwrap(), 4);
    }

    fn task(data: Arc<Mutex<u32>>) {
        std::thread::sleep(time::Duration::from_millis(4));
        let mut gurad = data.lock().unwrap();
        *gurad += 1;
    }
}

