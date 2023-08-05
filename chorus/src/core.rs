use std::marker::PhantomData;

use serde::de::DeserializeOwned;
// re-export so that users can use derive macros without importing serde
pub use serde::{Deserialize, Serialize};

pub trait ChoreographyLocation: Copy {
    fn name(&self) -> &'static str;
}

pub trait ChoreographicValue: Serialize + DeserializeOwned + Clone {}
impl<T: Serialize + DeserializeOwned + Clone> ChoreographicValue for T {}

pub trait Superposition {
    fn remote() -> Self;
}

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
    fn local(value: T) -> Self {
        Located {
            value: Some(value),
            phantom: PhantomData,
        }
    }
}

impl<T, L1> Superposition for Located<T, L1>
where
    T: ChoreographicValue,
{
    fn remote() -> Self {
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
    fn call<T, C: Choreography<T>>(&self, choreo: &C) -> T;
    fn colocally<T: Superposition, C: Choreography<T>>(&self, locations: &[&str], choreo: &C) -> T;
}

pub trait Choreography<T = ()> {
    fn run(&self, op: &impl ChoreoOp) -> T;
}

pub trait Backend {
    fn send<T: ChoreographicValue>(&self, from: &str, to: &str, data: T) -> ();
    fn broadcast<T: ChoreographicValue>(&self, from: &str, data: T) -> T;
    fn receive<T: ChoreographicValue>(&self, from: &str, at: &str) -> T;
}

pub struct Projector<L1: ChoreographyLocation, B: Backend> {
    target: L1,
    backend: B,
}

impl<L1: ChoreographyLocation, B: Backend> Projector<L1, B> {
    pub fn new(target: L1, backend: B) -> Self {
        Projector { target, backend }
    }

    pub fn local<T: ChoreographicValue>(&self, value: T) -> Located<T, L1> {
        Located::local(value)
    }

    pub fn remote<T: ChoreographicValue, L2: ChoreographyLocation>(
        &self,
        l2: L2,
    ) -> Located<T, L2> {
        // NOTE(shumbo): Ideally, this check should be done at the type level.
        if self.target.name() == l2.name() {
            panic!("Cannot create a remote value at the same location");
        }
        Located::remote()
    }

    pub fn unwrap<T: ChoreographicValue>(&self, located: Located<T, L1>) -> T {
        located.value.unwrap()
    }

    pub fn epp_and_run<'a, T, C: Choreography<T>>(&'a self, choreo: C) -> T {
        struct EppOp<'a, B: Backend> {
            target: &'static str,
            backend: &'a B,
        }
        impl<'a, B: Backend> ChoreoOp for EppOp<'a, B> {
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
                    Located::local(value)
                } else {
                    Located::remote()
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
                    Located::remote()
                } else if receiver.name() == self.target {
                    let value = self.backend.receive(sender.name(), receiver.name());
                    Located::local(value)
                } else {
                    Located::remote()
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

            fn call<T, C: Choreography<T>>(&self, choreo: &C) -> T {
                choreo.run(self)
            }

            fn colocally<T: Superposition, C: Choreography<T>>(
                &self,
                locations: &[&str],
                choreo: &C,
            ) -> T {
                for location in locations {
                    if location == &self.target {
                        return choreo.run(self);
                    }
                }
                T::remote()
            }
        }
        let op: EppOp<'a, B> = EppOp {
            target: self.target.name(),
            backend: &self.backend,
        };
        choreo.run(&op)
    }
}

extern crate chorus_derive;
pub use chorus_derive::{ChoreographyLocation, Superposition};
