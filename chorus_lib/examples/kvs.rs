extern crate chorus_lib;

use std::env;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};
use std::{io, process, thread};

use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, Deserialize, Faceted, FanInChoreography, HCons,
    HNil, Located, LocationSet, LocationSetFoldable, Member, MultiplyLocated, Portable, Projector,
    Serialize, Subset,
};
use chorus_lib::transport::http::{HttpTransport, HttpTransportConfigBuilder};

type Response = i32;
type Value = i32;
type Key = String;

#[derive(Serialize, Deserialize)]
enum Request {
    Get(Key),
    Put(Key, Value),
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyValueStore {
    store: HashMap<String, i32>,
}

impl KeyValueStore {
    fn load_from_file(file_path: &str) -> Self {
        if Path::new(file_path).exists() {
            let mut file = OpenOptions::new().read(true).open(file_path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).unwrap_or(Self {
                store: HashMap::new(),
            })
        } else {
            Self {
                store: HashMap::new(),
            }
        }
    }

    fn save_to_file(&self, file_path: &str) {
        let json_data = serde_json::to_string(&self).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .unwrap();
        file.write_all(json_data.as_bytes()).unwrap();
    }
}

fn get_thread_id() -> String {
    let pid = process::id();
    let thread_id = thread::current().id();

    let mut hasher = DefaultHasher::new();
    pid.hash(&mut hasher);
    thread_id.hash(&mut hasher);
    format!("{:x}", hasher.finish()) // Convert to hexadecimal for compactness
}

fn create_data_dir_if_necessary() {
    if !Path::new(".data").exists() {
        fs::create_dir(".data").unwrap();
        let gitignore_path = Path::new(".data/.gitignore");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(gitignore_path)
            .unwrap();
        file.write_all(b"*").unwrap();
    }
}

fn handle_get(key: String) -> i32 {
    let thread_id = get_thread_id();
    let file_path = format!(".data/{}", thread_id);

    create_data_dir_if_necessary();

    let kv_store = KeyValueStore::load_from_file(&file_path);
    kv_store.store.get(&key).cloned().unwrap_or(-1)
}

fn handle_put(key: String, value: i32) -> i32 {
    let thread_id = get_thread_id();
    let file_path = format!(".data/{}", thread_id);

    create_data_dir_if_necessary();

    let mut kv_store = KeyValueStore::load_from_file(&file_path);
    kv_store.store.insert(key, value);
    kv_store.save_to_file(&file_path);
    0
}

#[derive(ChoreographyLocation, Debug)]
struct Client;

#[derive(ChoreographyLocation, Debug)]
struct Server;

#[derive(ChoreographyLocation, Debug)]
struct Backup1;

#[derive(ChoreographyLocation, Debug)]
struct Backup2;

// This should perhaps be in core?
struct Gather<
    'a,
    V,
    Senders: LocationSet + Subset<Census, SendersPresent>,
    Recievers: LocationSet + Subset<Census, RecieversPresent>,
    Census: LocationSet,
    SendersPresent,
    RecieversPresent,
> {
    values: &'a Faceted<V, Senders>,
    phantom: PhantomData<(Census, SendersPresent, Recievers, RecieversPresent)>,
}
impl<
        'a,
        V: Portable + Copy,
        Senders: LocationSet + Subset<Census, SendersPresent>,
        Recievers: LocationSet + Subset<Census, RecieversPresent>,
        Census: LocationSet,
        SendersPresent,
        RecieversPresent,
    > FanInChoreography<V>
    for Gather<'a, V, Senders, Recievers, Census, SendersPresent, RecieversPresent>
{
    type L = Census;
    type QS = Senders;
    type RS = Recievers;
    fn run<
        Sender: ChoreographyLocation,
        _SendersPresent,
        _RecieversPresent,
        SenderPresent,
        SenderInSenders,
    >(
        &self,
        op: &impl ChoreoOp<Self::L>,
    ) -> MultiplyLocated<V, Self::RS>
    where
        Self::QS: Subset<Self::L, SendersPresent>,
        Self::RS: Subset<Self::L, RecieversPresent>,
        Sender: Member<Self::L, SenderPresent>,
        Sender: Member<Self::QS, SenderInSenders>,
    {
        let x = op.locally(Sender::new(), |un| *un.unwrap(self.values));
        let x = op.multicast::<Sender, V, Self::RS, SenderPresent, RecieversPresent>(
            Sender::new(),
            <Self::RS>::new(),
            &x,
        );
        x
    }
}

struct HandleRequest<Backups, BackupsPresent, BSpine> {
    request: Located<Request, Server>,
    _phantoms: PhantomData<(Backups, BackupsPresent, BSpine)>,
}
impl<Backups: LocationSet, BackupsPresent, BSpine> Choreography<Located<Response, Server>>
    for HandleRequest<Backups, BackupsPresent, BSpine>
where
    Backups: Subset<HCons<Server, Backups>, BackupsPresent>,
    Backups: LocationSetFoldable<HCons<Server, Backups>, Backups, BSpine>,
{
    type L = HCons<Server, Backups>;
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Server> {
        match op.broadcast(Server, self.request) {
            Request::Put(key, value) => {
                let oks = op.parallel(Backups::new(), || handle_put(key.clone(), value));
                let gathered = op.fanin::<Response, Backups, HCons<Server, HNil>, _, _, _, _>(
                    Backups::new(),
                    Gather {
                        values: &oks,
                        phantom: PhantomData,
                    },
                );
                op.locally(Server, |un| {
                    let ok = un
                        .unwrap(&gathered)
                        .get_map()
                        .into_values()
                        .all(|response| response == 0);
                    if ok {
                        return handle_put(key.clone(), value);
                    } else {
                        return -1;
                    }
                })
            }
            Request::Get(key) => op.locally(Server, |_| handle_get(key.clone())),
        }
    }
}

struct KVS<Backups: LocationSet, BackupsPresent, BackupsAreServers, BSpine> {
    request: Located<Request, Client>,
    _phantoms: PhantomData<(Backups, BackupsPresent, BackupsAreServers, BSpine)>,
}
impl<Backups: LocationSet, BackupsPresent, BackupsAreServers, BSpine>
    Choreography<Located<Response, Client>>
    for KVS<Backups, BackupsPresent, BackupsAreServers, BSpine>
where
    Backups: Subset<HCons<Client, HCons<Server, Backups>>, BackupsPresent>,
    Backups: Subset<HCons<Server, Backups>, BackupsAreServers>,
    Backups: LocationSetFoldable<HCons<Server, Backups>, Backups, BSpine>,
{
    type L = HCons<Client, HCons<Server, Backups>>;
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Client> {
        let request = op.comm(Client, Server, &self.request);
        let response = op
            .conclave(HandleRequest::<Backups, _, _> {
                request: request,
                _phantoms: PhantomData,
            })
            .flatten();
        op.comm(Server, Client, &response)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 || !["client", "server", "backup1", "backup2"].contains(&args[1].as_str()) {
        eprintln!("Usage: {} [client|server|backup1|backup2]", args[0]);
        process::exit(1);
    }
    let role = args[1].as_str();
    match role {
        "client" => {
            let config = HttpTransportConfigBuilder::for_target(Client, ("0.0.0.0", 9010))
                .with(Server, ("localhost", 9011))
                .with(Backup1, ("localhost", 9012))
                .with(Backup2, ("localhost", 9013))
                .build();
            let transport = HttpTransport::new(config);
            let projector = Projector::new(Client, transport);

            println!("Enter a command in one of the following formats:");
            println!("  get <key>");
            println!("  put <key> <value>");
            println!("Type 'exit' to quit.");
            loop {
                // Read input
                print!("> ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();

                // Parse command
                let parts: Vec<&str> = input.split_whitespace().collect();
                let request = match parts.as_slice() {
                    ["get", key] => Request::Get(key.to_string()),
                    ["put", key, value] if value.parse::<i32>().is_ok() => {
                        Request::Put(key.to_string(), value.parse::<i32>().unwrap())
                    }
                    ["exit"] => break,
                    _ => {
                        eprintln!("Invalid command. Use 'get <key>' or 'put <key> <value>'.");
                        continue;
                    }
                };
                let response =
                    projector.epp_and_run(KVS::<HCons<Backup1, HCons<Backup2, HNil>>, _, _, _> {
                        request: projector.local(request),
                        _phantoms: PhantomData,
                    });
                println!("Response: {:?}", projector.unwrap(response));
            }
        }
        "server" => {
            println!("Server process started.");
            let config = HttpTransportConfigBuilder::for_target(Server, ("0.0.0.0", 9011))
                .with(Client, ("localhost", 9010))
                .with(Backup1, ("localhost", 9012))
                .with(Backup2, ("localhost", 9013))
                .build();
            let transport = HttpTransport::new(config);
            let projector = Projector::new(Server, transport);
            loop {
                projector.epp_and_run(KVS::<HCons<Backup1, HCons<Backup2, HNil>>, _, _, _> {
                    request: projector.remote(Client),
                    _phantoms: PhantomData,
                });
            }
        }
        "backup1" => {
            println!("Backup1 process started.");
            let config = HttpTransportConfigBuilder::for_target(Backup1, ("0.0.0.0", 9012))
                .with(Client, ("localhost", 9010))
                .with(Server, ("localhost", 9011))
                .with(Backup2, ("localhost", 9013))
                .build();
            let transport = HttpTransport::new(config);
            let projector = Projector::new(Backup1, transport);
            loop {
                projector.epp_and_run(KVS::<HCons<Backup1, HCons<Backup2, HNil>>, _, _, _> {
                    request: projector.remote(Client),
                    _phantoms: PhantomData,
                });
            }
        }
        "backup2" => {
            println!("Backup2 process started.");
            let config = HttpTransportConfigBuilder::for_target(Backup2, ("0.0.0.0", 9013))
                .with(Client, ("localhost", 9010))
                .with(Server, ("localhost", 9011))
                .with(Backup1, ("localhost", 9012))
                .build();
            let transport = HttpTransport::new(config);
            let projector = Projector::new(Backup2, transport);
            loop {
                projector.epp_and_run(KVS::<HCons<Backup1, HCons<Backup2, HNil>>, _, _, _> {
                    request: projector.remote(Client),
                    _phantoms: PhantomData,
                });
            }
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};

    fn clear_data() {
        if Path::new(".data").exists() {
            fs::remove_dir_all(".data").unwrap();
        }
    }

    fn handle_requests(scenario: Vec<(Request, Response)>) {
        let n = scenario.len();
        type Locations = LocationSet!(Backup1, Backup2);

        let transport_channel = LocalTransportChannelBuilder::new()
            .with(Client)
            .with(Server)
            .with(Backup1)
            .with(Backup2)
            .build();
        let transport_client = LocalTransport::new(Client, transport_channel.clone());
        let transport_server = LocalTransport::new(Server, transport_channel.clone());
        let transport_backup1 = LocalTransport::new(Backup1, transport_channel.clone());
        let transport_backup2 = LocalTransport::new(Backup2, transport_channel.clone());

        let client_projector = Projector::new(Client, transport_client);
        let server_projector = Projector::new(Server, transport_server);
        let backup1_projector = Projector::new(Backup1, transport_backup1);
        let backup2_projector = Projector::new(Backup2, transport_backup2);

        let mut handles = Vec::new();
        handles.push(
            thread::Builder::new()
                .name("Server".to_string())
                .spawn(move || {
                    for _ in 0..n {
                        server_projector.epp_and_run(KVS::<Locations, _, _, _> {
                            request: server_projector.remote(Client),
                            _phantoms: PhantomData,
                        });
                    }
                })
                .unwrap(),
        );
        handles.push(
            thread::Builder::new()
                .name("Backup1".to_string())
                .spawn(move || {
                    for _ in 0..n {
                        backup1_projector.epp_and_run(KVS::<Locations, _, _, _> {
                            request: backup1_projector.remote(Client),
                            _phantoms: PhantomData,
                        });
                    }
                })
                .unwrap(),
        );
        handles.push(
            thread::Builder::new()
                .name("Backup2".to_string())
                .spawn(move || {
                    for _ in 0..n {
                        backup2_projector.epp_and_run(KVS::<Locations, _, _, _> {
                            request: backup2_projector.remote(Client),
                            _phantoms: PhantomData,
                        });
                    }
                })
                .unwrap(),
        );
        for (req, expected_response) in scenario {
            let response = client_projector.epp_and_run(KVS::<Locations, _, _, _> {
                request: client_projector.local(req),
                _phantoms: PhantomData,
            });
            assert_eq!(client_projector.unwrap(response), expected_response);
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_kvs() {
        clear_data();
        handle_requests(vec![
            (Request::Get("foo".to_string()), -1),
            (Request::Put("foo".to_string(), 42), 0),
            (Request::Get("foo".to_string()), 42),
            (Request::Put("foo".to_string(), 43), 0),
            (Request::Get("foo".to_string()), 43),
        ]);
    }
}
