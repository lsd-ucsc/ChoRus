//! Built-in transports.

pub mod http;
pub mod local;

use crate::core::{ChoreographyLocation, HCons, HList, LocationSet};
use std::collections::HashMap;
use std::marker::PhantomData;

/// A generic struct for configuration of `Transport`.
#[derive(Clone)]
pub struct TransportConfig<L: HList, InfoType, TargetLocation: ChoreographyLocation, TargetInfoType>
{
    /// The information about locations
    info: HashMap<String, InfoType>,
    /// The information about the target choreography
    target_info: (TargetLocation, TargetInfoType),
    /// The struct is parametrized by the location set (`L`).
    location_set: PhantomData<L>,
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
        mut self,
        _location: NewLocation,
        info: InfoType,
    ) -> TransportConfig<HCons<NewLocation, L>, InfoType, TargetLocation, TargetInfoType>
where {
        self.info.insert(NewLocation::name().to_string(), info);

        TransportConfig {
            info: self.info,
            target_info: self.target_info,
            location_set: PhantomData,
        }
    }
}
