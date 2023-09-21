//! Built-in transports.

pub mod http;
pub mod local;

use crate::core::{ChoreographyLocation, HCons, HList};
use crate::LocationSet;
use std::collections::HashMap;
use std::marker::PhantomData;

/// A generic struct for configuration of `Transport`.
#[derive(Clone)]
pub struct TransportConfig<L: HList, InfoType, TargetLocation: ChoreographyLocation, TargetInfoType>
{
    /// The information about locations
    pub info: HashMap<String, InfoType>,
    /// The information about the target choreography
    pub target_info: (TargetLocation, TargetInfoType),
    /// The struct is parametrized by the location set (`L`).
    pub location_set: PhantomData<L>,
}

impl<InfoType, TargetLocation: ChoreographyLocation, TargetInfoType>
    TransportConfig<LocationSet!(TargetLocation), InfoType, TargetLocation, TargetInfoType>
{
    /// A transport for a given target.
    pub fn for_target(location: TargetLocation, info: TargetInfoType) -> Self {
        Self {
            info: HashMap::new(),
            target_info: (location, info),
            location_set: PhantomData,
        }
    }
}

impl<L: HList, InfoType, TargetLocation: ChoreographyLocation, TargetInfoType>
    TransportConfig<L, InfoType, TargetLocation, TargetInfoType>
{
    /// Adds information about a new `ChoreographyLocation`.
    pub fn with<NewLocation: ChoreographyLocation>(
        self,
        _location: NewLocation,
        info: InfoType,
    ) -> TransportConfig<HCons<NewLocation, L>, InfoType, TargetLocation, TargetInfoType>
where {
        let mut new_info = HashMap::new();
        for (k, v) in self.info.into_iter() {
            new_info.insert(k, v);
        }
        new_info.insert(NewLocation::name().to_string(), info);

        TransportConfig {
            info: new_info,
            target_info: self.target_info,
            location_set: PhantomData,
        }
    }

    /// Finalize the `TransportConfig`.
    pub fn build(self) -> TransportConfig<L, InfoType, TargetLocation, TargetInfoType> {
        TransportConfig {
            info: self.info,
            location_set: PhantomData,
            target_info: self.target_info,
        }
    }
}
