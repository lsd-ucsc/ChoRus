use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait ChoreographyLocation {
    fn name(&self) -> &'static str;
}

pub trait ChoreographicValue: Serialize + DeserializeOwned + Clone {}
impl<T: Serialize + DeserializeOwned + Clone> ChoreographicValue for T {}

#[derive(PartialEq)]
pub struct Located<T, L1>
where
    T: ChoreographicValue,
{
    value: Option<T>,
    phantom: PhantomData<L1>,
}

impl<T, L1> Located<T, L1>
where
    T: ChoreographicValue,
{
    fn some(value: T) -> Self {
        Located {
            value: Some(value),
            phantom: PhantomData,
        }
    }
    fn none() -> Self {
        Located {
            value: None,
            phantom: PhantomData,
        }
    }
}

pub struct Unwrapper<L1: ChoreographyLocation> {
    phantom: PhantomData<L1>,
}

impl<L1: ChoreographyLocation> Unwrapper<L1> {
    pub fn unwrap<T: ChoreographicValue>(&self, located: &Located<T, L1>) -> T {
        located.value.clone().unwrap()
    }
}

pub trait ChoreoOp {
    fn locally<T: ChoreographicValue, L1: ChoreographyLocation>(
        &self,
        location: L1,
        computation: impl FnOnce(Unwrapper<L1>) -> T,
    ) -> Located<T, L1>;
    fn comm<L1: ChoreographyLocation, L2: ChoreographyLocation, T: ChoreographicValue>(
        &self,
        sender: L1,
        receiver: L2,
        data: Located<T, L1>,
    ) -> Located<T, L2>;
    fn broadcast<L1: ChoreographyLocation, T: ChoreographicValue>(
        &self,
        sender: L1,
        data: Located<T, L1>,
    ) -> T;
}

pub trait Choreography<T = ()> {
    fn run(&self, op: &impl ChoreoOp) -> T;
}

pub trait Backend {
    fn send<T: ChoreographicValue>(&self, from: &str, to: &str, data: T) -> ();
    fn broadcast<T: ChoreographicValue>(&self, from: &str, data: T) -> T;
    fn receive<T: ChoreographicValue>(&self, from: &str, at: &str) -> T;
}

pub fn epp_and_run<T, C: Choreography<T>, TARGET: ChoreographyLocation, BACKEND: Backend>(
    choreo: C,
    target: TARGET,
    backend: BACKEND,
) -> T {
    struct EppOp<B> {
        target: &'static str,
        backend: B,
    }
    impl<B: Backend> ChoreoOp for EppOp<B> {
        fn locally<T: ChoreographicValue, L1: ChoreographyLocation>(
            &self,
            location: L1,
            computation: impl FnOnce(Unwrapper<L1>) -> T,
        ) -> Located<T, L1> {
            if location.name() == self.target {
                let unwrapper = Unwrapper {
                    phantom: PhantomData,
                };
                let value = computation(unwrapper);
                Located::some(value)
            } else {
                Located::none()
            }
        }

        fn comm<L1: ChoreographyLocation, L2: ChoreographyLocation, T: ChoreographicValue>(
            &self,
            sender: L1,
            receiver: L2,
            data: Located<T, L1>,
        ) -> Located<T, L2> {
            if sender.name() == self.target {
                self.backend
                    .send(sender.name(), receiver.name(), data.value.unwrap());
                Located::none()
            } else if receiver.name() == self.target {
                let value = self.backend.receive(sender.name(), receiver.name());
                Located::some(value)
            } else {
                Located::none()
            }
        }

        fn broadcast<L1: ChoreographyLocation, T: ChoreographicValue>(
            &self,
            sender: L1,
            data: Located<T, L1>,
        ) -> T {
            if sender.name() == self.target {
                self.backend.broadcast(sender.name(), data.value.unwrap())
            } else {
                self.backend.receive(sender.name(), self.target)
            }
        }
    }
    let op: EppOp<BACKEND> = EppOp {
        target: target.name(),
        backend,
    };
    choreo.run(&op)
}

extern crate chorus_derive;
pub use chorus_derive::ChoreographyLocation;
