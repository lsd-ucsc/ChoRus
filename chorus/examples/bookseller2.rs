//! This example shows how to achieve higher-order choreographies and location polymorphism using the two-buyer protocol.
//!
//! In the two-buyer protocol, there are one seller and two buyers who coordinate to buy a book.
//! The buyer1

use std::collections::HashMap;
use std::thread;

use chorus::{
    backend::local::LocalBackend,
    core::{ChoreoOp, Choreography, ChoreographyLocation, Located, Projector},
};
use chrono::NaiveDate;

#[derive(ChoreographyLocation)]
struct Buyer1;

#[derive(ChoreographyLocation)]
struct Buyer2;

#[derive(ChoreographyLocation)]
struct Seller;

type Inventory = HashMap<String, (i32, NaiveDate)>;

trait Decider {
    fn new(price: Located<i32, Buyer1>) -> Self;
}

struct OneBuyerDecider {
    price: Located<i32, Buyer1>,
}

impl Decider for OneBuyerDecider {
    fn new(price: Located<i32, Buyer1>) -> Self {
        Self { price }
    }
}

impl Choreography<Located<bool, Buyer1>> for OneBuyerDecider {
    fn run(&self, op: &impl ChoreoOp) -> Located<bool, Buyer1> {
        let price = op.broadcast(Buyer1, &self.price);
        return op.locally(Buyer1, |_| {
            const BUYER1_BUDGET: i32 = 100;
            return price < BUYER1_BUDGET;
        });
    }
}

struct TwoBuyerDecider {
    price: Located<i32, Buyer1>,
}

impl Decider for TwoBuyerDecider {
    fn new(price: Located<i32, Buyer1>) -> Self {
        Self { price }
    }
}

impl Choreography<Located<bool, Buyer1>> for TwoBuyerDecider {
    fn run(&self, op: &impl ChoreoOp) -> Located<bool, Buyer1> {
        let remaining = op.locally(Buyer1, |un| {
            const BUYER1_BUDGET: i32 = 100;
            return un.unwrap(&self.price) - BUYER1_BUDGET;
        });
        let remaining = op.comm(Buyer1, Buyer2, &remaining);
        let decision = op.locally(Buyer2, |un| {
            const BUYER2_BUDGET: i32 = 200;
            return un.unwrap(&remaining) < BUYER2_BUDGET;
        });
        op.comm(Buyer2, Buyer1, &decision)
    }
}

struct BooksellerChoreography<D: Choreography<Located<bool, Buyer1>>> {
    _marker: std::marker::PhantomData<D>,
    // input
    inventory: Located<Inventory, Seller>,
    title: Located<String, Buyer1>,
}

impl<D: Choreography<Located<bool, Buyer1>> + Decider>
    Choreography<Located<Option<NaiveDate>, Buyer1>> for BooksellerChoreography<D>
{
    fn run(&self, op: &impl ChoreoOp) -> Located<Option<NaiveDate>, Buyer1> {
        let title_at_seller = op.comm(Buyer1, Seller, &self.title);
        let price_at_seller = op.locally(Seller, |un| {
            let inventory = un.unwrap(&self.inventory);
            let title = un.unwrap(&title_at_seller);
            if let Some((price, _)) = inventory.get(&title) {
                return *price;
            }
            return i32::MAX;
        });
        let price_at_buyer1 = op.comm(Seller, Buyer1, &price_at_seller);
        let decision_at_buyer1 =
            op.colocally(&[Buyer1.name(), Buyer2.name()], &D::new(price_at_buyer1));

        struct GetDeliveryDateChoreography<'a> {
            inventory: &'a Located<Inventory, Seller>,
            title_at_seller: Located<String, Seller>,
            decision_at_buyer1: Located<bool, Buyer1>,
        }
        impl Choreography<Located<Option<NaiveDate>, Buyer1>> for GetDeliveryDateChoreography<'_> {
            fn run(&self, op: &impl ChoreoOp) -> Located<Option<NaiveDate>, Buyer1> {
                let decision = op.broadcast(Buyer1, &self.decision_at_buyer1);
                if decision {
                    let delivery_date_at_seller = op.locally(Seller, |un| {
                        let title = un.unwrap(&self.title_at_seller);
                        let inventory = un.unwrap(&self.inventory);
                        let (_, delivery_date) = inventory.get(&title).unwrap();
                        return Some(*delivery_date);
                    });
                    let delivery_date_at_buyer1 = op.comm(Seller, Buyer1, &delivery_date_at_seller);
                    return delivery_date_at_buyer1;
                } else {
                    return op.locally(Buyer1, |_| None);
                }
            }
        }

        return op.colocally(
            &[Seller.name(), Buyer1.name()],
            &GetDeliveryDateChoreography {
                inventory: &self.inventory,
                title_at_seller,
                decision_at_buyer1,
            },
        );
    }
}

