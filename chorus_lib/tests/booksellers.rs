extern crate chorus_lib;

use std::collections::HashMap;
use std::cmp::max;
use std::marker::PhantomData;
use std::sync::Arc;
use std::thread;

use chorus_lib::{
    core::{ChoreoOp, Choreography, ChoreographyLocation, Located, LocationSet, Projector, Runner},
    transport::local::{LocalTransport, LocalTransportChannelBuilder},
};
use chrono::NaiveDate;

#[derive(ChoreographyLocation)]
struct Buyer1;

#[derive(ChoreographyLocation)]
struct Buyer2;

#[derive(ChoreographyLocation)]
struct Seller;

type Money = i32;
type Title = String;

type Inventory = HashMap<Title, (Money, NaiveDate)>;


trait Decider {
    type Budgets;
    fn new(price: Located<Option<Money>, Buyer1>, budgets: Self::Budgets) -> Self;
}

struct Booksellers<D: Choreography<Located<bool, Buyer1>> + Decider<Budgets=Budgets>, Budgets> {
    inventory: Located<Inventory, Seller>,
    title: Located<Title, Buyer1>,
    budgets: Budgets,
    _marker: PhantomData<D>,
}

impl<D: Choreography<Located<bool, Buyer1>, L = LocationSet!(Buyer1, Buyer2)> + Decider<Budgets=Budgets>, Budgets>
     Choreography<Option<NaiveDate>> for Booksellers<D, Budgets>
{
    type L = LocationSet!(Buyer1, Buyer2, Seller);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Option<NaiveDate> {
        let title_at_seller = op.comm(Buyer1, Seller, &self.title);
        let price_at_seller = op.locally(Seller, |un| {
            let inventory = un.unwrap(&self.inventory);
            let title = un.unwrap(&title_at_seller);
            inventory.get(title).map(|(price, _)|{*price})        });
        let price_at_buyer1 = op.comm(Seller, Buyer1, &price_at_seller);
        let decider = D::new(price_at_buyer1, self.budgets);
        let decision_at_buyer1 = op.call(decider);

        struct GetDeliveryDateChoreography {
            inventory: Located<Inventory, Seller>,
            title_at_seller: Located<Title, Seller>,
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
                        inventory.get(title).map(|(_, delivery_date)| {*delivery_date})
                    });
                    let delivery_date_at_buyer1 = op.comm(Seller, Buyer1, &delivery_date_at_seller);
                    return delivery_date_at_buyer1;
                } else {
                    return op.locally(Buyer1, |_| None);
                }
            }
        }

        return op.broadcast(Buyer1,
                            op.enclave(GetDeliveryDateChoreography {
                                        inventory: self.inventory.clone(),
                                        title_at_seller: title_at_seller.clone(),
                                        decision_at_buyer1,
                                       }).flatten());
    }
}

struct Unilateral {
    price: Located<Option<Money>, Buyer1>,
    budget: Located<Money, Buyer1>,
}
impl Decider for Unilateral {
    type Budgets = Located<Money, Buyer1>;
    fn new(price: Located<Option<Money>, Buyer1>, budgets: Located<Money, Buyer1>) -> Self{
        return Self{price: price, budget: budgets}
    }
}
impl Choreography<Located<bool, Buyer1>> for Unilateral {
    type L = LocationSet!(Buyer1, Buyer2);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<bool, Buyer1> {
        op.locally(Buyer1, |un| {
            match un.unwrap(&self.price) {
                Some(price) => price <= un.unwrap(&self.budget),
                None => false
            }
        })
    }
}


////////////////////////////////////////////////////////////////////////
struct Colaborative {
    price: Located<Option<Money>, Buyer1>,
    budget1: Located<Money, Buyer1>,
    budget2: Located<Money, Buyer2>,
}
impl Decider for Colaborative {
    type Budgets = (Located<Money, Buyer1>, Located<Money, Buyer2>);
    fn new(price: Located<Option<Money>, Buyer1>, (budget1, budget2): (Located<Money, Buyer1>, Located<Money, Buyer2>)) -> Self{
        return Self{price: price, budget1: budget1, budget2: budget2}
    }
}
impl Choreography<Located<bool, Buyer1>> for Colaborative {
    type L = LocationSet!(Buyer1, Buyer2);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<bool, Buyer1> {
        match op.broadcast(Buyer1, self.price) {
            Some(price) => {
                    let remainder = op.comm(Buyer2, Buyer1, &op.locally(Buyer2, |un| {
                        max(0, price - un.unwrap(&self.budget2))
                    }));
                    op.locally(Buyer1, |un| {un.unwrap(&remainder) <= un.unwrap(&self.budget1)})
                },
            None => op.locally(Buyer1, |_| {false})
        }
    }
}


