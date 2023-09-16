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
    fn name() -> &'static str;
}

/// Represents a value that can be used in a choreography. ChoRus uses [serde](https://serde.rs/) to serialize and deserialize values.
/// It can be derived using `#[derive(Serialize, Deserialize)]` as long as all the fields satisfy the `Portable` trait.
pub trait Portable: Serialize + DeserializeOwned {}
impl<T: Serialize + DeserializeOwned> Portable for T {}

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

// --- HList and Helpers ---

/// heterogeneous list
pub trait HList {
    /// returns
    fn to_string_list() -> Vec<&'static str>;
}
/// end of HList
pub struct HNil;
/// An element of HList
pub struct HCons<Head, Tail>(Head, Tail);

impl HList for HNil {
    fn to_string_list() -> Vec<&'static str> {
        Vec::new()
    }
}
impl<Head, Tail> HList for HCons<Head, Tail>
where
    Head: ChoreographyLocation,
    Tail: HList,
{
    fn to_string_list() -> Vec<&'static str> {
        let mut v = Tail::to_string_list();
        v.push(Head::name());
        v
    }
}

// TODO(shumbo): Export the macro under the `core` module

/// Macro to generate hlist
#[macro_export]
macro_rules! hlist {
    () => { $crate::core::HNil };
    ($head:ty $(,)*) => { $crate::core::HCons<$head, $crate::core::HNil> };
    ($head:ty, $($tail:tt)*) => { $crate::core::HCons<$head, hlist!($($tail)*)> };
}

/// Marker
pub struct Here;
/// Marker
pub struct There<Index>(Index);

/// Check membership
pub trait Member<L, Index> {
    /// Return HList of non-member
    type Remainder: HList;
}

impl<Head, Tail> Member<HCons<Head, Tail>, Here> for Head
where
    Tail: HList,
{
    type Remainder = Tail;
}
impl<Head, Head1, Tail, X, TailIndex> Member<HCons<Head, HCons<Head1, Tail>>, There<TailIndex>>
    for X
where
    Head: ChoreographyLocation,
    X: Member<HCons<Head1, Tail>, TailIndex>,
{
    type Remainder = HCons<Head, X::Remainder>;
}

/// Check subset
pub trait Subset<L: HList, Index> {}

// Base case: HNil is a subset of any set
impl<L: HList> Subset<L, Here> for HNil {}

// Recursive case
impl<L: HList, Head, Tail: HList, IHead, ITail> Subset<L, HCons<IHead, ITail>> for HCons<Head, Tail>
where
    Head: Member<L, IHead>,
    Tail: Subset<L, ITail>,
{
}

/// Equal
 pub trait Equal<L: HList, Index> {}

 // Base case: HNil is equal to HNil
 impl Equal<HNil, Here> for HNil {}

 // Recursive case: Head::Tail is equal to L if
 // 1. Head is a member of L
 // 2. Tail is equal to the remainder of L
 impl<L: HList, Head, Tail, Index1, Index2> Equal<L, HCons<Index1, Index2>> for HCons<Head, Tail>
 where
     Head: Member<L, Index1>,
     Tail: Equal<Head::Remainder, Index2>,
 {
 }

/// Provides a method to work with located values at the current location
pub struct Unwrapper<L1: ChoreographyLocation> {
    phantom: PhantomData<L1>,
}

impl<L1: ChoreographyLocation> Unwrapper<L1> {
    /// Takes a reference to the located value at the current location and returns its reference
    pub fn unwrap<'a, V>(&self, located: &'a Located<V, L1>) -> &'a V {
        located.value.as_ref().unwrap()
    }
}

/// Provides choreographic operations.
///
/// The trait provides methods to work with located values. An implementation of the trait is "injected" into
/// a choreography at runtime and provides the actual implementation of the operators.
pub trait ChoreoOp<L: HList> {
    /// Performs a computation at the specified location.
    ///
    /// `locally` performs a computation at a location, which are specified by `location` and `computation`, respectively.
    ///
    /// - `location` is a location where the computation is performed.
    /// - `computation` is a function that takes an `Unwrapper`. Using the `Unwrapper`, the function can access located values at the location.
    ///
    /// The `computation` can return a value of type `V` and the value will be stored in a `Located` struct at the choreography level.
    fn locally<V, L1: ChoreographyLocation, Index>(
        &self,
        location: L1,
        computation: impl Fn(Unwrapper<L1>) -> V,
    ) -> Located<V, L1>
    where
        L1: Member<L, Index>;
    /// Performs a communication between two locations.
    ///
    /// `comm` sends `data` from `sender` to `receiver`. The `data` must be a `Located` struct at the `sender` location
    /// and the value type must implement `Portable`.
    fn comm<L1: ChoreographyLocation, L2: ChoreographyLocation, V: Portable, Index1, Index2>(
        &self,
        sender: L1,
        receiver: L2,
        data: &Located<V, L1>,
    ) -> Located<V, L2>
    where
        L1: Member<L, Index1>,
        L2: Member<L, Index2>;

