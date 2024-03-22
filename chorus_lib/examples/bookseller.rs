extern crate chorus_lib;

use std::io;
use std::thread;

use chorus_lib::transport::local::LocalTransportChannelBuilder;
use chrono::NaiveDate;

use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, LocationSet, Projector};
use chorus_lib::transport::local::LocalTransport;

struct BookEntry {
    price: i32,
    delivery_date: NaiveDate,
}

impl BookEntry {
    fn new(price: i32, delivery_date: NaiveDate) -> Self {
        BookEntry {
            price,
            delivery_date,
        }
    }
}

fn get_book(title: &str) -> Option<BookEntry> {
    match title.trim() {
        "TAPL" => Some(BookEntry::new(
            80,
            NaiveDate::from_ymd_opt(2023, 8, 3).unwrap(),
        )),
        "HoTT" => Some(BookEntry::new(
            120,
            NaiveDate::from_ymd_opt(2023, 9, 18).unwrap(),
        )),
        _ => None,
    }
}

const BUDGET: i32 = 100;

#[derive(ChoreographyLocation)]
struct Seller;

#[derive(ChoreographyLocation)]
struct Buyer;

struct BooksellerChoreography;
impl Choreography for BooksellerChoreography {
    type L = LocationSet!(Seller, Buyer);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let title_at_buyer = op.locally(Buyer, |_| {
            println!("Enter the title of the book to buy (TAPL or HoTT)");
            let mut title = String::new();
            io::stdin().read_line(&mut title).unwrap();
            title
        });
        op.locally(Buyer, |un| {
            println!("Title: {}", un.unwrap(&title_at_buyer));
        });
        let title_at_seller = op.comm(Buyer, Seller, &title_at_buyer);
        let price_at_seller = op.locally(Seller, |un| {
            let title = un.unwrap(&title_at_seller);
            get_book(&title).map(|entry| entry.price)
        });
        let price_at_buyer = op.comm(Seller, Buyer, &price_at_seller);
        let decision_at_buyer = op.locally(Buyer, |un| {
            un.unwrap(&price_at_buyer)
                .map(|price| price < BUDGET)
                .unwrap_or(false) // if the book is not found, the buyer cannot buy it
        });
        let decision = op.broadcast(Buyer, decision_at_buyer);
        if decision {
            let delivery_date_at_seller = op.locally(Seller, |un| {
                let title = un.unwrap(&title_at_seller);
                get_book(&title).unwrap().delivery_date
            });
            let delivery_date_at_buyer = op.comm(Seller, Buyer, &delivery_date_at_seller);
            op.locally(Buyer, |un| {
                let delivery_date = un.unwrap(&delivery_date_at_buyer);
                println!("The book will be delivered on {}", delivery_date);
            });
        } else {
            op.locally(Buyer, |_| {
                println!("The buyer cannot buy the book");
            });
        }
    }
}

fn main() {
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Seller)
        .with(Buyer)
        .build();
    let transport_seller = LocalTransport::new(Seller, transport_channel.clone());
    let transport_buyer = LocalTransport::new(Buyer, transport_channel.clone());

    let seller_projector = Projector::new(Seller, transport_seller);
    let buyer_projector = Projector::new(Buyer, transport_buyer);

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    handles.push(thread::spawn(move || {
        seller_projector.epp_and_run(BooksellerChoreography);
    }));
    handles.push(thread::spawn(move || {
        buyer_projector.epp_and_run(BooksellerChoreography);
    }));
    for h in handles {
        h.join().unwrap();
    }
}
