//! The local transport.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json;

use std::marker::PhantomData;

use crate::LocationSet;

use crate::core::{ChoreographyLocation, HCons, HList, Portable, Transport};
use crate::utils::queue::BlockingQueue;

type QueueMap = HashMap<String, HashMap<String, BlockingQueue<String>>>;

/// A Transport channel used between multiple `Transport`s.
pub struct LocalTransportChannel<L: HList> {
    /// The location set where the channel is defined on.
    location_set: std::marker::PhantomData<L>,
    queue_map: Arc<QueueMap>,
}

impl<L: HList> Clone for LocalTransportChannel<L> {
    fn clone(&self) -> Self {
        LocalTransportChannel {
            location_set: PhantomData,
            queue_map: self.queue_map.clone(),
        }
    }
}

impl LocalTransportChannel<LocationSet!()> {
    /// Creates a new `LocalTransportChannel` instance
    pub fn new() -> Self {
        Self {
            location_set: PhantomData,
            queue_map: HashMap::new().into(),
        }
    }
}

impl<L: HList> LocalTransportChannel<L> {
    /// Adds a new location to the set of locations in the `LocalTransportChannel`.
    pub fn with<NewLocation: ChoreographyLocation>(
        self,
        _location: NewLocation,
    ) -> LocalTransportChannel<HCons<NewLocation, L>> {
        let mut queue_map: QueueMap = HashMap::new();
        let mut str_list = L::to_string_list();
        str_list.push(NewLocation::name());
        for sender in &str_list {
            let mut n = HashMap::new();
            for receiver in &str_list {
                n.insert(receiver.to_string(), BlockingQueue::new());
            }
            queue_map.insert(sender.to_string(), n);
        }

        LocalTransportChannel {
            location_set: PhantomData,
            queue_map: queue_map.into(),
        }
    }
}

/// The local transport.
///
/// This transport uses a blocking queue to allow for communication between threads. Each location must be executed in its thread.
///
/// Unlike network-based transports, all locations must share the same `LocalTransport` instance. The struct implements `Clone` so that it can be shared across threads.
pub struct LocalTransport<L: HList, TargetLocation> {
    internal_locations: Vec<String>,
    location_set: PhantomData<L>,
    local_channel: LocalTransportChannel<L>,
    target_location: PhantomData<TargetLocation>,
}

impl<L: HList, TargetLocation> LocalTransport<L, TargetLocation> {
    /// Creates a new `LocalTransport` instance from a Target `ChoreographyLocation` and a `LocalTransportChannel`.
    pub fn new(_target: TargetLocation, local_channel: LocalTransportChannel<L>) -> Self {
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

        let transport_channel = LocalTransportChannel::new().with(Alice).with(Bob);

        let mut handles = Vec::new();
        {
            let transport = LocalTransport::new(Alice, transport_channel.clone());
            handles.push(thread::spawn(move || {
                transport.send::<i32>(Alice::name(), Bob::name(), &v);
            }));
        }
        {
            let transport = LocalTransport::new(Bob, transport_channel.clone());
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
