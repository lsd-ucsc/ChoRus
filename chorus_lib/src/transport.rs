//! Built-in transports.

pub mod http;
pub mod local;

use crate::core::{ChoreographyLocation, HList};
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