    /// Performs a broadcast from a location to all other locations.
    ///
    /// `broadcast` broadcasts `data` from `sender` to all other locations. The `data` must be a `Located` struct at the `sender` location.
    /// The method returns the non-located value.
    fn broadcast<L1: ChoreographyLocation, V: Portable, Index>(
        &self,
        sender: L1,
        data: Located<V, L1>,
    ) -> V
    where
        L1: Member<L, Index>;

    /// Calls a choreography.
    fn call<R, M, Index, C: Choreography<R, L = M>>(&self, choreo: C) -> R
    where
        M: HList + Subset<L, Index>;

    /// Calls a choreography on a subset of locations.
    fn colocally<R: Superposition, S: HList, C: Choreography<R, L = S>, Index>(
        &self,
        choreo: C,
    ) -> R
    where
        S: Subset<L, Index>;
}

/// Represents a choreography.
///
/// The trait for defining a choreography. It should be implemented for a struct that represents a choreography.
///
/// The type parameter `R` is the return type of the choreography.
///
/// The trait provides a method `run` that takes an implementation of `ChoreoOp` and returns a value of type `R`.
pub trait Choreography<R = ()> {
    /// Locations
    type L: HList;
    /// A method that executes a choreography.
    ///
    /// The method takes an implementation of `ChoreoOp`. Inside the method, you can use the operators provided by `ChoreoOp` to define a choreography.
    ///
    /// The method returns a value of type `R`, which is the return type of the choreography.
    fn run(self, op: &impl ChoreoOp<Self::L>) -> R;
}

/// Provides methods to send and receive messages.
///
/// The trait provides methods to send and receive messages between locations. Implement this trait to define a custom transport.
pub trait Transport {
    /// Returns a list of locations.
    fn locations(&self) -> Vec<String>;
    /// Sends a message from `from` to `to`.
    fn send<V: Portable>(&self, from: &str, to: &str, data: &V) -> ();
    /// Receives a message from `from` to `at`.
    fn receive<V: Portable>(&self, from: &str, at: &str) -> V;
}

/// Provides a method to perform end-point projection.
pub struct Projector<AL: HList, L1: ChoreographyLocation, T: Transport, Index> where 
    L1: Member<AL, Index>{
    target: PhantomData<L1>,
    transport: T,
    available_locations: PhantomData<AL>,
    index: PhantomData<Index>,
}

/// Provides a wrapper struct for users so that they can only specify AL; since it can't be inferred.
pub struct ProjectorForAL<AL: HList>(PhantomData<AL>);

impl<AL: HList> ProjectorForAL<AL> {
    /// Constructs a `Projector` struct.
    ///
    /// - `target` is the projection target of the choreography.
    /// - `transport` is an implementation of `Transport`.
    pub fn new<L1: ChoreographyLocation, B: Transport, Index>(
        target: L1,
        transport: B,
    ) -> Projector<AL, L1, B, Index>
    where
        L1: Member<AL, Index>,
    {
        Projector::new(target, transport)
    }
}

