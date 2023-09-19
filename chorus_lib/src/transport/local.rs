//! The local transport.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json;

use std::marker::PhantomData;

#[cfg(test)]
use crate::LocationSet;

use crate::core::{ChoreographyLocation, HList, Portable, Transport};
use crate::utils::queue::BlockingQueue;

type QueueMap = HashMap<String, HashMap<String, BlockingQueue<String>>>;

/// A Transport channel used between multiple `Transport`s.
#[derive(Clone)]
pub struct LocalTransportChannel<L: crate::core::HList> {
    /// The location set where the channel is defined on.
    pub location_set: std::marker::PhantomData<L>,
    queue_map: Arc<QueueMap>,
}

impl<L: HList> LocalTransportChannel<L> {
    /// Creates a `LocalTransportChannel`.
    pub fn new() -> LocalTransportChannel<L> {
        let mut queue_map: QueueMap = HashMap::new();
        for sender in L::to_string_list() {
            let mut n = HashMap::new();
            for receiver in L::to_string_list() {
                n.insert(receiver.to_string(), BlockingQueue::new());
            }
            queue_map.insert(sender.to_string(), n);
        }

        LocalTransportChannel {
            location_set: PhantomData,
            queue_map: Arc::new(queue_map.into()),
        }
    }
}

/// The local transport.
///
/// This transport uses a blocking queue to allow for communication between threads. Each location must be executed in its thread.
///
/// Unlike network-based transports, all locations must share the same `LocalTransport` instance. The struct implements `Clone` so that it can be shared across threads.
#[derive(Clone)]
pub struct LocalTransport<L: HList, TargetLocation> {
    internal_locations: Vec<String>,
    location_set: PhantomData<L>,
    local_channel: Arc<LocalTransportChannel<L>>,
    target_location: PhantomData<TargetLocation>,
}

impl<L: HList, TargetLocation> LocalTransport<L, TargetLocation> {
    /// Creates a new `LocalTransport` instance from a Target `ChoreographyLocation` and a `LocalTransportChannel`.
    pub fn new(_target: TargetLocation, local_channel: Arc<LocalTransportChannel<L>>) -> Self {
        let locations_list = L::to_string_list();

        let mut locations_vec = Vec::new();
        for loc in locations_list.clone() {
            locations_vec.push(loc.to_string());
        }

        LocalTransport {
            internal_locations: locations_vec,
            location_set: PhantomData,
            local_channel,
            target_location: PhantomData,
        }
    }
}

impl<L: HList, TargetLocation: ChoreographyLocation> Transport<L, TargetLocation>
    for LocalTransport<L, TargetLocation>
{
    fn locations(&self) -> Vec<String> {
        return self.internal_locations.clone();
    }

    fn send<T: Portable>(&self, from: &str, to: &str, data: &T) -> () {
        let data = serde_json::to_string(data).unwrap();
        self.local_channel
            .queue_map
            .get(from)
            .unwrap()
            .get(to)
            .unwrap()
            .push(data)
    }

    fn receive<T: Portable>(&self, from: &str, at: &str) -> T {
        let data = self
            .local_channel
            .queue_map
            .get(from)
            .unwrap()
            .get(at)
            .unwrap()
            .pop();
        serde_json::from_str(&data).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ChoreographyLocation;
    use std::thread;

    #[derive(ChoreographyLocation)]
    struct Alice;

    #[derive(ChoreographyLocation)]
    struct Bob;

    #[test]
    fn test_local_transport() {
        let v = 42;

        let transport_channel = Arc::new(LocalTransportChannel::<LocationSet!(Bob, Alice)>::new());

        let mut handles = Vec::new();
        {
            let transport = LocalTransport::new(Alice, Arc::clone(&transport_channel));
            handles.push(thread::spawn(move || {
                transport.send::<i32>(Alice::name(), Bob::name(), &v);
            }));
        }
        {
            let transport = LocalTransport::new(Bob, Arc::clone(&transport_channel));
            handles.push(thread::spawn(move || {
                let v2 = transport.receive::<i32>(Alice::name(), Bob::name());
                assert_eq!(v, v2);
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
