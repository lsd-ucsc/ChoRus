extern crate chorus_lib;

use std::collections::HashMap;
use std::thread;

use chorus_lib::transport_config;
use chorus_lib::LocationSet;
use chorus_lib::{
    core::{ChoreoOp, Choreography, ChoreographyLocation, Located, Projector},
    transport::local::LocalTransport,
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
    type L = LocationSet!(Buyer1, Buyer2);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<bool, Buyer1> {
        let price = op.broadcast(Buyer1, self.price);
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
    type L = LocationSet!(Buyer1, Buyer2);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<bool, Buyer1> {
        let remaining = op.locally(Buyer1, |un| {
            const BUYER1_BUDGET: i32 = 100;
            return un.unwrap(&self.price) - BUYER1_BUDGET;
        });
        let remaining = op.comm(Buyer1, Buyer2, &remaining);
        let decision = op.locally(Buyer2, |un| {
            const BUYER2_BUDGET: i32 = 200;
            return *un.unwrap(&remaining) < BUYER2_BUDGET;
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

impl<D: Choreography<Located<bool, Buyer1>, L = LocationSet!(Buyer1, Buyer2)> + Decider>
    Choreography<Located<Option<NaiveDate>, Buyer1>> for BooksellerChoreography<D>
{
    type L = LocationSet!(Buyer1, Buyer2, Seller);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Option<NaiveDate>, Buyer1> {
        let title_at_seller = op.comm(Buyer1, Seller, &self.title);
        let price_at_seller = op.locally(Seller, |un| {
            let inventory = un.unwrap(&self.inventory);
            let title = un.unwrap(&title_at_seller);
            if let Some((price, _)) = inventory.get(title) {
                return *price;
            }
            return i32::MAX;
        });
        let price_at_buyer1 = op.comm(Seller, Buyer1, &price_at_seller);
        let decision_at_buyer1 = op.colocally(D::new(price_at_buyer1));

        struct GetDeliveryDateChoreography {
            inventory: Located<Inventory, Seller>,
            title_at_seller: Located<String, Seller>,
            decision_at_buyer1: Located<bool, Buyer1>,
        }
        impl Choreography<Located<Option<NaiveDate>, Buyer1>> for GetDeliveryDateChoreography {
            type L = LocationSet!(Buyer1, Seller);
            fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Option<NaiveDate>, Buyer1> {
                let decision = op.broadcast(Buyer1, self.decision_at_buyer1);
                if decision {
                    let delivery_date_at_seller = op.locally(Seller, |un| {
                        let title = un.unwrap(&self.title_at_seller);
                        let inventory = un.unwrap(&self.inventory);
                        let (_, delivery_date) = inventory.get(title).unwrap();
                        return Some(*delivery_date);
                    });
                    let delivery_date_at_buyer1 = op.comm(Seller, Buyer1, &delivery_date_at_seller);
                    return delivery_date_at_buyer1;
                } else {
                    return op.locally(Buyer1, |_| None);
                }
            }
        }

        return op.colocally(GetDeliveryDateChoreography {
            inventory: self.inventory.clone(),
            title_at_seller: title_at_seller.clone(),
            decision_at_buyer1,
        });
    }
}

fn main() {
    let inventory = {
        let mut i = Inventory::new();
        i.insert(
            "TAPL".to_string(),
            (50, NaiveDate::from_ymd_opt(2023, 8, 3).unwrap()),
        );
        i.insert(
            "HoTT".to_string(),
            (150, NaiveDate::from_ymd_opt(2023, 9, 18).unwrap()),
        );
        i
    };

    let transport_channel =
        LocalTransport::<LocationSet!(Seller, Buyer1, Buyer2)>::transport_channel();

    println!("Tries to buy HoTT with one buyer");
    type OneBuyerBooksellerChoreography = BooksellerChoreography<OneBuyerDecider>;
    let mut handles = Vec::new();
    {
        let config = transport_config!(
            Seller,
            Seller: (),
            Buyer1: (),
            Buyer2: (),
        );

        let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(&config, transport_channel);

        let seller_projector = Projector::new(Seller, transport);

        let inventory = inventory.clone();
        handles.push(thread::spawn(move || {
            seller_projector.epp_and_run(OneBuyerBooksellerChoreography {
                _marker: std::marker::PhantomData,
                inventory: seller_projector.local(inventory),
                title: seller_projector.remote(Buyer1),
            });
        }));
    }
    {
        let config = transport_config!(
            Buyer1,
            Seller: (),
            Buyer1: (),
            Buyer2: (),
        );

        let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(&config, transport_channel);

        let buyer1_projector = Projector::new(Buyer1, transport);

        handles.push(thread::spawn(move || {
            let result = buyer1_projector.epp_and_run(OneBuyerBooksellerChoreography {
                _marker: std::marker::PhantomData,
                inventory: buyer1_projector.remote(Seller),
                title: buyer1_projector.local("HoTT".to_string()),
            });
            println!(
                "The book will be delivered on {:?}",
                buyer1_projector.unwrap(result)
            );
        }));
    }
    {
        let config = transport_config!(
            Buyer2,
            Seller: (),
            Buyer1: (),
            Buyer2: (),
        );

        let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(&config, transport_channel);

        let buyer2_projector = Projector::new(Buyer2, transport);

        handles.push(thread::spawn(move || {
            buyer2_projector.epp_and_run(OneBuyerBooksellerChoreography {
                _marker: std::marker::PhantomData,
                inventory: buyer2_projector.remote(Seller),
                title: buyer2_projector.remote(Buyer1),
            });
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    println!("Tries to buy HoTT with two buyer");
    type TwoBuyerBooksellerChoreography = BooksellerChoreography<TwoBuyerDecider>;
    let mut handles = Vec::new();
    {
        let config = transport_config!(
            Seller,
            Seller: (),
            Buyer1: (),
            Buyer2: (),
        );

        let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(&config, transport_channel);

        let seller_projector = Projector::new(Seller, transport);

        let inventory = inventory.clone();
        handles.push(thread::spawn(move || {
            seller_projector.epp_and_run(TwoBuyerBooksellerChoreography {
                _marker: std::marker::PhantomData,
                inventory: seller_projector.local(inventory),
                title: seller_projector.remote(Buyer1),
            });
        }));
    }
    {
        let config = transport_config!(
            Buyer1,
            Seller: (),
            Buyer1: (),
            Buyer2: (),
        );

        let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(&config, transport_channel);

        let buyer1_projector = Projector::new(Buyer1, transport);

        handles.push(thread::spawn(move || {
            let result = buyer1_projector.epp_and_run(TwoBuyerBooksellerChoreography {
                _marker: std::marker::PhantomData,
                inventory: buyer1_projector.remote(Seller),
                title: buyer1_projector.local("HoTT".to_string()),
            });
            println!(
                "The book will be delivered on {:?}",
                buyer1_projector.unwrap(result)
            );
        }));
    }
    {
        let config = transport_config!(
            Buyer2,
            Seller: (),
            Buyer1: (),
            Buyer2: (),
        );

        let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(&config, transport_channel);

        let buyer2_projector = Projector::new(Buyer2, transport);

        handles.push(thread::spawn(move || {
            buyer2_projector.epp_and_run(TwoBuyerBooksellerChoreography {
                _marker: std::marker::PhantomData,
                inventory: buyer2_projector.remote(Seller),
                title: buyer2_projector.remote(Buyer1),
            });
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
