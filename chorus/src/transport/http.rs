use std::thread;
use std::{collections::HashMap, sync::Arc};

use reqwest::blocking::Client;
use retry::{
    delay::{jitter, Exponential},
    retry,
};
use tiny_http::Server;

use crate::{core::Transport, utils::queue::BlockingQueue};

/// The header name for the source location.
const HEADER_SRC: &str = "X-CHORUS-SOURCE";

/// The HTTP transport.
pub struct HttpTransport {
    config: HashMap<String, (String, u32)>,
    client: Client,
    queue_map: Arc<HashMap<String, BlockingQueue<String>>>,
    server: Arc<Server>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl HttpTransport {
    pub fn new(at: &'static str, config: &HashMap<&'static str, (&'static str, u32)>) -> Self {
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

        Self {
            config,
            client: Client::new(),
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

    fn send<V: crate::core::ChoreographicValue>(&self, from: &str, to: &str, data: V) -> () {
        let (hostname, port) = self.config.get(to).unwrap();
        retry(
            Exponential::from_millis(10).map(jitter).take(10),
            move || {
                let d = data.clone();
                self.client
                    .post(format!("http://{}:{}", hostname, port))
                    .body(serde_json::to_string(&d).unwrap())
                    .header(HEADER_SRC, from)
                    .send()
            },
        )
        .unwrap();
    }

    fn receive<V: crate::core::ChoreographicValue>(&self, from: &str, _at: &str) -> V {
        let str = self.queue_map.get(from).unwrap().pop();
        serde_json::from_str(&str).unwrap()
    }
}
