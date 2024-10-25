//! Abstract choreography constructs.
//!
//! This module provides core choreography constructs, such as `Choreography`, `Located`, and `Projector`.

use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use serde::de::DeserializeOwned;
// re-export so that users can use derive macros without importing serde
#[doc(no_inline)]
pub use serde::{Deserialize, Serialize};

/// Represents a location.
///
/// It can be derived using `#[derive(ChoreographyLocation)]`.
///
/// ```
/// # use chorus_lib::core::ChoreographyLocation;
/// #
/// #[derive(ChoreographyLocation)]
/// struct Alice;
/// ```
pub trait ChoreographyLocation: Copy {
    /// Constructs a location.
    fn new() -> Self;
    /// Returns the name of the location as a string.
    fn name() -> &'static str;
}

/// Represents a value that can be used in a choreography.
///
/// ChoRus uses [serde](https://serde.rs/) to serialize and deserialize values.
///
/// It can be derived using `#[derive(Serialize, Deserialize)]` as long as all the fields satisfy the `Portable` trait.
pub trait Portable: Serialize + DeserializeOwned {}
impl<T: Serialize + DeserializeOwned> Portable for T {}

/// Represents a value that might *NOT* be located at a location. Values returned by `enclave` must satisfy this trait.
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
pub type Located<V, L1> = MultiplyLocated<V, LocationSet!(L1)>;

/// Represents a value located at multiple locations.
pub struct MultiplyLocated<V, L>
where
    L: LocationSet,
{
    value: Option<V>,
    phantom: PhantomData<L>,
}

impl<V, L> MultiplyLocated<V, L>
where
    L: LocationSet,
{
    /// Constructs a struct located at the current location with value
    pub fn local(value: V) -> Self {
        MultiplyLocated {
            value: Some(value),
            phantom: PhantomData,
        }
    }
}

impl<V, LS1, LS2> MultiplyLocated<MultiplyLocated<V, LS1>, LS2>
where
    LS1: LocationSet,
    LS2: LocationSet,
{
    /// Flattens a located value located at multiple locations.
    pub fn flatten<LS1SubsetLS2>(self) -> MultiplyLocated<V, LS1>
    where
        LS1: Subset<LS2, LS1SubsetLS2>,
    {
        let value = self.value.map(|x| x.value).flatten();
        MultiplyLocated {
            value,
            phantom: PhantomData,
        }
    }
}

impl<V, L> Clone for MultiplyLocated<V, L>
where
    V: Clone,
    L: LocationSet,
{
    fn clone(&self) -> Self {
        MultiplyLocated {
            value: self.value.clone(),
            phantom: PhantomData,
        }
    }
}

impl<V, L> Superposition for MultiplyLocated<V, L>
where
    L: LocationSet,
{
    /// Constructs a struct located at another location
    fn remote() -> Self {
        MultiplyLocated {
            value: None,
            phantom: PhantomData,
        }
    }
}

/// Represents a mapping from location names to values
pub struct Quire<V, L>
where
    L: LocationSet,
{
    value: HashMap<String, V>,
    phantom: PhantomData<L>,
}

impl<V, L> Debug for Quire<V, L>
where
    L: LocationSet,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.value.iter()).finish()
    }
}

impl<V> Quire<V, HNil> {
    /// Constructs a struct located at the current location with value
    pub fn new() -> Self {
        Quire {
            value: HashMap::new(),
            phantom: PhantomData,
        }
    }
}

impl<V, L> Quire<V, L>
where
    L: LocationSet,
{
    /// Add a value located at a location
    pub fn add<L1: ChoreographyLocation>(self, _location: L1, value: V) -> Quire<V, HCons<L1, L>> {
        let mut map = self.value;
        map.insert(L1::name().to_string(), value);
        Quire {
            value: map,
            phantom: PhantomData,
        }
    }
    /// Get as a hash map
    pub fn get_map(self) -> HashMap<String, V> {
        self.value
    }
}

/// Represents possibly different values located at multiple locations
#[derive(Debug)]
pub struct Faceted<V, L>
where
    L: LocationSet,
{
    value: HashMap<String, V>,
    phantom: PhantomData<L>,
}

// --- HList and Helpers ---

/// xx
pub trait LocationSetFolder<B> {
    /// x
    type L: LocationSet;
    /// looping over
    type QS: LocationSet;
    /// x
    fn f<Q: ChoreographyLocation, QSSubsetL, QMemberL, QMemberQS>(&self, acc: B, curr: Q) -> B
    where
        Self::QS: Subset<Self::L, QSSubsetL>,
        Q: Member<Self::L, QMemberL>,
        Q: Member<Self::QS, QMemberQS>;
}

/// heterogeneous list
#[doc(hidden)]
pub trait LocationSet: Sized {
    fn new() -> Self;
    /// returns
    fn to_string_list() -> Vec<&'static str>;
}

