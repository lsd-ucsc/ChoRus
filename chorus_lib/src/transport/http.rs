//! The HTTP transport.

use std::thread;
use std::{collections::HashMap, sync::Arc};

use std::marker::PhantomData;

use retry::{
    delay::{jitter, Fixed},
    retry,
};
use tiny_http::Server;
use ureq::{Agent, AgentBuilder};

use crate::{
    core::{ChoreographyLocation, LocationSet, Member, Portable, Transport},
    transport::{TransportConfig, TransportConfigBuilder},
    utils::queue::BlockingQueue,
};

type QueueMap = HashMap<String, BlockingQueue<String>>;

/// Config for `HttpTransport`.
pub type HttpTransportConfig<L, Target> = TransportConfig<Target, (String, u16), L, (String, u16)>;

/// A builder for `HttpTransportConfig`.
///
/// # Examples
///
/// ```
/// # use chorus_lib::core::{LocationSet, ChoreographyLocation};
/// # use chorus_lib::transport::http::HttpTransportConfigBuilder;
/// #
/// # #[derive(ChoreographyLocation)]
/// # struct Alice;
/// #
/// # #[derive(ChoreographyLocation)]
/// # struct Bob;
/// #
/// let transport_config = HttpTransportConfigBuilder::for_target(Alice, ("0.0.0.0".to_string(), 9010))
///   .with(Bob, ("example.com".to_string(), 80))
///   .build();
/// ```
pub type HttpTransportConfigBuilder<Target, L> =
    TransportConfigBuilder<Target, (String, u16), L, (String, u16)>;

/// The header name for the source location.
const HEADER_SRC: &str = "X-CHORUS-SOURCE";

/// The HTTP transport.
pub struct HttpTransport<L: LocationSet, TLocation> {
    config: HashMap<String, (String, u16)>,
    agent: Agent,
    server: Arc<Server>,
    join_handle: Option<thread::JoinHandle<()>>,
    location_set: PhantomData<L>,
    queue_map: Arc<QueueMap>,
    target_location: PhantomData<TLocation>,
}

impl<L: LocationSet, TLocation: ChoreographyLocation> HttpTransport<L, TLocation> {
    /// Creates a new `HttpTransport` instance from the configuration.
    pub fn new<Index>(http_config: HttpTransportConfig<L, TLocation>) -> Self
    where
        TLocation: Member<L, Index>,
    {
        let queue_map: Arc<QueueMap> = {
            let mut m = HashMap::new();
            for loc in L::to_string_list() {
                m.insert(loc.to_string(), BlockingQueue::new());
            }
            Arc::new(m.into())
        };

        let (_, (hostname, port)) = &http_config.target_info;
        let server = Arc::new(Server::http(format!("{}:{}", hostname, port)).unwrap());
        let join_handle = Some({
            let server = server.clone();
            let queue_map = queue_map.clone();

            thread::spawn(move || {
                for mut request in server.incoming_requests() {
                    let mut body = String::new();
                    request
                        .as_reader()
                        .read_to_string(&mut body)
                        .expect("Failed to read body");
                    let mut headers = request.headers().iter();
                    let src = headers.find(|header| header.field.equiv(HEADER_SRC));
                    if let Some(src) = src {
                        let src = &src.value;
                        queue_map.get(src.as_str()).unwrap().push(body);
                        request
                            .respond(tiny_http::Response::from_string("OK").with_status_code(200))
                            .unwrap();
                    } else {
                        request
                            .respond(
                                tiny_http::Response::from_string("Bad Request")
                                    .with_status_code(400),
                            )
                            .unwrap();
                    }
                }
            })
        });

        let agent = AgentBuilder::new().build();

        Self {
            config: http_config.info,
            agent,
            join_handle,
            server,
            location_set: PhantomData,
            queue_map,
            target_location: PhantomData,
        }
    }
}

impl<L: LocationSet, TLocation> Drop for HttpTransport<L, TLocation> {
    fn drop(&mut self) {
        self.server.unblock();
        self.join_handle.take().map(thread::JoinHandle::join);
    }
}

impl<L: LocationSet, TLocation: ChoreographyLocation> Transport<L, TLocation>
    for HttpTransport<L, TLocation>
{
    fn locations(&self) -> Vec<String> {
        Vec::from_iter(self.config.keys().map(|s| s.clone()))
    }

    fn send<V: Portable>(&self, from: &str, to: &str, data: &V) -> () {
        let (hostname, port) = self.config.get(to).unwrap();
        retry(Fixed::from_millis(1000).map(jitter), move || {
            self.agent
                .post(format!("http://{}:{}", hostname, port).as_str())
                .set(HEADER_SRC, from)
                .send_string(serde_json::to_string(data).unwrap().as_str())
        })
        .unwrap();
    }

    fn receive<V: Portable>(&self, from: &str, _at: &str) -> V {
        let str = self.queue_map.get(from).unwrap().pop();
        serde_json::from_str(&str).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread::{self, sleep};
    use std::time::Duration;

    use super::*;
    use crate::core::ChoreographyLocation;

    #[derive(ChoreographyLocation)]
    struct Alice;

    #[derive(ChoreographyLocation)]
    struct Bob;

    #[derive(ChoreographyLocation)]
    struct Carol;

    #[test]
    fn test_http_transport() {
        let v = 42;

        let (signal, wait) = mpsc::channel::<()>();

        let mut handles = Vec::new();
        {
            let config =
                HttpTransportConfigBuilder::for_target(Alice, ("0.0.0.0".to_string(), 9010))
                    .with(Bob, ("localhost".to_string(), 9011))
                    .build();

            handles.push(thread::spawn(move || {
                wait.recv().unwrap(); // wait for Bob to start
                let transport = HttpTransport::new(config);
                transport.send::<i32>(Alice::name(), Bob::name(), &v);
            }));
        }
        {
            let config = HttpTransportConfigBuilder::for_target(Bob, ("0.0.0.0".to_string(), 9011))
                .with(Alice, ("localhost".to_string(), 9010))
                .build();

            handles.push(thread::spawn(move || {
                let transport = HttpTransport::new(config);
                signal.send(()).unwrap();
                let v2 = transport.receive::<i32>(Alice::name(), Bob::name());
                assert_eq!(v, v2);
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_http_transport_retry() {
        let v = 42;
        let (signal, wait) = mpsc::channel::<()>();

        let mut handles = Vec::new();
        {
            let config =
                HttpTransportConfigBuilder::for_target(Alice, ("0.0.0.0".to_string(), 9020))
                    .with(Bob, ("localhost".to_string(), 9021))
                    .build();

            handles.push(thread::spawn(move || {
                signal.send(()).unwrap();
                let transport = HttpTransport::new(config);
                transport.send::<i32>(Alice::name(), Bob::name(), &v);
            }));
        }
        {
            let config = HttpTransportConfigBuilder::for_target(Bob, ("0.0.0.0".to_string(), 9021))
                .with(Alice, ("localhost".to_string(), 9020))
                .build();

            handles.push(thread::spawn(move || {
                // wait for Alice to start, which forces Alice to retry
                wait.recv().unwrap();
                sleep(Duration::from_millis(100));
                let transport = HttpTransport::new(config);
                let v2 = transport.receive::<i32>(Alice::name(), Bob::name());
                assert_eq!(v, v2);
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
