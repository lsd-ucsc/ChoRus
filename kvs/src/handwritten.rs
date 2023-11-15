use chorus_lib::{
    core::{ChoreographyLocation, LocationSet, Transport},
    transport::http::HttpTransport,
};

use crate::shared::*;

pub fn client(
    transport: &HttpTransport<LocationSet!(Backup, Primary, Client), Client>,
    request: Request,
) -> Response {
    transport.send(Client::name(), Primary::name(), &request);
    let response = transport.receive::<Response>(Primary::name(), Client::name());
    return response;
}

pub fn primary(
    transport: &HttpTransport<LocationSet!(Backup, Client, Primary), Primary>,
    state: &State,
) -> () {
    let request = transport.receive::<Request>(Client::name(), Primary::name());
    let is_mutating = request.is_mutating();
    transport.send(Primary::name(), Backup::name(), &is_mutating);
    if is_mutating {
        transport.send(Primary::name(), Backup::name(), &request);
        transport.receive::<Response>(Backup::name(), Primary::name());
    }
    let response = handle_request(state, &request);
    transport.send(Primary::name(), Client::name(), &response);
}

pub fn backup(transport: &HttpTransport<L, Backup>, state: &State) -> () {
    let is_mutating = transport.receive::<bool>(Primary::name(), Backup::name());
    if is_mutating {
        let request = transport.receive::<Request>(Primary::name(), Backup::name());
        let response = handle_request(state, &request);
        transport.send(Backup::name(), Primary::name(), &response);
    }
}