/// end of HList
#[doc(hidden)]
#[derive(Debug)]
pub struct HNil;

/// An element of HList
#[doc(hidden)]
#[derive(Debug)]
pub struct HCons<Head, Tail>(Head, Tail);

/// x
pub trait LocationSetFoldable<L: LocationSet, QS: LocationSet, Index> {
    /// x
    fn foldr<B, F: LocationSetFolder<B, L = L, QS = QS>>(f: F, acc: B) -> B;
}

impl<L: LocationSet, QS: LocationSet> LocationSetFoldable<L, QS, Here> for HNil {
    fn foldr<B, F: LocationSetFolder<B, L = L>>(_f: F, acc: B) -> B {
        acc
    }
}

impl<
        L: LocationSet,
        QS: LocationSet,
        Head: ChoreographyLocation,
        Tail,
        QSSubsetL,
        HeadMemberL,
        HeadMemberQS,
        ITail,
    > LocationSetFoldable<L, QS, (QSSubsetL, HeadMemberL, HeadMemberQS, ITail)>
    for HCons<Head, Tail>
where
    QS: Subset<L, QSSubsetL>,
    Head: Member<L, HeadMemberL>,
    Head: Member<QS, HeadMemberQS>,
    Tail: LocationSetFoldable<L, QS, ITail>,
{
    fn foldr<B, F: LocationSetFolder<B, L = L, QS = QS>>(f: F, acc: B) -> B {
        let x = f.f(acc, Head::new());
        Tail::foldr(f, x)
    }
}

impl LocationSet for HNil {
    fn new() -> Self {
        HNil
    }
    fn to_string_list() -> Vec<&'static str> {
        Vec::new()
    }
}
impl<Head, Tail> LocationSet for HCons<Head, Tail>
where
    Head: ChoreographyLocation,
    Tail: LocationSet,
{
    fn new() -> Self {
        HCons(Head::new(), Tail::new())
    }
    fn to_string_list() -> Vec<&'static str> {
        let mut v = Tail::to_string_list();
        v.push(Head::name());
        v
    }
}

#[derive(ChoreographyLocation)]
struct Alice;
#[derive(ChoreographyLocation)]
struct Bob;
#[derive(ChoreographyLocation)]
struct Carol;

// To export `LocationSet` under the `core` module, we define an internal macro and export it.
// This is because Rust does not allow us to export a macro from a module without re-exporting it.
// `__ChoRus_Internal_LocationSet` is the internal macro and it is configured not to be visible in the documentation.

/// Macro to define a set of locations that a choreography is defined on.
///
/// ```
/// # use chorus_lib::core::{ChoreographyLocation, LocationSet};
/// #
/// # #[derive(ChoreographyLocation)]
/// # struct Alice;
/// # #[derive(ChoreographyLocation)]
/// # struct Bob;
/// # #[derive(ChoreographyLocation)]
/// # struct Carol;
/// #
/// type L = LocationSet!(Alice, Bob, Carol);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __ChoRus_Internal_LocationSet {
    () => { $crate::core::HNil };
    ($head:ty $(,)*) => { $crate::core::HCons<$head, $crate::core::HNil> };
    ($head:ty, $($tail:tt)*) => { $crate::core::HCons<$head, $crate::core::LocationSet!($($tail)*)> };
}

#[doc(inline)]
pub use __ChoRus_Internal_LocationSet as LocationSet;

/// Marker
#[doc(hidden)]
pub struct Here;
#[doc(hidden)]
pub struct Here2;
/// Marker
#[doc(hidden)]
pub struct There<Index>(Index);

/// Check if a location is a member of a location set
///
/// The trait is used to check if a location is a member of a location set.
///
/// It takes two type parameters `L` and `Index`. `L` is a location set and `Index` is some type that is inferred by the compiler.
/// If a location `L1` is in `L`, then there exists a type `Index` such that `L1` implements `Member<L, Index>`.
pub trait Member<L, Index> {
    /// A location set that is the remainder of `L` after removing the member.
    type Remainder: LocationSet;
}

impl<Head, Tail> Member<HCons<Head, Tail>, Here> for Head
where
    Tail: LocationSet,
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

/// Check if a location set is a subset of another location set
///
/// The trait is used to check if a location set is a subset of another location set.
///
/// It takes two type parameters `L` and `Index`. `L` is a location set and `Index` is some type that is inferred by the compiler.
/// If a location set `M` is a subset of `L`, then there exists a type `Index` such that `M` implements `Subset<L, Index>`.
pub trait Subset<S, Index> {}

// Base case: `HNil` is a subset of any collection
impl<S> Subset<S, Here> for HNil {}

// Recursive case: `Head` is a `Member<S, Index1>` and `Tail` is a `Subset<S, Index2>`
impl<Head, Tail, S, Index1, Index2> Subset<S, (Index1, Index2)> for HCons<Head, Tail>
where
    Head: Member<S, Index1>,
    Tail: Subset<S, Index2>,
{
}

