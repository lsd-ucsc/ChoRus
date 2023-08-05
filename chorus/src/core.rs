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
pub struct Located<V, L1>
where
    V: ChoreographicValue,
{
    value: Option<V>,
    phantom: PhantomData<L1>,
}

impl<V, L1> Located<V, L1>
where
    V: ChoreographicValue,
{
    fn local(value: V) -> Self {
        Located {
            value: Some(value),
            phantom: PhantomData,
        }
    }
}

impl<V, L1> Superposition for Located<V, L1>
where
    V: ChoreographicValue,
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
    pub fn unwrap<V: ChoreographicValue>(&self, located: &Located<V, L1>) -> V {
        located.value.clone().unwrap()
    }
}

pub trait ChoreoOp {
    fn locally<V: ChoreographicValue, L1: ChoreographyLocation>(
        &self,
        location: L1,
        computation: impl FnOnce(Unwrapper<L1>) -> V,
    ) -> Located<V, L1>;
    fn comm<L1: ChoreographyLocation, L2: ChoreographyLocation, V: ChoreographicValue>(
        &self,
        sender: L1,
        receiver: L2,
        data: &Located<V, L1>,
    ) -> Located<V, L2>;
    fn broadcast<L1: ChoreographyLocation, V: ChoreographicValue>(
        &self,
        sender: L1,
        data: &Located<V, L1>,
    ) -> V;
    fn call<R, C: Choreography<R>>(&self, choreo: &C) -> R;
    fn colocally<R: Superposition, C: Choreography<R>>(&self, locations: &[&str], choreo: &C) -> R;
}

pub trait Choreography<R = ()> {
    fn run(&self, op: &impl ChoreoOp) -> R;
}

pub trait Transport {
    fn locations(&self) -> Vec<String>;
    fn send<V: ChoreographicValue>(&self, from: &str, to: &str, data: V) -> ();
    fn receive<V: ChoreographicValue>(&self, from: &str, at: &str) -> V;
}

pub struct Projector<L1: ChoreographyLocation, T: Transport> {
    target: L1,
    transport: T,
}

impl<L1: ChoreographyLocation, B: Transport> Projector<L1, B> {
    pub fn new(target: L1, transport: B) -> Self {
        Projector { target, transport }
    }

    pub fn local<V: ChoreographicValue>(&self, value: V) -> Located<V, L1> {
        Located::local(value)
    }

    pub fn remote<V: ChoreographicValue, L2: ChoreographyLocation>(
        &self,
        l2: L2,
    ) -> Located<V, L2> {
        // NOTE(shumbo): Ideally, this check should be done at the type level.
        if self.target.name() == l2.name() {
            panic!("Cannot create a remote value at the same location");
        }
        Located::remote()
    }

    pub fn unwrap<V: ChoreographicValue>(&self, located: Located<V, L1>) -> V {
        located.value.unwrap()
    }

    pub fn epp_and_run<'a, V, C: Choreography<V>>(&'a self, choreo: C) -> V {
        struct EppOp<'a, B: Transport> {
            target: String,
            transport: &'a B,
            locations: Vec<String>,
        }
        impl<'a, B: Transport> ChoreoOp for EppOp<'a, B> {
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
                data: &Located<T, L1>,
            ) -> Located<T, L2> {
                if sender.name() == self.target {
                    self.transport.send(
                        sender.name(),
                        receiver.name(),
                        data.value.clone().unwrap(),
                    );
                    Located::remote()
                } else if receiver.name() == self.target {
                    let value = self.transport.receive(sender.name(), receiver.name());
                    Located::local(value)
                } else {
                    Located::remote()
                }
            }

            fn broadcast<L1: ChoreographyLocation, T: ChoreographicValue>(
                &self,
                sender: L1,
                data: &Located<T, L1>,
            ) -> T {
                if sender.name() == self.target {
                    for dest in &self.locations {
                        if self.target != *dest {
                            self.transport
                                .send(&self.target, &dest, data.value.clone().unwrap());
                        }
                    }
                    return data.value.clone().unwrap();
                } else {
                    self.transport.receive(sender.name(), &self.target)
                }
            }

            fn call<T, C: Choreography<T>>(&self, choreo: &C) -> T {
                choreo.run(self)
            }

            fn colocally<T: Superposition, C: Choreography<T>>(
                &self,
                locs: &[&str],
                choreo: &C,
            ) -> T {
                let locs_vec = Vec::from_iter(locs.into_iter().map(|s| s.to_string()));
                for location in &locs_vec {
                    let op = EppOp {
                        target: location.clone(),
                        transport: self.transport,
                        locations: locs_vec.clone(),
                    };
                    if *location == self.target.to_string() {
                        return choreo.run(&op);
                    }
                }
                T::remote()
            }
        }
        let op: EppOp<'a, B> = EppOp {
            target: self.target.name().to_string(),
            transport: &self.transport,
            locations: self.transport.locations(),
        };
        choreo.run(&op)
    }
}

extern crate chorus_derive;
pub use chorus_derive::{ChoreographyLocation, Superposition};
