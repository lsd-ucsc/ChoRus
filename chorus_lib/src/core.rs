//! Abstract choreography constructs.
//!
//! This module provides core choreography constructs, such as `Choreography`, `Located`, and `Projector`.

use std::marker::PhantomData;

use serde::de::DeserializeOwned;
// re-export so that users can use derive macros without importing serde
pub use serde::{Deserialize, Serialize};

/// Represents a location. It can be derived using `#[derive(ChoreographyLocation)]`.
pub trait ChoreographyLocation: Copy {
    /// Returns the name of the location as a string.
    fn name(&self) -> &'static str;
}

/// Represents a value that can be used in a choreography. ChoRus uses [serde](https://serde.rs/) to serialize and deserialize values.
/// It can be derived using `#[derive(Serialize, Deserialize, Clone)]` as long as all the fields satisfy the `ChoreographicValue` trait.
pub trait ChoreographicValue: Serialize + DeserializeOwned + Clone {} // TODO(shumbo): Is `Clone` really necessary?
impl<T: Serialize + DeserializeOwned + Clone> ChoreographicValue for T {}

/// Represents a value that might *NOT* be located at a location. Values returned by `colocally` must satisfy this trait.
///
/// In most cases, you don't need to implement this trait manually. You can derive it using `#[derive(Superposition)]` as long as all the fields consist of located values.
pub trait Superposition {
    /// Constructs a struct that is *NOT* located at a location.
    fn remote() -> Self;
}

impl Superposition for () {
    fn remote() -> Self {
        ()
    }
}

/// Represents a value located at a location.
///
/// The struct takes two type parameters: `V` and `L1`.
///
/// - `V` is an actual type of the value.
/// - `L1` is the location of the value. It must satisfy the `ChoreographicLocation` trait.
#[derive(PartialEq)]
pub struct Located<V, L1>
where
    L1: ChoreographyLocation,
{
    /// `Some` if it is located at the current location and `None` if it is located at another location.
    value: Option<V>,
    /// The struct is parametrized by the location (`L1`).
    phantom: PhantomData<L1>,
}

impl<V, L1> Located<V, L1>
where
    L1: ChoreographyLocation,
{
    /// Constructs a struct located at the current location with value
    fn local(value: V) -> Self {
        Located {
            value: Some(value),
            phantom: PhantomData,
        }
    }
}

/// If the value implements `Clone`, the located value of the same type also implements `Clone`.
impl<V: Clone, L1> Clone for Located<V, L1>
where
    L1: ChoreographyLocation,
{
    fn clone(&self) -> Self {
        Located {
            value: self.value.clone(),
            phantom: PhantomData,
        }
    }
}

impl<V, L1> Superposition for Located<V, L1>
where
    L1: ChoreographyLocation,
{
    /// Constructs a struct located at another location
    fn remote() -> Self {
        Located {
            value: None,
            phantom: PhantomData,
        }
    }
}

/// Provides a method to work with located values at the current location
pub struct Unwrapper<L1: ChoreographyLocation> {
    phantom: PhantomData<L1>,
}

impl<L1: ChoreographyLocation> Unwrapper<L1> {
    /// Takes the value located at the current location and returns its value
    pub fn unwrap<V>(&self, located: Located<V, L1>) -> V {
        located.value.unwrap()
    }
}

/// Provides choreographic operations.
///
/// The trait provides methods to work with located values. An implementation of the trait is "injected" into
/// a choreography at runtime and provides the actual implementation of the operators.
pub trait ChoreoOp {
    /// Performs a computation at the specified location.
    ///
    /// `locally` performs a computation at a location, which are specified by `location` and `computation`, respectively.
    ///
    /// - `location` is a location where the computation is performed.
    /// - `computation` is a function that takes an `Unwrapper`. Using the `Unwrapper`, the function can access located values at the location.
    ///
    /// The function can return a value of type `V` that satisfies the `ChoreographicValue` trait. The returned value is stored in a `Located` struct at the choreography level.
    fn locally<V, L1: ChoreographyLocation>(
        &self,
        location: L1,
        computation: impl FnOnce(Unwrapper<L1>) -> V,
    ) -> Located<V, L1>;
    /// Performs a communication between two locations.
    ///
    /// `comm` sends `data` from `sender` to `receiver`. The `data` must be a `Located` struct at the `sender` location
    /// and the value type must implement `ChoreographicValue`.
    fn comm<L1: ChoreographyLocation, L2: ChoreographyLocation, V: ChoreographicValue>(
        &self,
        sender: L1,
        receiver: L2,
        data: &Located<V, L1>,
    ) -> Located<V, L2>;

