//! Built-in transports.

pub mod http;
pub mod local;

use crate::core::{ChoreographyLocation, HCons, HList, LocationSet};
use std::collections::HashMap;
use std::marker::PhantomData;

/// A generic struct for configuration of `Transport`.
#[derive(Clone)]
pub struct TransportConfig<Target: ChoreographyLocation, TargetInfo, L: HList, Info> {
    /// The information about locations
    pub info: HashMap<String, Info>,
    /// The information about the target choreography
    pub target_info: (Target, TargetInfo),
    /// The struct is parametrized by the location set (`L`).
    location_set: PhantomData<L>,
}

/// A builder for `TransportConfig`.
///
/// Use this builder to create a `TransportConfig` instance.
///
/// # Examples
///
/// ```
/// use chorus_lib::core::{LocationSet, ChoreographyLocation};
/// use chorus_lib::transport::TransportConfigBuilder;
///
/// #[derive(ChoreographyLocation)]
/// struct Alice;
///
/// #[derive(ChoreographyLocation)]
/// struct Bob;
///
/// let transport_config = TransportConfigBuilder::for_target(Alice, "value_for_target".to_string())
///    .with(Bob, "value_for_bob".to_string())
///    .build();
/// ```
pub struct TransportConfigBuilder<Target: ChoreographyLocation, TargetInfo, L: HList, Info> {
    target: (Target, TargetInfo),
    location_set: PhantomData<L>,
    info: HashMap<String, Info>,
}

impl<Target: ChoreographyLocation, TargetInfo, Info>
    TransportConfigBuilder<Target, TargetInfo, LocationSet!(Target), Info>
{
    /// Creates a new `TransportConfigBuilder` instance for a given target.
    pub fn for_target(target: Target, info: TargetInfo) -> Self {
        Self {
            target: (target, info),
            location_set: PhantomData,
            info: HashMap::new(),
        }
    }
}

impl<Target: ChoreographyLocation, TargetInfo, L: HList, Info>
    TransportConfigBuilder<Target, TargetInfo, L, Info>
{
    /// Adds information about a new `ChoreographyLocation`.
    ///
    /// This method tells the builder that the choreography involves a new location and how to communicate with it.
    pub fn with<NewLocation: ChoreographyLocation>(
        self,
        location: NewLocation,
        info: Info,
    ) -> TransportConfigBuilder<Target, TargetInfo, HCons<NewLocation, L>, Info> {
        _ = location;
        let mut new_info = self.info;
        new_info.insert(NewLocation::name().to_string(), info);
        TransportConfigBuilder {
            target: self.target,
            location_set: PhantomData,
            info: new_info,
        }
    }

    /// Builds a `TransportConfig` instance.
    pub fn build(self) -> TransportConfig<Target, TargetInfo, L, Info> {
        TransportConfig {
            info: self.info,
            target_info: self.target,
            location_set: PhantomData,
        }
    }
}
