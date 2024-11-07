extern crate chorus_lib;

use std::marker::PhantomData;
use std::thread;

use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation,
    Deserialize,
    Faceted,
    FanInChoreography,
    HCons, HNil,
    Located, LocationSet,
    LocationSetFoldable,
    Member,
    MultiplyLocated,
    Projector,
    Portable,
    Quire,
    Serialize,
    Subset,
};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};

type Response = i32;
type Key = String;

#[derive(Serialize, Deserialize)]
enum Request {
    Get(Key),
    Put(Key, i32),
}

fn handle_get(key: Key) -> Response {
    key.len().try_into().unwrap()
}

fn handle_put(key: Key, val: i32) -> Response {
    (val != handle_get(key)) as Response
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
struct Gather<'a,
              V,
              Senders: LocationSet + Subset<Census, SendersPresent>,
              Recievers: LocationSet + Subset<Census, RecieversPresent>,
              Census: LocationSet,
              SendersPresent,
              RecieversPresent>
{
    values: &'a Faceted<V, Senders>,
    phantom: PhantomData<(Census, SendersPresent, Recievers, RecieversPresent)>,
}
impl<'a,
     V: Portable + Copy,
     Senders: LocationSet + Subset<Census, SendersPresent>,
     Recievers: LocationSet + Subset<Census, RecieversPresent>,
     Census: LocationSet,
     SendersPresent,
     RecieversPresent>
  FanInChoreography<V> for Gather<'a, V, Senders, Recievers, Census, SendersPresent, RecieversPresent>
{
    type L = Census;
    type QS = Senders;
    type RS = Recievers;
    fn run<Sender: ChoreographyLocation, _SendersPresent, _RecieversPresent, SenderPresent, SenderInSenders>(
        &self,
        op: &impl ChoreoOp<Self::L>,
    ) -> MultiplyLocated<V, Self::RS>
    where
        Self::QS: Subset<Self::L, SendersPresent>,
        Self::RS: Subset<Self::L, RecieversPresent>,
        Sender: Member<Self::L, SenderPresent>,
        Sender: Member<Self::QS, SenderInSenders>,
    {
        let x = op.locally(Sender::new(), |un| *un.unwrap3(&self.values));
        let x = op.multicast::<Sender, V, Self::RS, SenderPresent, RecieversPresent>(
            Sender::new(),
            <Self::RS>::new(),
            &x,
        );
        x
    }
}


struct HandleRequest<Backups, BackupsPresent, BSpine>
{
    request: Located<Request, Server>,
    _phantoms: PhantomData<(Backups, BackupsPresent, BSpine)>,
}
impl<Backups: LocationSet, BackupsPresent, BSpine>
  Choreography<Located<Response, Server>> for HandleRequest<Backups, BackupsPresent, BSpine>
where Backups: Subset<HCons<Server, Backups>, BackupsPresent>,
      Backups: LocationSetFoldable<HCons<Server, Backups>, Backups, BSpine>
{
    type L = HCons<Server, Backups>;
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Server> {
        match op.broadcast(Server, self.request) {
            Request::Put(key, value) => {
                let oks = op.parallel(Backups::new(), ||{ handle_put(key.clone(), value) });
                let gathered = op.fanin::<Response, Backups, HCons<Server, HNil>, _, _, _, _>(
                    Backups::new(),
                    Gather{values: &oks, phantom: PhantomData}
                    );
                op.locally(Server, |un| {
                  let ok = un.unwrap::<Quire<Response, Backups>, _, _>(&gathered).get_map().into_values().all(|response|{response == 0});
                  if ok {
                    return handle_put(key.clone(), value)
                  } else {
                    return -1
                  }
                })
            }
            Request::Get(key) => op.locally(Server, |_| { handle_get(key.clone()) })
        }
    }
}

struct KVS<Backups: LocationSet,
           BackupsPresent,
           BackupsAreServers,
           BSpine>
{
    request: Located<Request, Client>,
    _phantoms: PhantomData<(Backups, BackupsPresent, BackupsAreServers, BSpine)>,
}
impl<Backups: LocationSet, BackupsPresent, BackupsAreServers, BSpine>
  Choreography<Located<Response, Client>> for KVS<Backups, BackupsPresent, BackupsAreServers, BSpine>
where Backups: Subset<HCons<Client, HCons<Server, Backups>>, BackupsPresent>,
      Backups: Subset<HCons<Server, Backups>, BackupsAreServers>,
      Backups: LocationSetFoldable<HCons<Server, Backups>, Backups, BSpine>
{
    type L = HCons<Client, HCons<Server, Backups>>;
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Client> {
        let request = op.comm(Client, Server, &self.request);
        let response = op.enclave(HandleRequest::<Backups, _, _>{
            request: request,
            _phantoms: PhantomData
        }).flatten();
        op.comm(Server, Client, &response)
    }
}

fn run_test(request: Request, answer: Response) {
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Client)
        .with(Server)
        .with(Backup1)
        .with(Backup2)
        .build();
    let transport_client  = LocalTransport::new(Client, transport_channel.clone());
    let transport_server  = LocalTransport::new(Server, transport_channel.clone());
    let transport_backup1 = LocalTransport::new(Backup1, transport_channel.clone());
    let transport_backup2 = LocalTransport::new(Backup2, transport_channel.clone());

    let client_projector  = Projector::new(Client, transport_client);
    let server_projector  = Projector::new(Server, transport_server);
    let backup1_projector = Projector::new(Backup1, transport_backup1);
    let backup2_projector = Projector::new(Backup2, transport_backup2);

    let mut handles: Vec<thread::JoinHandle<Located<Response, Client>>> = Vec::new();
    handles.push(
        thread::Builder::new()
            .name("Client".to_string())
            .spawn(move || {
                client_projector.epp_and_run(KVS::<HCons<Backup1, HCons<Backup2, HNil>>, _, _, _>{
                    request: client_projector.local(request),
                    _phantoms: PhantomData,
                })
            })
            .unwrap(),
    );
    handles.push(
        thread::Builder::new()
            .name("Server".to_string())
            .spawn(move || {
                server_projector.epp_and_run(KVS::<HCons<Backup1, HCons<Backup2, HNil>>, _, _, _>{
                    request: server_projector.remote(Client),
                    _phantoms: PhantomData,
                })
            })
            .unwrap(),
    );
    handles.push(
        thread::Builder::new()
            .name("Backup1".to_string())
            .spawn(move || {
                backup1_projector.epp_and_run(KVS::<HCons<Backup1, HCons<Backup2, HNil>>, _, _, _>{
                    request: backup1_projector.remote(Client),
                    _phantoms: PhantomData,
                })
            })
            .unwrap(),
    );
    handles.push(
        thread::Builder::new()
            .name("Backup2".to_string())
            .spawn(move || {
                backup2_projector.epp_and_run(KVS::<HCons<Backup1, HCons<Backup2, HNil>>, _, _, _>{
                    request: backup2_projector.remote(Client),
                    _phantoms: PhantomData,
                })
            })
            .unwrap(),
    );
    let retval = Projector::new(Client, LocalTransport::new(Client, transport_channel.clone()))
        .unwrap(handles.pop().unwrap().join().unwrap());
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(retval, answer);
}

#[test]
fn main() {
    let two = "xx".to_string();
    let three = "xxx".to_string();
    run_test(Request::Get(two.clone()), 2);
    run_test(Request::Get(three.clone()), 3);
    run_test(Request::Put(two.clone(), 2), 0);
    run_test(Request::Put(three.clone(), 2), -1);
}