/// Provides a method to work with located values at the current location
pub struct Unwrapper<L1: ChoreographyLocation> {
    phantom: PhantomData<L1>,
}

impl<L1: ChoreographyLocation> Unwrapper<L1> {
    /// TODO: documentation
    pub fn unwrap<'a, V, S: LocationSet, Index>(&self, mlv: &'a MultiplyLocated<V, S>) -> &'a V
    where
        L1: Member<S, Index>,
    {
        mlv.value.as_ref().unwrap()
    }
    /// TODO: documentation
    pub fn unwrap3<'a, V, S: LocationSet, Index>(&self, faceted: &'a Faceted<V, S>) -> &'a V
    where
        L1: Member<S, Index>,
    {
        faceted.value.get(L1::name()).unwrap()
    }
}

/// Provides choreographic operations.
///
/// The trait provides methods to work with located values. An implementation of the trait is "injected" into
/// a choreography at runtime and provides the actual implementation of the operators.
pub trait ChoreoOp<ChoreoLS: LocationSet> {
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
    ) -> MultiplyLocated<V, LocationSet!(L1)>
    where
        L1: Member<ChoreoLS, Index>;
    /// Performs a communication between two locations.
    ///
    /// `comm` sends `data` from `sender` to `receiver`. The `data` must be a `Located` struct at the `sender` location
    /// and the value type must implement `Portable`.
    fn comm<
        L: LocationSet,
        Sender: ChoreographyLocation,
        Receiver: ChoreographyLocation,
        V: Portable,
        Index1,
        Index2,
        Index3,
    >(
        &self,
        sender: Sender,
        receiver: Receiver,
        data: &MultiplyLocated<V, L>,
    ) -> MultiplyLocated<V, LocationSet!(Receiver)>
    where
        L: Subset<ChoreoLS, Index1>,
        Sender: Member<ChoreoLS, Index2>,
        Receiver: Member<ChoreoLS, Index3>;

    /// Performs a broadcast from a location to all other locations.
    ///
    /// `broadcast` broadcasts `data` from `sender` to all other locations. The `data` must be a `Located` struct at the `sender` location.
    /// The method returns the non-located value.
    fn broadcast<L: LocationSet, Sender: ChoreographyLocation, V: Portable, Index1, Index2>(
        &self,
        sender: Sender,
        data: MultiplyLocated<V, L>,
    ) -> V
    where
        L: Subset<ChoreoLS, Index1>,
        Sender: Member<ChoreoLS, Index2>;

    /// Performs a multicast from a location to a set of locations.
    ///
    /// Use `<LocationSet!(L1, L2, ...)>::new()` to create a value of location set.
    fn multicast<Sender: ChoreographyLocation, V: Portable, D: LocationSet, Index1, Index2>(
        &self,
        src: Sender,
        destination: D,
        data: &MultiplyLocated<V, LocationSet!(Sender)>,
    ) -> MultiplyLocated<V, D>
    where
        Sender: Member<ChoreoLS, Index1>,
        D: Subset<ChoreoLS, Index2>;

    /// TODO: documentation
    fn naked<S: LocationSet, V, Index>(&self, data: MultiplyLocated<V, S>) -> V
    where
        ChoreoLS: Subset<S, Index>;

    /// TODO: documentation
    fn unnaked<V>(&self, data: V) -> MultiplyLocated<V, ChoreoLS>;

    /// Calls a choreography.
    fn call<R, M, Index, C: Choreography<R, L = M>>(&self, choreo: C) -> R
    where
        M: LocationSet + Subset<ChoreoLS, Index>;

    /// Calls a choreography on a subset of locations.
    fn enclave<R, S: LocationSet, C: Choreography<R, L = S>, Index>(
        &self,
        choreo: C,
    ) -> MultiplyLocated<R, S>
    where
        S: Subset<ChoreoLS, Index>;

    /// Performs parallel computation.
    fn parallel<V, S: LocationSet, Index>(
        &self,
        locations: S,
        computation: impl Fn() -> V, // TODO: add unwrapper for S
    ) -> Faceted<V, S>
    where
        S: Subset<ChoreoLS, Index>;

    /// Performs fanout computation.
    fn fanout<
        // return value type
        V,
        // locations looping over
        QS: LocationSet,
        // FanOut Choreography over L iterating over QS returning V
        FOC: FanOutChoreography<V, L = ChoreoLS, QS = QS>,
        // Proof that QS is a subset of L
        QSSubsetL,
        QSFoldable,
    >(
        &self,
        locations: QS,
        c: FOC,
    ) -> Faceted<V, QS>
    where
        QS: Subset<ChoreoLS, QSSubsetL>,
        QS: LocationSetFoldable<ChoreoLS, QS, QSFoldable>;

    /// Performs fanin computation.
    fn fanin<
        // return value type
        V,
        // locations looping over
        QS: LocationSet,
        // Recipient locations
        RS: LocationSet,
        // FanIn Choreography over L iterating over QS returning V
        FIC: FanInChoreography<V, L = ChoreoLS, QS = QS, RS = RS>,
        // Proof that QS is a subset of L
        QSSubsetL,
        RSSubsetL,
        QSFoldable,
    >(
        &self,
        locations: QS,
        c: FIC,
    ) -> MultiplyLocated<Quire<V, QS>, RS>
    where
        QS: Subset<ChoreoLS, QSSubsetL>,
        RS: Subset<ChoreoLS, RSSubsetL>,
        QS: LocationSetFoldable<ChoreoLS, QS, QSFoldable>;
}

