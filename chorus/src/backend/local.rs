use std::collections::HashMap;
use std::sync::Arc;

use serde_json;

use crate::core::Backend;
use crate::utils::queue::BlockingQueue;

type QueueMap = HashMap<String, HashMap<String, BlockingQueue<String>>>;

#[derive(Clone)]
pub struct LocalBackend {
    internal_locations: Vec<String>,
    queue_map: Arc<QueueMap>,
}

impl LocalBackend {
    pub fn from(locations: &[&str]) -> Self {
        let mut queue_map: QueueMap = HashMap::new();
        for sender in locations.clone() {
            let mut n = HashMap::new();
            for receiver in locations.clone() {
                n.insert(receiver.to_string(), BlockingQueue::new());
            }
            queue_map.insert(sender.to_string(), n);
        }
        let mut locations_vec = Vec::new();
        for loc in locations.clone() {
            locations_vec.push(loc.to_string());
        }
        LocalBackend {
            queue_map: Arc::new(queue_map),
            internal_locations: locations_vec,
        }
    }
}

impl Backend for LocalBackend {
    fn locations(&self) -> Vec<String> {
        return self.internal_locations.clone();
    }

    fn send<T: crate::core::ChoreographicValue>(&self, from: &str, to: &str, data: T) -> () {
        let data = serde_json::to_string(&data).unwrap();
        self.queue_map
            .get(from)
            .unwrap()
            .get(to)
            .unwrap()
            .push(data)
    }

    fn receive<T: crate::core::ChoreographicValue>(&self, from: &str, at: &str) -> T {
        let data = self.queue_map.get(from).unwrap().get(at).unwrap().pop();
        serde_json::from_str(&data).unwrap()
    }
}
