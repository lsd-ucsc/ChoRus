extern crate chorus_lib;

use std::io;
use std::thread;

use chorus_lib::{core::Located, transport::local::LocalTransportChannelBuilder};
use chrono::NaiveDate;

use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, LocationSet, Projector};
use chorus_lib::transport::local::LocalTransport;

fn get_book(title: &str) -> Option<(i32, NaiveDate)> {
    match title.trim() {
        "TAPL" => Some((80, NaiveDate::from_ymd_opt(2023, 8, 3).unwrap())),
        "HoTT" => Some((120, NaiveDate::from_ymd_opt(2023, 9, 18).unwrap())),
        _ => None,
    }
}

const BUDGET: i32 = 100;

#[derive(ChoreographyLocation)]
struct Seller;

#[derive(ChoreographyLocation)]
struct Buyer;

struct BooksellerChoreography {
    title: Located<String, Buyer>,
    budget: Located<i32, Buyer>,
}
impl Choreography<bool> for BooksellerChoreography {
    type L = LocationSet!(Seller, Buyer);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> bool {
        let title_at_seller = op.comm(Buyer, Seller, &self.title);
        let price_at_seller = op.locally(Seller, |un| {
            let title = un.unwrap(&title_at_seller);
            if let Some((price, _)) = get_book(&title) {
                return Some(price);
            }
            return None;
        });
        let price_at_buyer = op.comm(Seller, Buyer, &price_at_seller);
        let decision_at_buyer = op.locally(Buyer, |un| {
            if let Some(price) = un.unwrap(&price_at_buyer) {
                println!("Price is {}", price);
                return *price < *un.unwrap(&self.budget);
            }
            println!("The book does not exist");
            return false;
        });
        let decision = op.broadcast(Buyer, decision_at_buyer);
        if decision {
            let delivery_date_at_seller = op.locally(Seller, |un| {
                let title = un.unwrap(&title_at_seller);
                let (_, delivery_date) = get_book(&title).unwrap();
                return delivery_date;
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
        decision
    }
}

fn main() {
    println!("Enter the title of the book to buy (TAPL or HoTT)");
    let mut title = String::new();
    io::stdin().read_line(&mut title).unwrap();

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
        seller_projector.epp_and_run(BooksellerChoreography {
            title: seller_projector.remote(Buyer),
            budget: seller_projector.remote(Buyer),
        });
    }));
    handles.push(thread::spawn(move || {
        buyer_projector.epp_and_run(BooksellerChoreography {
            title: buyer_projector.local(title),
            budget: buyer_projector.local(BUDGET),
        });
    }));
    for h in handles {
        h.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distributed_tapl() {
        let title = String::from("TAPL");

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
            seller_projector.epp_and_run(BooksellerChoreography {
                title: seller_projector.remote(Buyer),
                budget: seller_projector.remote(Buyer),
            });
        }));
        handles.push(thread::spawn(move || {
            buyer_projector.epp_and_run(BooksellerChoreography {
                title: buyer_projector.local(title),
                budget: buyer_projector.local(BUDGET),
            });
        }));
        for h in handles {
            h.join().unwrap();
        }
    }
}