fn run_test(inventory: Inventory, title: Title, budget1: Money, budget2: Option<Money>, answer: Option<NaiveDate>) {
    let central_runner = Runner::new();
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Seller).with(Buyer1).with(Buyer2)
        .build();
    let seller_projector = Arc::new(Projector::new(
        Seller,
        LocalTransport::new(Seller, transport_channel.clone()),
    ));
    let buyer1_projector = Arc::new(Projector::new(
        Buyer1,
        LocalTransport::new(Buyer1, transport_channel.clone()),
    ));
    let buyer2_projector = Arc::new(Projector::new(
        Buyer2,
        LocalTransport::new(Buyer2, transport_channel.clone()),
    ));
    let mut handles = Vec::new();

    if let Some(budget2) = budget2 {
        {
            let choreo : Booksellers<Colaborative, (Located<Money, Buyer1>, Located<Money, Buyer2>)> = Booksellers{
                inventory: central_runner.local(inventory.clone()),
                title: central_runner.local(title.clone()),
                budgets: (central_runner.local(budget1), central_runner.local(budget2)),
                _marker: PhantomData,
            };
            let central_result = central_runner.run(choreo);
            assert_eq!(central_result, answer);
        }
        {
            handles.push(thread::spawn(move || {
                let choreo : Booksellers<Colaborative, (Located<Money, Buyer1>, Located<Money, Buyer2>)> = Booksellers{
                    inventory: seller_projector.local(inventory.clone()),
                    title: seller_projector.remote(Buyer1),
                    budgets: (seller_projector.remote(Buyer1), seller_projector.remote(Buyer2)),
                    _marker: PhantomData,
                };
                seller_projector.epp_and_run(choreo)
            }));
        }
        {
            handles.push(thread::spawn(move || {
                let choreo : Booksellers<Colaborative, (Located<Money, Buyer1>, Located<Money, Buyer2>)> = Booksellers{
                    inventory: buyer1_projector.remote(Seller),
                    title: buyer1_projector.local(title).clone(),
                    budgets: (buyer1_projector.local(budget1), buyer1_projector.remote(Buyer2)),
                    _marker: PhantomData,
                };
                buyer1_projector.epp_and_run(choreo)
            }));
        }
        {
            handles.push(thread::spawn(move || {
                let choreo : Booksellers<Colaborative, (Located<Money, Buyer1>, Located<Money, Buyer2>)> = Booksellers{
                    inventory: buyer2_projector.remote(Seller),
                    title: buyer2_projector.remote(Buyer1),
                    budgets: (buyer2_projector.remote(Buyer1), buyer2_projector.local(budget2)),
                    _marker: PhantomData,
                };
                buyer2_projector.epp_and_run(choreo)
            }));
        }
    } else {
        {
            let choreo : Booksellers<Unilateral, Located<Money, Buyer1>> = Booksellers{
                inventory: central_runner.local(inventory.clone()),
                title: central_runner.local(title.clone()),
                budgets: central_runner.local(budget1),
                _marker: PhantomData,
            };
            let central_result = central_runner.run(choreo);
            assert_eq!(central_result, answer);
        }
        {
            handles.push(thread::spawn(move || {
                let choreo : Booksellers<Unilateral, Located<Money, Buyer1>> = Booksellers{
                    inventory: seller_projector.local(inventory.clone()),
                    title: seller_projector.remote(Buyer1),
                    budgets: seller_projector.remote(Buyer1),
                    _marker: PhantomData,
                };
                seller_projector.epp_and_run(choreo)
            }));
        }
        {
            handles.push(thread::spawn(move || {
                let choreo : Booksellers<Unilateral, Located<Money, Buyer1>> = Booksellers{
                    inventory: buyer1_projector.remote(Seller),
                    title: buyer1_projector.local(title).clone(),
                    budgets: buyer1_projector.local(budget1),
                    _marker: PhantomData,
                };
                buyer1_projector.epp_and_run(choreo)
            }));
        }
        {
            handles.push(thread::spawn(move || {
                let choreo : Booksellers<Unilateral, Located<Money, Buyer1>> = Booksellers{
                    inventory: buyer2_projector.remote(Seller),
                    title: buyer2_projector.remote(Buyer1),
                    budgets: buyer2_projector.remote(Buyer1),
                    _marker: PhantomData,
                };
                buyer2_projector.epp_and_run(choreo)
            }));
        }
    }

    for h in handles {
        assert_eq!(h.join().unwrap(), answer);
    }
}

#[test]
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
    let tapl = "TAPL".to_string();
    let hott = "HoTT".to_string();
    run_test(inventory.clone(), tapl.clone(), 100, None, Some(NaiveDate::from_ymd_opt(2023, 8, 3).unwrap()));
    run_test(inventory.clone(), hott.clone(), 25, None, None);
    run_test(inventory.clone(), tapl.clone(), 30, Some(30), Some(NaiveDate::from_ymd_opt(2023, 8, 3).unwrap()));
    run_test(inventory.clone(), hott.clone(), 30, Some(30), None);
}
