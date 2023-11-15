use std::env;

use chorus_lib::transport::http::{HttpTransport, HttpTransportConfigBuilder};

use kvs::handwritten::backup;
use kvs::handwritten::{client, primary};
use kvs::shared::*;

fn main() {
    let location = env::args()
        .nth(1)
        .expect("Expected location: client, primary, or backup");
    match location.as_str() {
        "client" => {
            println!("Starting client...");
            let config = HttpTransportConfigBuilder::for_target(Client, ("0.0.0.0", 3000))
                .with(Primary, ("0.0.0.0", 3001))
                .with(Backup, ("0.0.0.0", 3002))
                .build();
            let transport = HttpTransport::new(config);
            loop {
                let request = read_request();
                let response = client(&transport, request);
                println!("Response: {:?}", response);
            }
        }
        "primary" => {
            println!("Starting primary...");
            let state = State::default();
            let config = HttpTransportConfigBuilder::for_target(Primary, ("0.0.0.0", 3001))
                .with(Client, ("0.0.0.0", 3000))
                .with(Backup, ("0.0.0.0", 3002))
                .build();
            let transport = HttpTransport::new(config);
            loop {
                primary(&transport, &state);
            }
        }
        "backup" => {
            println!("Starting backup...");
            let state = State::default();
            let config = HttpTransportConfigBuilder::for_target(Backup, ("0.0.0.0", 3002))
                .with(Primary, ("0.0.0.0", 3001))
                .with(Client, ("0.0.0.0", 3000))
                .build();
            let transport = HttpTransport::new(config);
            loop {
                backup(&transport, &state);
            }
        }
        _ => panic!("Invalid location: {}", location),
    };
}