impl<AL: HList, L1: ChoreographyLocation, B: Transport, Index> Projector<AL, L1, B, Index> 
    where L1: Member<AL, Index> 
{
    /// Constructs a `Projector` struct.
    ///
    /// - `target` is the projection target of the choreography.
    /// - `transport` is an implementation of `Transport`.
    pub fn new(_target: L1, transport: B) -> Self
    {
        Projector {
            target: PhantomData,
            transport,
            available_locations: PhantomData,
            index: PhantomData,
        }
    }

    /// Constructs a `Located` struct located at the projection target using the actual value.
    ///
    /// Use this method to run a choreography that takes a located value as an input.
    pub fn local<V>(&self, value: V) -> Located<V, L1>
    {
        Located::local(value)
    }

    /// Constructs a `Located` struct *NOT* located at the projection target.
    ///
    /// Use this method to run a choreography that takes a located value as an input.
    ///
    /// Note that the method panics at runtime if the projection target and the location of the value are the same.
    pub fn remote<V, L2: ChoreographyLocation, Index2>(&self, _l2: L2) -> Located<V, L2>
    where
        L2: Member<<L1 as Member<AL, Index>>::Remainder, Index2>,
    {
        Located::remote()
    }

    /// Unwraps a located value at the projection target.
    ///
    /// Use this method to access the located value returned by a choreography.
    pub fn unwrap<V>(&self, located: Located<V, L1>) -> V {
        located.value.unwrap()
    }

    /// Performs end-point projection and runs a choreography.
    pub fn epp_and_run<'a, V, L: HList, C: Choreography<V, L = L>, IndexSet>(&'a self, choreo: C) -> V
    where
        L: Equal<AL, IndexSet>,
    {
        struct EppOp<'a, L: HList, L1: ChoreographyLocation, B: Transport> {
            target: PhantomData<L1>,
            transport: &'a B,
            locations: Vec<String>,
            marker: PhantomData<L>,
        }
        impl<'a, L: HList, T: ChoreographyLocation, B: Transport> ChoreoOp<L> for EppOp<'a, L, T, B> {
            fn locally<V, L1: ChoreographyLocation, Index>(
                &self,
                _location: L1,
                computation: impl Fn(Unwrapper<L1>) -> V,
            ) -> Located<V, L1> {
                if L1::name() == T::name() {
                    let unwrapper = Unwrapper {
                        phantom: PhantomData,
                    };
                    let value = computation(unwrapper);
                    Located::local(value)
                } else {
                    Located::remote()
                }
            }

            fn comm<
                L1: ChoreographyLocation,
                L2: ChoreographyLocation,
                V: Portable,
                Index1,
                Index2,
            >(
                &self,
                _sender: L1,
                _receiver: L2,
                data: &Located<V, L1>,
            ) -> Located<V, L2> {
                if L1::name() == T::name() {
                    self.transport
                        .send(L1::name(), L2::name(), data.value.as_ref().unwrap());
                    Located::remote()
                } else if L2::name() == T::name() {
                    let value = self.transport.receive(L1::name(), L2::name());
                    Located::local(value)
                } else {
                    Located::remote()
                }
            }

            fn broadcast<L1: ChoreographyLocation, V: Portable, Index>(
                &self,
                _sender: L1,
                data: Located<V, L1>,
            ) -> V {
                if L1::name() == T::name() {
                    for dest in &self.locations {
                        if T::name() != *dest {
                            self.transport.send(&T::name(), &dest, &data.value);
                        }
                    }
                    return data.value.unwrap();
                } else {
                    self.transport.receive(L1::name(), &T::name())
                }
            }

            fn call<R, M, Index, C: Choreography<R, L = M>>(&self, choreo: C) -> R
            where
                M: HList + Subset<L, Index>,
            {
                let op: EppOp<'a, M, T, B> = EppOp {
                    target: PhantomData::<T>,
                    transport: &self.transport,
                    locations: self.transport.locations(),
                    marker: PhantomData::<M>,
                };
                choreo.run(&op)
            }

            fn colocally<R: Superposition, S: HList, C: Choreography<R, L = S>, Index>(
                &self,
                choreo: C,
            ) -> R {
                let locs_vec =
                    Vec::from_iter(S::to_string_list().into_iter().map(|s| s.to_string()));

                for location in &locs_vec {
                    if *location == T::name().to_string() {
                        let op = EppOp {
                            target: PhantomData::<T>,
                            transport: self.transport,
                            locations: locs_vec.clone(),
                            marker: PhantomData::<S>,
                        };
                        return choreo.run(&op);
                    }
                }
                R::remote()
            }
        }
        let op: EppOp<'a, L, L1, B> = EppOp {
            target: PhantomData::<L1>,
            transport: &self.transport,
            locations: self.transport.locations(),
            marker: PhantomData::<L>,
        };
        choreo.run(&op)
    }
}

/// Provides a method to run a choreography without end-point projection.
pub struct Runner<L: HList> {
    marker: PhantomData<L>,
}

impl<L: HList> Runner<L> {
    /// Constructs a runner.
    pub fn new() -> Self {
        Runner {
            marker: PhantomData::<L>,
        }
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
    pub fn run<'a, V, C: Choreography<V, L = L>>(&'a self, choreo: C) -> V {
        struct RunOp<L>(PhantomData<L>);
        impl<L: HList> ChoreoOp<L> for RunOp<L> {
            fn locally<V, L1: ChoreographyLocation, Index>(
                &self,
                _location: L1,
                computation: impl Fn(Unwrapper<L1>) -> V,
            ) -> Located<V, L1> {
                let unwrapper = Unwrapper {
                    phantom: PhantomData,
                };
                let value = computation(unwrapper);
                Located::local(value)
            }

            fn comm<
                L1: ChoreographyLocation,
                L2: ChoreographyLocation,
                V: Portable,
                Index1,
                Index2,
            >(
                &self,
                _sender: L1,
                _receiver: L2,
                data: &Located<V, L1>,
            ) -> Located<V, L2> {
                // clone the value by encoding and decoding it. Requiring `Clone` could improve the performance but is not necessary.
                // Also, this is closer to what happens to the value with end-point projection.
                let s = serde_json::to_string(data.value.as_ref().unwrap()).unwrap();
                Located::local(serde_json::from_str(s.as_str()).unwrap())
            }

            fn broadcast<L1: ChoreographyLocation, V: Portable, Index>(
                &self,
                _sender: L1,
                data: Located<V, L1>,
            ) -> V {
                data.value.unwrap()
            }

            fn call<R, M, Index, C: Choreography<R, L = M>>(&self, choreo: C) -> R
            where
                M: HList + Subset<L, Index>,
            {
                let op: RunOp<M> = RunOp(PhantomData);
                choreo.run(&op)
            }

            fn colocally<R: Superposition, S: HList, C: Choreography<R, L = S>, Index>(
                &self,
                choreo: C,
            ) -> R {
                let op = RunOp::<S>(PhantomData);
                choreo.run(&op)
            }
        }
        let op: RunOp<L> = RunOp(PhantomData);
        choreo.run(&op)
    }
}

extern crate chorus_derive;
pub use chorus_derive::{ChoreographyLocation, Superposition};
