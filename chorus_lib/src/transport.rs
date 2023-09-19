//! Built-in transports.

pub mod http;
pub mod local;

use crate::core::ChoreographyLocation;

/// A generic struct for configuration of `Transport`.
#[derive(Clone)]
pub struct TransportConfig<
    L: crate::core::HList,
    InfoType,
    TargetLocation: ChoreographyLocation,
    TargetInfoType,
> {
    /// The information about locations
    pub info: std::collections::HashMap<String, InfoType>,
    /// The information about the target choreography
    pub target_info: (TargetLocation, TargetInfoType),
    /// The struct is parametrized by the location set (`L`).
    pub location_set: std::marker::PhantomData<L>,
}

/// This macro makes a `TransportConfig`.
#[macro_export]
macro_rules! transport_config {
    ( $choreography_loc:ident, $( $loc:ident : $val:expr ),* $(,)? ) => {
        {
            let choreography_name = $choreography_loc::name().to_string();
            let mut config = std::collections::HashMap::new();
            let mut target_info = None;
            $(
                if $loc::name().to_string() != choreography_name{
                    config.insert($loc::name().to_string(), $val);
                } else {
                    target_info = Some($val);
                }
            )*

            $crate::transport::TransportConfig::<$crate::LocationSet!($( $loc ),*), _, _, _> {
                info: config,
                location_set: core::marker::PhantomData,
                target_info: ($choreography_loc, target_info.unwrap()),
            }
        }
    };
}
