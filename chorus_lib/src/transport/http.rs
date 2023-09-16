//! The HTTP transport.

use std::thread;
use std::{collections::HashMap, sync::Arc};

use retry::{
    delay::{jitter, Fixed},
    retry,
};
use tiny_http::Server;
use ureq::{Agent, AgentBuilder};

use crate::{
    core::{Portable, Transport},
    utils::queue::BlockingQueue,
};

/// The header name for the source location.
const HEADER_SRC: &str = "X-CHORUS-SOURCE";

/// The HTTP transport.
pub struct HttpTransport {
    config: HashMap<String, (String, u16)>,
    agent: Agent,
    queue_map: Arc<HashMap<String, BlockingQueue<String>>>,
    server: Arc<Server>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl HttpTransport {
    /// Creates a new `HttpTransport` instance from the projection target and a configuration.
    pub fn new(at: &'static str, config: &HashMap<&str, (&str, u16)>) -> Self {
        let config = HashMap::from_iter(
            config
                .iter()
                .map(|(k, (hostname, port))| (k.to_string(), (hostname.to_string(), *port))),
        );
        let locs = Vec::from_iter(config.keys().map(|s| s.clone()));

        let queue_map = {
            let mut m = HashMap::new();
            for loc in &locs {
                m.insert(loc.to_string(), BlockingQueue::new());
            }
            Arc::new(m)
        };

        let (hostname, port) = config.get(at).unwrap();
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
            config,
            agent,
            queue_map,
            join_handle,
            server,
        }
    }
}

impl Drop for HttpTransport {
    fn drop(&mut self) {
        self.server.unblock();
        self.join_handle.take().map(thread::JoinHandle::join);
    }
}

impl Transport for HttpTransport {
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

    #[test]
    fn test_http_transport() {
        let v = 42;
        let mut config = HashMap::new();
        let (signal, wait) = mpsc::channel::<()>();
        config.insert(Alice::name(), ("localhost", 9010));
        config.insert(Bob::name(), ("localhost", 9011));
        let mut handles = Vec::new();
        {
            let config = config.clone();
            handles.push(thread::spawn(move || {
                wait.recv().unwrap(); // wait for Bob to start
                let transport = HttpTransport::new(Alice::name(), &config);
                transport.send::<i32>(Alice::name(), Bob::name(), &v);
            }));
        }
        {
            let config = config.clone();
            handles.push(thread::spawn(move || {
                let transport = HttpTransport::new(Bob::name(), &config);
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
        let mut config = HashMap::new();
        let (signal, wait) = mpsc::channel::<()>();
        config.insert(Alice::name(), ("localhost", 9020));
        config.insert(Bob::name(), ("localhost", 9021));
        let mut handles = Vec::new();
        {
            let config = config.clone();
            handles.push(thread::spawn(move || {
                signal.send(()).unwrap();
                let transport = HttpTransport::new(Alice::name(), &config);
                transport.send::<i32>(Alice::name(), Bob::name(), &v);
            }));
        }
        {
            let config = config.clone();
            handles.push(thread::spawn(move || {
                // wait for Alice to start, which forces Alice to retry
                wait.recv().unwrap();
                sleep(Duration::from_millis(100));
                let transport = HttpTransport::new(Bob::name(), &config);
                let v2 = transport.receive::<i32>(Alice::name(), Bob::name());
                assert_eq!(v, v2);
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