    /// Performs a broadcast from a location to all other locations.
    ///
    /// `broadcast` broadcasts `data` from `sender` to all other locations. The `data` must be a `Located` struct at the `sender` location.
    /// The method returns the non-located value.
    fn broadcast<L1: ChoreographyLocation, V: ChoreographicValue>(
        &self,
        sender: L1,
        data: &Located<V, L1>,
    ) -> V;

    /// Calls a choreography.
    fn call<R, C: Choreography<R>>(&self, choreo: C) -> R;

    /// Calls a choreography on a subset of locations.
    fn colocally<R: Superposition, C: Choreography<R>>(&self, locations: &[&str], choreo: C) -> R;
}

/// Represents a choreography.
///
/// The trait for defining a choreography. It should be implemented for a struct that represents a choreography.
///
/// The type parameter `R` is the return type of the choreography.
///
/// The trait provides a method `run` that takes an implementation of `ChoreoOp` and returns a value of type `R`.
pub trait Choreography<R = ()> {
    /// A method that executes a choreography.
    ///
    /// The method takes an implementation of `ChoreoOp`. Inside the method, you can use the operators provided by `ChoreoOp` to define a choreography.
    ///
    /// The method returns a value of type `R`, which is the return type of the choreography.
    fn run(self, op: &impl ChoreoOp) -> R;
}

/// Provides methods to send and receive messages.
///
/// The trait provides methods to send and receive messages between locations. Implement this trait to define a custom transport.
pub trait Transport {
    /// Returns a list of locations.
    fn locations(&self) -> Vec<String>;
    /// Sends a message from `from` to `to`.
    fn send<V: ChoreographicValue>(&self, from: &str, to: &str, data: &V) -> ();
    /// Receives a message from `from` to `at`.
    fn receive<V: ChoreographicValue>(&self, from: &str, at: &str) -> V;
}

/// Provides a method to perform end-point projection.
pub struct Projector<L1: ChoreographyLocation, T: Transport> {
    target: L1,
    transport: T,
}

impl<L1: ChoreographyLocation, B: Transport> Projector<L1, B> {
    /// Constructs a `Projector` struct.
    ///
    /// - `target` is the projection target of the choreography.
    /// - `transport` is an implementation of `Transport`.
    pub fn new(target: L1, transport: B) -> Self {
        Projector { target, transport }
    }

    /// Constructs a `Located` struct located at the projection target using the actual value.
    ///
    /// Use this method to run a choreography that takes a located value as an input.
    pub fn local<V>(&self, value: V) -> Located<V, L1> {
        Located::local(value)
    }

    /// Constructs a `Located` struct *NOT* located at the projection target.
    ///
    /// Use this method to run a choreography that takes a located value as an input.
    ///
    /// Note that the method panics at runtime if the projection target and the location of the value are the same.
    pub fn remote<V, L2: ChoreographyLocation>(&self, l2: L2) -> Located<V, L2> {
        // NOTE(shumbo): Ideally, this check should be done at the type level.
        if self.target.name() == l2.name() {
            panic!("Cannot create a remote value at the same location");
        }
        Located::remote()
    }

    /// Unwraps a located value at the projection target.
    ///
    /// Use this method to access the located value returned by a choreography.
    pub fn unwrap<V>(&self, located: Located<V, L1>) -> V {
        located.value.unwrap()
    }

