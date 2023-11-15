use std::env;

use chorus_lib::{
    core::Projector,
    transport::http::{HttpTransport, HttpTransportConfigBuilder},
};
use kvs::choreographic::*;
use kvs::shared::*;

fn main() {
    let location = env::args()
        .nth(1)
        .expect("Expected location: client, primary, or backup");
    match location.as_str() {
        "client" => {
            println!("Starting client...");
            let config = HttpTransportConfigBuilder::for_target(Client, ("127.0.0.1", 3000))
                .with(Primary, ("127.0.0.1", 3001))
                .with(Backup, ("127.0.0.1", 3002))
                .build();
            let transport = HttpTransport::new(config);
            let projector = Projector::new(Client, transport);
            loop {
                let request = read_request();
                let response = projector.epp_and_run(PrimaryBackupKvsChoreography {
                    state: (projector.remote(Primary), projector.remote(Backup)),
                    request: projector.local(request),
                });
                println!("Response: {:?}", projector.unwrap(response));
            }
        }
        "primary" => {
            println!("Starting primary...");
            let state = State::default();
            let config = HttpTransportConfigBuilder::for_target(Primary, ("127.0.0.1", 3001))
                .with(Backup, ("127.0.0.1", 3002))
                .with(Client, ("127.0.0.1", 3000))
                .build();
            let transport = HttpTransport::new(config);
            let projector = Projector::new(Primary, transport);
            loop {
                projector.epp_and_run(PrimaryBackupKvsChoreography {
                    state: (projector.local(&state), projector.remote(Backup)),
                    request: projector.remote(Client),
                });
            }
        }
        "backup" => {
            println!("Starting backup...");
            let state = State::default();
            let config = HttpTransportConfigBuilder::for_target(Backup, ("127.0.0.1", 3002))
                .with(Primary, ("127.0.0.1", 3001))
                .with(Client, ("127.0.0.1", 3000))
                .build();
            let transport = HttpTransport::new(config);
            let projector = Projector::new(Backup, transport);
            loop {
                projector.epp_and_run(PrimaryBackupKvsChoreography {
                    state: (projector.remote(Primary), projector.local(&state)),
                    request: projector.remote(Client),
                });
            }
        }
        _ => panic!("Invalid location: {}", location),
    };
}