/// Special choreography for fanout
pub trait FanOutChoreography<V> {
    /// All locations involved in the choreography
    type L: LocationSet;
    /// Locations looping over
    type QS: LocationSet;
    /// TODO: documentation
    fn run<Q: ChoreographyLocation, QSSubsetL, QMemberL, QMemberQS>(
        &self,
        op: &impl ChoreoOp<Self::L>,
    ) -> Located<V, Q>
    where
        Self::QS: Subset<Self::L, QSSubsetL>,
        Q: Member<Self::L, QMemberL>,
        Q: Member<Self::QS, QMemberQS>;
}

/// Special choreography for fanin
pub trait FanInChoreography<V> {
    /// All locations involved in the choreography
    type L: LocationSet;
    /// Locations looping over
    type QS: LocationSet;
    /// Recipient locations
    type RS: LocationSet;
    /// run a choreography
    fn run<Q: ChoreographyLocation, QSSubsetL, RSSubsetL, QMemberL, QMemberQS>(
        &self,
        op: &impl ChoreoOp<Self::L>,
    ) -> MultiplyLocated<V, Self::RS>
    where
        Self::QS: Subset<Self::L, QSSubsetL>,
        Self::RS: Subset<Self::L, RSSubsetL>,
        Q: Member<Self::L, QMemberL>,
        Q: Member<Self::QS, QMemberQS>;
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
    type L: LocationSet;
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
///
/// The type parameter `L` is the location set that the transport is operating on.
///
/// The type parameter `TargetLocation` is the target `ChoreographyLocation`.
pub trait Transport<L: LocationSet, TargetLocation: ChoreographyLocation> {
    /// Returns a list of locations.
    fn locations(&self) -> Vec<&'static str>;
    /// Sends a message from `from` to `to`.
    fn send<V: Portable>(&self, from: &str, to: &str, data: &V) -> ();
    /// Receives a message from `from` to `at`.
    fn receive<V: Portable>(&self, from: &str, at: &str) -> V;
}

/// Provides a method to perform end-point projection.
pub struct Projector<
    // `LS` is a location set supported by the transport
    // Projector is capable of projecting any choreographies whose location set is a subset of `LS`
    TransportLS: LocationSet,
    // `L1` is the projection target
    Target: ChoreographyLocation,
    // `T` is the transport that supports locations `LS` and for target `L1`
    T: Transport<TransportLS, Target>,
    Index,
> where
    Target: Member<TransportLS, Index>,
{
    target: PhantomData<Target>,
    transport: T,
    location_set: PhantomData<TransportLS>,
    index: PhantomData<Index>,
}

impl<
        TransportLS: LocationSet,
        Target: ChoreographyLocation,
        B: Transport<TransportLS, Target>,
        Index,
    > Projector<TransportLS, Target, B, Index>
where
    Target: Member<TransportLS, Index>,
{
    /// Constructs a `Projector` struct.
    ///
    /// - `target` is the projection target of the choreography.
    /// - `transport` is an implementation of `Transport`.
    pub fn new(target: Target, transport: B) -> Self {
        _ = target;
        Projector {
            target: PhantomData,
            transport,
            location_set: PhantomData,
            index: PhantomData,
        }
    }

    /// Constructs a `Located` struct located at the projection target using the actual value.
    ///
    /// Use this method to run a choreography that takes a located value as an input.
    pub fn local<V>(&self, value: V) -> Located<V, Target> {
        Located::local(value)
    }

    /// Constructs a `Located` struct *NOT* located at the projection target.
    ///
    /// Use this method to run a choreography that takes a located value as an input.
    pub fn remote<V, L: ChoreographyLocation, Index2>(&self, at: L) -> Located<V, L>
    where
        L: Member<<Target as Member<TransportLS, Index>>::Remainder, Index2>,
    {
        _ = at;
        Located::remote()
    }

    /// Unwraps a located value at the projection target.
    ///
    /// Use this method to access the located value returned by a choreography.
    pub fn unwrap<L: LocationSet, V, Index1, Index2>(&self, located: MultiplyLocated<V, L>) -> V
    where
        L: Subset<TransportLS, Index1>,
        Target: Member<L, Index2>,
    {
        located.value.unwrap()
    }

    /// Performs end-point projection and runs a choreography.
    pub fn epp_and_run<
        'a,
        V,
        // location set of the choreography to EPP
        ChoreoLS: LocationSet,
        C: Choreography<V, L = ChoreoLS>,
        IndexSet,
    >(
        &'a self,
        choreo: C,
    ) -> V
    where
        ChoreoLS: Subset<TransportLS, IndexSet>,
    {
        struct EppOp<
            'a,
            ChoreoLS: LocationSet, // L is a location set associated with the choreography
            Target: ChoreographyLocation,
            TransportLS: LocationSet,
            B: Transport<TransportLS, Target>,
        > {
            target: PhantomData<Target>,
            transport: &'a B,
            locations: Vec<&'static str>,
            marker: PhantomData<ChoreoLS>,
            projector_location_set: PhantomData<TransportLS>,
        }
        impl<
                'a,
                ChoreoLS: LocationSet,
                Target: ChoreographyLocation,
                TransportLS: LocationSet,
                B: Transport<TransportLS, Target>,
            > ChoreoOp<ChoreoLS> for EppOp<'a, ChoreoLS, Target, TransportLS, B>
        {
            fn locally<V, L1: ChoreographyLocation, Index>(
                &self,
                _location: L1,
                computation: impl Fn(Unwrapper<L1>) -> V,
            ) -> MultiplyLocated<V, LocationSet!(L1)> {
                if L1::name() == Target::name() {
                    let unwrapper = Unwrapper {
                        phantom: PhantomData,
                    };
                    let value = computation(unwrapper);
                    MultiplyLocated::local(value)
                } else {
                    MultiplyLocated::remote()
                }
            }

            fn comm<
                L: LocationSet,
                Sender: ChoreographyLocation,
                Receiver: ChoreographyLocation,
                V: Portable,
                Index1,
                Index2,
                Index3,
            >(
                &self,
                _sender: Sender,
                _receiver: Receiver,
                data: &MultiplyLocated<V, L>,
            ) -> MultiplyLocated<V, LocationSet!(Receiver)> {
                if Sender::name() == Target::name() && Sender::name() == Receiver::name() {
                    let s = serde_json::to_string(data.value.as_ref().unwrap()).unwrap();
                    return MultiplyLocated::local(serde_json::from_str(s.as_str()).unwrap());
                }
                if Sender::name() == Target::name() {
                    self.transport.send(
                        Sender::name(),
                        Receiver::name(),
                        data.value.as_ref().unwrap(),
                    );
                    MultiplyLocated::remote()
                } else if Receiver::name() == Target::name() {
                    let value = self.transport.receive(Sender::name(), Receiver::name());
                    MultiplyLocated::local(value)
                } else {
                    MultiplyLocated::remote()
                }
            }

            fn broadcast<
                L: LocationSet,
                Sender: ChoreographyLocation,
                V: Portable,
                Index1,
                Index2,
            >(
                &self,
                _sender: Sender,
                data: MultiplyLocated<V, L>,
            ) -> V {
                if Sender::name() == Target::name() {
                    for dest in &self.locations {
                        if Target::name() != *dest {
                            self.transport.send(
                                &Target::name(),
                                &dest,
                                data.value.as_ref().unwrap(),
                            );
                        }
                    }
                    return data.value.unwrap();
                } else {
                    self.transport.receive(Sender::name(), &Target::name())
                }
            }

            fn multicast<
                Sender: ChoreographyLocation,
                V: Portable,
                D: LocationSet,
                Index1,
                Index2,
            >(
                &self,
                _src: Sender,
                _destination: D,
                data: &MultiplyLocated<V, LocationSet!(Sender)>,
            ) -> MultiplyLocated<V, D> {
                if Sender::name() == Target::name() {
                    for dest in D::to_string_list() {
                        if Target::name() != dest {
                            self.transport.send(
                                &Target::name(),
                                dest,
                                data.value.as_ref().unwrap(),
                            );
                        }
                    }
                    let s = serde_json::to_string(data.value.as_ref().unwrap()).unwrap();
                    return MultiplyLocated::local(serde_json::from_str(s.as_str()).unwrap());
                } else {
                    let mut is_receiver = false;
                    for dest in D::to_string_list() {
                        if Target::name() == dest {
                            is_receiver = true;
                        }
                    }
                    if is_receiver {
                        let v = self.transport.receive(Sender::name(), Target::name());
                        return MultiplyLocated::local(v);
                    } else {
                        return MultiplyLocated::remote();
                    }
                }
            }

            fn naked<S: LocationSet, V, Index>(&self, data: MultiplyLocated<V, S>) -> V {
                return data.value.unwrap();
            }

            fn unnaked<V>(&self, data: V) -> MultiplyLocated<V, ChoreoLS> {
                return MultiplyLocated::local(data);
            }

            fn call<R, M, Index, C: Choreography<R, L = M>>(&self, choreo: C) -> R
            where
                M: LocationSet + Subset<ChoreoLS, Index>,
            {
                let op: EppOp<'a, M, Target, TransportLS, B> = EppOp {
                    target: PhantomData::<Target>,
                    transport: &self.transport,
                    locations: self.transport.locations(),
                    marker: PhantomData::<M>,
                    projector_location_set: PhantomData::<TransportLS>,
                };
                choreo.run(&op)
            }

            fn enclave<R, S: LocationSet, C: Choreography<R, L = S>, Index>(
                &self,
                choreo: C,
            ) -> MultiplyLocated<R, S> {
                let locs_vec = S::to_string_list();

                for location in &locs_vec {
                    if *location == Target::name().to_string() {
                        let op = EppOp {
                            target: PhantomData::<Target>,
                            transport: self.transport,
                            locations: locs_vec,
                            marker: PhantomData::<S>,
                            projector_location_set: PhantomData::<TransportLS>,
                        };
                        return MultiplyLocated::local(choreo.run(&op));
                    }
                }
                MultiplyLocated::remote()
            }

            fn parallel<V, S: LocationSet, Index>(
                &self,
                _locations: S,
                computation: impl Fn() -> V, // TODO: add unwrapper for S
            ) -> Faceted<V, S>
            where
                S: Subset<ChoreoLS, Index>,
            {
                let mut values = HashMap::new();
                for location in S::to_string_list() {
                    if location == Target::name() {
                        let v = computation();
                        values.insert(String::from(location), v);
                    }
                }
                Faceted {
                    value: values,
                    phantom: PhantomData,
                }
            }

            fn fanout<
                // return value type
                V,
                // locations looping over
                QS: LocationSet,
                // FanOut Choreography over L iterating over QS returning V
                FOC: FanOutChoreography<V, L = ChoreoLS, QS = QS>,
                // Proof that QS is a subset of L
                QSSubsetL,
                QSFoldable,
            >(
                &self,
                _locations: QS,
                c: FOC,
            ) -> Faceted<V, QS>
            where
                QS: Subset<ChoreoLS, QSSubsetL>,
                QS: LocationSetFoldable<ChoreoLS, QS, QSFoldable>,
            {
                let op: EppOp<ChoreoLS, Target, TransportLS, B> = EppOp {
                    target: PhantomData::<Target>,
                    transport: self.transport,
                    locations: self.transport.locations(),
                    marker: PhantomData::<ChoreoLS>,
                    projector_location_set: PhantomData::<TransportLS>,
                };
                let values = HashMap::new();

                struct Loop<
                    'a,
                    ChoreoLS: LocationSet,
                    Target: ChoreographyLocation,
                    TransportLS: LocationSet,
                    B: Transport<TransportLS, Target>,
                    V,
                    QSSubsetL,
                    QS: LocationSet + Subset<ChoreoLS, QSSubsetL>,
                    FOC: FanOutChoreography<V, L = ChoreoLS, QS = QS>,
                > {
                    phantom: PhantomData<(V, QS, QSSubsetL, FOC)>,
                    op: EppOp<'a, ChoreoLS, Target, TransportLS, B>,
                    foc: FOC,
                }

                impl<
                        'a,
                        ChoreoLS: LocationSet,
                        Target: ChoreographyLocation,
                        TransportLS: LocationSet,
                        B: Transport<TransportLS, Target>,
                        V,
                        QSSubsetL,
                        QS: LocationSet + Subset<ChoreoLS, QSSubsetL>,
                        FOC: FanOutChoreography<V, L = ChoreoLS, QS = QS>,
                    > LocationSetFolder<HashMap<String, V>>
                    for Loop<'a, ChoreoLS, Target, TransportLS, B, V, QSSubsetL, QS, FOC>
                {
                    type L = ChoreoLS;
                    type QS = QS;
                    fn f<Q: ChoreographyLocation, QSSubsetL2, QMemberL, QMemberQS>(
                        &self,
                        mut acc: HashMap<String, V>,
                        _: Q,
                    ) -> HashMap<String, V>
                    where
                        Self::QS: Subset<Self::L, QSSubsetL2>,
                        Q: Member<Self::L, QMemberL>,
                        Q: Member<Self::QS, QMemberQS>,
                    {
                        let v = self.foc.run::<Q, QSSubsetL, QMemberL, QMemberQS>(&self.op);
                        match v.value {
                            Some(value) => {
                                acc.insert(String::from(Q::name()), value);
                            }
                            None => {}
                        };
                        acc
                    }
                }
                let values = QS::foldr(
                    Loop::<ChoreoLS, Target, TransportLS, B, V, QSSubsetL, QS, FOC> {
                        phantom: PhantomData,
                        op,
                        foc: c,
                    },
                    values,
                );
                Faceted {
                    value: values,
                    phantom: PhantomData,
                }
            }
            fn fanin<
                // return value type
                V,
                // locations looping over
                QS: LocationSet,
                // Recipient locations
                RS: LocationSet,
                // FanIn Choreography over L iterating over QS returning V
                FIC: FanInChoreography<V, L = ChoreoLS, QS = QS, RS = RS>,
                // Proof that QS is a subset of L
                QSSubsetL,
                RSSubsetL,
                QSFoldable,
            >(
                &self,
                _locations: QS,
                c: FIC,
            ) -> MultiplyLocated<Quire<V, QS>, RS>
            where
                QS: Subset<ChoreoLS, QSSubsetL>,
                RS: Subset<ChoreoLS, RSSubsetL>,
                QS: LocationSetFoldable<ChoreoLS, QS, QSFoldable>,
            {
                let op: EppOp<ChoreoLS, Target, TransportLS, B> = EppOp {
                    target: PhantomData::<Target>,
                    transport: self.transport,
                    locations: self.transport.locations(),
                    marker: PhantomData::<ChoreoLS>,
                    projector_location_set: PhantomData::<TransportLS>,
                };

                struct Loop<
                    'a,
                    ChoreoLS: LocationSet,
                    Target: ChoreographyLocation,
                    TransportLS: LocationSet,
                    B: Transport<TransportLS, Target>,
                    V,
                    QSSubsetL,
                    QS: LocationSet + Subset<ChoreoLS, QSSubsetL>,
                    RSSubsetL,
                    RS: LocationSet + Subset<ChoreoLS, RSSubsetL>,
                    FIC: FanInChoreography<V, L = ChoreoLS, QS = QS, RS = RS>,
                > {
                    phantom: PhantomData<(V, QS, QSSubsetL, RS, RSSubsetL, FIC)>,
                    op: EppOp<'a, ChoreoLS, Target, TransportLS, B>,
                    fic: FIC,
                }

                impl<
                        'a,
                        ChoreoLS: LocationSet,
                        Target: ChoreographyLocation,
                        TransportLS: LocationSet,
                        B: Transport<TransportLS, Target>,
                        V,
                        QSSubsetL,
                        QS: LocationSet + Subset<ChoreoLS, QSSubsetL>,
                        RSSubsetL,
                        RS: LocationSet + Subset<ChoreoLS, RSSubsetL>,
                        FIC: FanInChoreography<V, L = ChoreoLS, QS = QS, RS = RS>,
                    > LocationSetFolder<HashMap<String, V>>
                    for Loop<
                        'a,
                        ChoreoLS,
                        Target,
                        TransportLS,
                        B,
                        V,
                        QSSubsetL,
                        QS,
                        RSSubsetL,
                        RS,
                        FIC,
                    >
                {
                    type L = ChoreoLS;
                    type QS = QS;

                    fn f<Q: ChoreographyLocation, QSSubsetL2, QMemberL, QMemberQS>(
                        &self,
                        mut acc: HashMap<String, V>,
                        _: Q,
                    ) -> HashMap<String, V>
                    where
                        Self::QS: Subset<Self::L, QSSubsetL2>,
                        Q: Member<Self::L, QMemberL>,
                        Q: Member<Self::QS, QMemberQS>,
                    {
                        let v = self
                            .fic
                            .run::<Q, QSSubsetL, RSSubsetL, QMemberL, QMemberQS>(&self.op);
                        // if the target is in RS, `v` has a value (`Some`)
                        match v.value {
                            Some(value) => {
                                acc.insert(String::from(Q::name()), value);
                            }
                            None => {}
                        }
                        acc
                    }
                }

                let values = QS::foldr(
                    Loop::<ChoreoLS, Target, TransportLS, B, V, QSSubsetL, QS, RSSubsetL, RS, FIC> {
                        phantom: PhantomData,
                        op,
                        fic: c,
                    },
                    HashMap::new(),
                );

                MultiplyLocated::<Quire<V, QS>, RS>::local(Quire {
                    value: values,
                    phantom: PhantomData,
                })
            }
        }
        let op: EppOp<'a, ChoreoLS, Target, TransportLS, B> = EppOp {
            target: PhantomData::<Target>,
            transport: &self.transport,
            locations: self.transport.locations(),
            marker: PhantomData::<ChoreoLS>,
            projector_location_set: PhantomData::<TransportLS>,
        };
        choreo.run(&op)
    }
}

