use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct BlockingQueue<T> {
    data: Mutex<VecDeque<T>>,
    not_empty: Condvar,
}

impl<T> BlockingQueue<T> {
    pub fn new() -> Self {
        BlockingQueue {
            data: Mutex::new(VecDeque::new()),
            not_empty: Condvar::new(),
        }
    }

    pub fn push(&self, item: T) {
        let mut queue = self.data.lock().unwrap();
        queue.push_back(item);
        self.not_empty.notify_one();
    }

    pub fn pop(&self) -> T {
        let mut queue = self.data.lock().unwrap();
        while queue.is_empty() {
            queue = self.not_empty.wait(queue).unwrap();
        }
        let item = queue.pop_front().unwrap();
        item
    }
}
