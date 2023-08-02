extern crate chorus;

use std::io;
use std::thread;

use chrono::NaiveDate;

use chorus::backend::local::LocalBackend;
use chorus::core::{epp_and_run, ChoreoOp, Choreography, ChoreographyLocation};

fn get_book(title: &str) -> Option<(i32, NaiveDate)> {
    match title.trim() {
        "TAPL" => Some((80, NaiveDate::from_ymd_opt(2023, 8, 3).unwrap())),
        "HoTT" => Some((120, NaiveDate::from_ymd_opt(2023, 9, 18).unwrap())),
        _ => None,
    }
}

const BUDGET: i32 = 100;

struct Seller;
impl ChoreographyLocation for Seller {
    fn name(&self) -> &'static str {
        "Seller"
    }
}
const SELLER: Seller = Seller;

struct Buyer;
impl ChoreographyLocation for Buyer {
    fn name(&self) -> &'static str {
        "Buyer"
    }
}
const BUYER: Buyer = Buyer;

struct BooksellerChoreography;
impl Choreography for BooksellerChoreography {
    fn run(&self, op: &impl ChoreoOp) {
        let title_at_buyer = op.locally(BUYER, |_| {
            println!("Enter the title of the book to buy (TAPL or HoTT)");
            let mut title = String::new();
            io::stdin().read_line(&mut title).unwrap();
            title
        });
        let title_at_seller = op.comm(BUYER, SELLER, title_at_buyer);
        let price_at_seller = op.locally(SELLER, |un| {
            let title = un.unwrap(&title_at_seller);
            if let Some((price, _)) = get_book(&title) {
                return Some(price);
            }
            return None;
        });
        let price_at_buyer = op.comm(SELLER, BUYER, price_at_seller);
        let decision_at_buyer = op.locally(BUYER, |un| {
            if let Some(price) = un.unwrap(&price_at_buyer) {
                println!("Price is {}", price);
                return price < BUDGET;
            }
            println!("The book does not exist");
            return false;
        });
        let decision = op.broadcast(BUYER, decision_at_buyer);
        if decision {
            let delivery_date_at_seller = op.locally(SELLER, |un| {
                let title = un.unwrap(&title_at_seller);
                let (_, delivery_date) = get_book(&title).unwrap();
                return delivery_date;
            });
            let delivery_date_at_buyer = op.comm(SELLER, BUYER, delivery_date_at_seller);
            op.locally(BUYER, |un| {
                let delivery_date = un.unwrap(&delivery_date_at_buyer);
                println!("The book will be delivered on {}", delivery_date);
            });
        } else {
            op.locally(BUYER, |_| {
                println!("The buyer cannot buy the book");
            });
        }
    }
}

fn main() {
    let backend = LocalBackend::from(vec!["Seller", "Buyer"].into_iter());
    let seller_backend = backend.clone();
    let buyer_backend = backend.clone();

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    handles.push(thread::spawn(|| {
        epp_and_run(BooksellerChoreography, SELLER, seller_backend);
    }));
    handles.push(thread::spawn(|| {
        epp_and_run(BooksellerChoreography, BUYER, buyer_backend);
    }));
    for h in handles {
        h.join().unwrap();
    }
}