/// Provides a method to run a choreography without end-point projection.
pub struct Runner<RunnerLS: LocationSet> {
    marker: PhantomData<RunnerLS>,
}

impl<RunnerLS: LocationSet> Runner<RunnerLS> {
    /// Constructs a runner.
    pub fn new() -> Self {
        Runner {
            marker: PhantomData::<RunnerLS>,
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
    pub fn run<'a, V, C: Choreography<V, L = RunnerLS>>(&'a self, choreo: C) -> V {
        // Note: Technically, the location set of the choreography can be a subset of `RunnerLS`.
        // However, by using the same type, the compiler can infer `RunnerLS` for given choreography.

        struct RunOp<L>(PhantomData<L>);
        impl<L: LocationSet> ChoreoOp<L> for RunOp<L> {
            fn locally<V, L1: ChoreographyLocation, Index>(
                &self,
                _location: L1,
                computation: impl Fn(Unwrapper<L1>) -> V,
            ) -> MultiplyLocated<V, LocationSet!(L1)> {
                let unwrapper = Unwrapper {
                    phantom: PhantomData,
                };
                let value = computation(unwrapper);
                MultiplyLocated::local(value)
            }

            fn comm<
                S: LocationSet,
                Sender: ChoreographyLocation,
                Receiver: ChoreographyLocation,
                V: Portable,
                Index1,
                Index2,
                Index3,
            >(
                &self,
                _sender: Sender,
                _receiver: Receiver,
                data: &MultiplyLocated<V, S>,
            ) -> MultiplyLocated<V, LocationSet!(Receiver)> {
                // clone the value by encoding and decoding it. Requiring `Clone` could improve the performance but is not necessary.
                // Also, this is closer to what happens to the value with end-point projection.
                let s = serde_json::to_string(data.value.as_ref().unwrap()).unwrap();
                MultiplyLocated::local(serde_json::from_str(s.as_str()).unwrap())
            }

            fn broadcast<
                S: LocationSet,
                Sender: ChoreographyLocation,
                V: Portable,
                Index1,
                Index2,
            >(
                &self,
                _sender: Sender,
                data: MultiplyLocated<V, S>,
            ) -> V {
                data.value.unwrap()
            }

            fn multicast<
                Sender: ChoreographyLocation,
                V: Portable,
                D: LocationSet,
                Index1,
                Index2,
            >(
                &self,
                _src: Sender,
                _destination: D,
                data: &MultiplyLocated<V, LocationSet!(Sender)>,
            ) -> MultiplyLocated<V, D> {
                let s = serde_json::to_string(data.value.as_ref().unwrap()).unwrap();
                return MultiplyLocated::local(serde_json::from_str(s.as_str()).unwrap());
            }

            fn naked<S: LocationSet, V, Index>(&self, data: MultiplyLocated<V, S>) -> V {
                return data.value.unwrap();
            }

            fn unnaked<V>(&self, data: V) -> MultiplyLocated<V, L> {
                return MultiplyLocated::local(data);
            }

            fn call<R, M, Index, C: Choreography<R, L = M>>(&self, choreo: C) -> R
            where
                M: LocationSet + Subset<L, Index>,
            {
                let op: RunOp<M> = RunOp(PhantomData);
                choreo.run(&op)
            }

            fn enclave<R, S: LocationSet, C: Choreography<R, L = S>, Index>(
                &self,
                choreo: C,
            ) -> MultiplyLocated<R, S> {
                let op = RunOp::<S>(PhantomData);
                MultiplyLocated::local(choreo.run(&op))
            }

            fn parallel<V, S: LocationSet, Index>(
                &self,
                _locations: S,
                computation: impl Fn() -> V, // TODO: add unwrapper for S
            ) -> Faceted<V, S>
            where
                S: Subset<L, Index>,
            {
                let mut values = HashMap::new();
                for location in S::to_string_list() {
                    let v = computation();
                    values.insert(location.to_string(), v);
                }
                Faceted {
                    value: values,
                    phantom: PhantomData,
                }
            }
            fn fanout<
                // return value type
                V,
                // locations looping over
                QS: LocationSet,
                // FanOut Choreography over L iterating over QS returning V
                FOC: FanOutChoreography<V, L = L, QS = QS>,
                // Proof that QS is a subset of L
                QSSubsetL,
                QSFoldable,
            >(
                &self,
                _locations: QS,
                _c: FOC,
            ) -> Faceted<V, QS>
            where
                QS: Subset<L, QSSubsetL>,
                QS: LocationSetFoldable<L, QS, QSFoldable>,
            {
                todo!()
            }

            fn fanin<
                // return value type
                V,
                // locations looping over
                QS: LocationSet,
                // Recipient locations
                RS: LocationSet,
                // FanIn Choreography over L iterating over QS returning V
                FIC: FanInChoreography<V, L = L, QS = QS, RS = RS>,
                // Proof that QS is a subset of L
                QSSubsetL,
                RSSubsetL,
                QSFoldable,
            >(
                &self,
                _locations: QS,
                _c: FIC,
            ) -> MultiplyLocated<Quire<V, QS>, RS>
            where
                QS: Subset<L, QSSubsetL>,
                RS: Subset<L, RSSubsetL>,
                QS: LocationSetFoldable<L, QS, QSFoldable>,
            {
                todo!()
            }
        }
        let op: RunOp<RunnerLS> = RunOp(PhantomData);
        choreo.run(&op)
    }
}

extern crate chorus_derive;
pub use chorus_derive::{ChoreographyLocation, Superposition};
