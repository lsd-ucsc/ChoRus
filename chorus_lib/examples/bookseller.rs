extern crate chorus_lib;

use std::io;
use std::thread;

use chorus_lib::hlist;
use chrono::NaiveDate;

use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, ProjectorForAL};
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

struct BooksellerChoreography;
impl Choreography for BooksellerChoreography {
    type L = hlist!(Seller, Buyer);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let title_at_buyer = op.locally(Buyer, |_| {
            println!("Enter the title of the book to buy (TAPL or HoTT)");
            let mut title = String::new();
            io::stdin().read_line(&mut title).unwrap();
            title
        });
        let title_at_seller = op.comm(Buyer, Seller, &title_at_buyer);
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
                return *price < BUDGET;
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
    }
}

fn main() {
    let transport = LocalTransport::from(&[Seller::name(), Buyer::name()]);
    type AL = hlist!(Buyer, Seller); 
    let seller_projector = ProjectorForAL::<AL>::new(Seller, transport.clone());
    let buyer_projector = ProjectorForAL::<AL>::new(Buyer, transport.clone());

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