fn main() {
    let backend = LocalBackend::from(&[Seller.name(), Buyer1.name(), Buyer2.name()]);
    let seller_backend = backend.clone();
    let buyer1_backend = backend.clone();
    let buyer2_backend = backend.clone();

    println!("Tries to buy HoTT with one buyer");
    type OneBuyerBooksellerChoreography = BooksellerChoreography<OneBuyerDecider>;
    let mut handles = Vec::new();
    handles.push(thread::spawn(|| {
        let p = Projector::new(Seller, seller_backend);
        p.epp_and_run(OneBuyerBooksellerChoreography {
            _marker: std::marker::PhantomData,
            inventory: p.local({
                let mut inventory = Inventory::new();
                inventory.insert(
                    "TAPL".to_string(),
                    (50, NaiveDate::from_ymd_opt(2023, 8, 3).unwrap()),
                );
                inventory.insert(
                    "HoTT".to_string(),
                    (150, NaiveDate::from_ymd_opt(2023, 9, 18).unwrap()),
                );
                inventory
            }),
            title: p.remote(Buyer1),
        });
    }));
    handles.push(thread::spawn(|| {
        let p = Projector::new(Buyer1, buyer1_backend);
        let result = p.epp_and_run(OneBuyerBooksellerChoreography {
            _marker: std::marker::PhantomData,
            inventory: p.remote(Seller),
            title: p.local("HoTT".to_string()),
        });
        println!("The book will be delivered on {:?}", p.unwrap(result));
    }));
    handles.push(thread::spawn(|| {
        let p = Projector::new(Buyer2, buyer2_backend);
        p.epp_and_run(OneBuyerBooksellerChoreography {
            _marker: std::marker::PhantomData,
            inventory: p.remote(Seller),
            title: p.remote(Buyer1),
        });
    }));
    for h in handles {
        h.join().unwrap();
    }

    println!("Tries to buy HoTT with two buyer");
    type TwoBuyerBooksellerChoreography = BooksellerChoreography<TwoBuyerDecider>;
    let seller_backend = backend.clone();
    let buyer1_backend = backend.clone();
    let buyer2_backend = backend.clone();
    let mut handles = Vec::new();
    handles.push(thread::spawn(|| {
        let p = Projector::new(Seller, seller_backend);
        p.epp_and_run(TwoBuyerBooksellerChoreography {
            _marker: std::marker::PhantomData,
            inventory: p.local({
                let mut inventory = Inventory::new();
                inventory.insert(
                    "TAPL".to_string(),
                    (50, NaiveDate::from_ymd_opt(2023, 8, 3).unwrap()),
                );
                inventory.insert(
                    "HoTT".to_string(),
                    (150, NaiveDate::from_ymd_opt(2023, 9, 18).unwrap()),
                );
                inventory
            }),
            title: p.remote(Buyer1),
        });
    }));
    handles.push(thread::spawn(|| {
        let p = Projector::new(Buyer1, buyer1_backend);
        let result = p.epp_and_run(TwoBuyerBooksellerChoreography {
            _marker: std::marker::PhantomData,
            inventory: p.remote(Seller),
            title: p.local("HoTT".to_string()),
        });
        println!("The book will be delivered on {:?}", p.unwrap(result));
    }));
    handles.push(thread::spawn(|| {
        let p = Projector::new(Buyer2, buyer2_backend);
        p.epp_and_run(TwoBuyerBooksellerChoreography {
            _marker: std::marker::PhantomData,
            inventory: p.remote(Seller),
            title: p.remote(Buyer1),
        });
    }));
    for h in handles {
        h.join().unwrap();
    }
}