    /// Performs end-point projection and runs a choreography.
    pub fn epp_and_run<'a, V, C: Choreography<V>>(&'a self, choreo: C) -> V {
        struct EppOp<'a, B: Transport> {
            target: String,
            transport: &'a B,
            locations: Vec<String>,
        }
        impl<'a, B: Transport> ChoreoOp for EppOp<'a, B> {
            fn locally<V, L1: ChoreographyLocation>(
                &self,
                location: L1,
                computation: impl FnOnce(Unwrapper<L1>) -> V,
            ) -> Located<V, L1> {
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

            fn comm<L1: ChoreographyLocation, L2: ChoreographyLocation, V: ChoreographicValue>(
                &self,
                sender: L1,
                receiver: L2,
                data: &Located<V, L1>,
            ) -> Located<V, L2> {
                if sender.name() == self.target {
                    self.transport
                        .send(sender.name(), receiver.name(), &data.value);
                    Located::remote()
                } else if receiver.name() == self.target {
                    let value = self.transport.receive(sender.name(), receiver.name());
                    Located::local(value)
                } else {
                    Located::remote()
                }
            }

            fn broadcast<L1: ChoreographyLocation, V: ChoreographicValue>(
                &self,
                sender: L1,
                data: &Located<V, L1>,
            ) -> V {
                if sender.name() == self.target {
                    for dest in &self.locations {
                        if self.target != *dest {
                            self.transport.send(&self.target, &dest, &data.value);
                        }
                    }
                    return data.value.clone().unwrap();
                } else {
                    self.transport.receive(sender.name(), &self.target)
                }
            }

            fn call<T, C: Choreography<T>>(&self, choreo: C) -> T {
                choreo.run(self)
            }

            fn colocally<T: Superposition, C: Choreography<T>>(
                &self,
                locs: &[&str],
                choreo: C,
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

/// Provides a method to run a choreography without end-point projection.
pub struct Runner;

impl Runner {
    /// Constructs a runner.
    pub fn new() -> Self {
        Runner
    }

    /// Constructs a located value.
    ///
    /// To execute a choreography with a runner, you must provide located values at all locations
    pub fn local<V, L1: ChoreographyLocation>(&self, value: V) -> Located<V, L1> {
        Located::local(value)
    }

    /// Unwraps a located value
    ///
    /// Runner can unwrap a located value at any location
    pub fn unwrap<V, L1: ChoreographyLocation>(&self, located: Located<V, L1>) -> V {
        located.value.unwrap()
    }

    /// Runs a choreography directly
    pub fn run<'a, V, C: Choreography<V>>(&'a self, choreo: C) -> V {
        struct RunOp;
        impl ChoreoOp for RunOp {
            fn locally<V, L1: ChoreographyLocation>(
                &self,
                _location: L1,
                computation: impl FnOnce(Unwrapper<L1>) -> V,
            ) -> Located<V, L1> {
                let unwrapper = Unwrapper {
                    phantom: PhantomData,
                };
                let value = computation(unwrapper);
                Located::local(value)
            }

            fn comm<L1: ChoreographyLocation, L2: ChoreographyLocation, V: ChoreographicValue>(
                &self,
                _sender: L1,
                _receiver: L2,
                data: &Located<V, L1>,
            ) -> Located<V, L2> {
                let value = data.value.clone().unwrap();
                Located::local(value)
            }

            fn broadcast<L1: ChoreographyLocation, V: ChoreographicValue>(
                &self,
                _sender: L1,
                data: &Located<V, L1>,
            ) -> V {
                data.value.clone().unwrap()
            }

            fn call<R, C: Choreography<R>>(&self, choreo: C) -> R {
                choreo.run(self)
            }

            fn colocally<R: Superposition, C: Choreography<R>>(
                &self,
                _locations: &[&str],
                choreo: C,
            ) -> R {
                choreo.run(self)
            }
        }
        choreo.run(&RunOp)
    }
}

extern crate chorus_derive;
pub use chorus_derive::{ChoreographyLocation, Superposition};
