extern crate chorus_lib;

use std::collections::HashMap;
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

type Decider = fn(Located<Option<Money>, Buyer1>) -> Box<dyn Choreography<Located<bool, Buyer1>, L = LocationSet!(Buyer1, Buyer2)>>;

fn unilateral(budget: Located<Money, Buyer1>) -> Decider {
    |price| {
        struct Choreo{
            price: Located<Money, Buyer1>,
            budget: Located<Money, Buyer1>,
        }
        impl Choreography<Located<bool, Buyer1>> for Choreo {
            type L = LocationSet!(Buyer1, Buyer2);
            fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<bool, Buyer1> {
                op.locally(Buyer1, |un| {
                    un.unwrap(&self.price) <= un.unwrap(&self.budget)
                })
            }
        }
        Box::new(Choreo{price, budget})
    }
}

//fn collaborative(budget1: Located<Money, Buyer1>, budget2: Located<Money, Buyer2>) -> Decider {
//    |price| {
//    impl Choreography<Located<bool, Buyer1>> for TwoBuyerDecider {
 //   type L = LocationSet!(Buyer1, Buyer2);
  //  fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<bool, Buyer1> {
   //     let remaining = op.locally(Buyer1, |un| {
    //        const BUYER1_BUDGET: Money = 100;
     //       return un.unwrap(&self.price) - BUYER1_BUDGET;
      //  });
       // let remaining = op.comm(Buyer1, Buyer2, &remaining);
        //let decision = op.locally(Buyer2, |un| {
 //           const BUYER2_BUDGET: Money = 200;
  //          return *un.unwrap(&remaining) < BUYER2_BUDGET;
   //     });
    //    op.comm(Buyer2, Buyer1, &decision)
    //}
//}
//    }
//}


struct BooksellerChoreography{
    decider: Decider,
    inventory: Located<Inventory, Seller>,
    title: Located<Title, Buyer1>,
}

impl Choreography<Option<NaiveDate>> for BooksellerChoreography
{
    type L = LocationSet!(Buyer1, Buyer2, Seller);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Option<NaiveDate> {
        let title_at_seller = op.comm(Buyer1, Seller, &self.title);
        let price_at_seller = op.locally(Seller, |un| {
            let inventory = un.unwrap(&self.inventory);
            let title = un.unwrap(&title_at_seller);
            match inventory.get(title) {
                Some((price, _)) => Some(price),
                None => None,
            }
        });
        let price_at_buyer1 = op.comm(Seller, Buyer1, &price_at_seller);
        let decision_at_buyer1 = op.enclave(&self.decider(price_at_buyer1));

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
                        match inventory.get(title) {
                            Some((_, delivery_date)) => Some(delivery_date),
                            None => None,
                        }
                    });
                    let delivery_date_at_buyer1 = op.comm(Seller, Buyer1, &delivery_date_at_seller);
                    return delivery_date_at_buyer1;
                } else {
                    return op.locally(Buyer1, |_| None);
                }
            }
        }

        return op.enclave(GetDeliveryDateChoreography {
            inventory: self.inventory.clone(),
            title_at_seller: title_at_seller.clone(),
            decision_at_buyer1,
        });
    }
}

/*fn locate<A, P: ChoreographyLocation>(a: A, p: P) -> Located<A, P> {
    struct Dummy<A1, P1: ChoreographyLocation> {
        a: RC<A1>,
        p: P1
    }
    impl<A2, P2> Choreography<Located<A2, P2>> for Dummy<A2, P2>
    where P2: ChoreographyLocation {
        type L = LocationSet!(P2);
        fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<A2, P2> {
            op.locally(self.p, |_| self.a)
        }
    }
    Runner::new().run(Dummy{a, p});
}*/

fn run_test(inventory: Inventory, title: Title, budget1: Money, budget2: Option<Money>, answer: Option<NaiveDate>) {
    let central_runner = Runner::new();
    let decider: Decider = match budget2 {
        Some(b2) => panic!(),
        None => unilateral(central_runner.local(budget1)),
    };

    let central_result = central_runner.run(BooksellerChoreography{
        decider,
        inventory: central_runner.local(inventory),
        title: central_runner.local(title),
    });
    assert_eq!(central_result, answer);

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

    } else {
        {
            let seller_projector = seller_projector.clone();
            let inventory = inventory.clone();
            handles.push(thread::spawn(move || {
                seller_projector.epp_and_run(BooksellerChoreography {
                    decider,
                    inventory: seller_projector.local(inventory),
                    title: seller_projector.remote(Buyer1),
                });
            }));
        }
        {
            let buyer1_projector = buyer1_projector.clone();
            handles.push(thread::spawn(move || {
                buyer1_projector.epp_and_run(BooksellerChoreography {
                    decider,
                    inventory: buyer1_projector.remote(Seller),
                    title: buyer1_projector.local("HoTT".to_string()),
                });
            }));
        }
        {
            let buyer2_projector = buyer2_projector.clone();
            handles.push(thread::spawn(move || {
                buyer2_projector.epp_and_run(BooksellerChoreography {
                    decider,
                    inventory: buyer2_projector.remote(Seller),
                    title: buyer2_projector.remote(Buyer1),
                });
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
    let title = "TAPL".to_string();
    let budget = 300;
    run_test(inventory, title, budget, None, Some(NaiveDate::from_ymd_opt(2023, 8, 3).unwrap()))
}
