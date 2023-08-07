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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocking_queue_push_pop() {
        let queue = BlockingQueue::<i32>::new();
        queue.push(1);
        queue.push(2);
        queue.push(3);
        assert_eq!(queue.pop(), 1);
        assert_eq!(queue.pop(), 2);
        assert_eq!(queue.pop(), 3);
    }

    #[test]
    fn test_blocking_queue_pop_push() {
        let queue = std::sync::Arc::new(BlockingQueue::<i32>::new());
        let handle = {
            let queue = queue.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(100));
                queue.push(1);
                queue.push(2);
                queue.push(3);
            })
        };
        assert_eq!(queue.pop(), 1);
        assert_eq!(queue.pop(), 2);
        assert_eq!(queue.pop(), 3);
        handle.join().unwrap();
    }
}
