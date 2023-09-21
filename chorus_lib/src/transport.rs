//! Built-in transports.

pub mod http;
pub mod local;

use crate::core::{Append, ChoreographyLocation, HList};
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

/// Initiate a transport for a given target.
/// Note that information about other locations have to provided using `.with`
pub fn transport_for_target<InfoType, TargetLocation, TargetInfoType>(
    location: TargetLocation,
    info: TargetInfoType,
) -> TransportConfig<LocationSet!(TargetLocation), InfoType, TargetLocation, TargetInfoType>
where
    TargetLocation: ChoreographyLocation,
{
    TransportConfig::for_target(location, info)
}

impl<L: HList, InfoType, TargetLocation: ChoreographyLocation, TargetInfoType>
    TransportConfig<L, InfoType, TargetLocation, TargetInfoType>
{
	/// A transport for a given target
    fn for_target(location: TargetLocation, info: TargetInfoType) -> Self {
        Self {
            info: HashMap::new(),
            target_info: (location, info),
            location_set: PhantomData,
        }
    }

    /// Assuming you have a way to append to HLists, and that `L2` is the type-level list
    /// that results from appending `NewLocation` to `L`.
    pub fn with<NewLocation, L2>(
        self,
        _location: NewLocation,
        info: InfoType,
    ) -> TransportConfig<L2, InfoType, TargetLocation, TargetInfoType>
    where
        // L2: RemoveHead<NewLocation, Result = L>,
        L: Append<NewLocation, Result = L2>,
        NewLocation: ChoreographyLocation,
        L2: HList,
    {
        let mut new_info = HashMap::new();
        for (k, v) in self.info.into_iter() {
            new_info.insert(k, v);
        }
        // Assuming NewLocation has a `name` associated function
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

/// This macro makes a `TransportConfig`.
#[macro_export]
macro_rules! transport_config {
    ( $choreography_loc:ident => $choreography_val:expr, $( $loc:ident : $val:expr ),* $(,)? ) => {
        {
            let mut config = std::collections::HashMap::new();
            $(
                config.insert($loc::name().to_string(), $val);
            )*

            $crate::transport::TransportConfig::<$crate::LocationSet!($choreography_loc, $( $loc ),*), _, _, _> {
                info: config,
                location_set: std::marker::PhantomData,
                target_info: ($choreography_loc, $choreography_val),
            }
        }
    };
}
