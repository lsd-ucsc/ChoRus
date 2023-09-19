//! Built-in transports.

pub mod http;
pub mod local;

use std::sync::Arc;
use crate::core::ChoreographyLocation;

/// A generic struct for configuration of `Transport`.
#[derive(Clone)]
pub struct TransportConfig<L: crate::core::HList, InfoType, TargetLocation: ChoreographyLocation, TargetInfoType> {
    /// The information about locations
    pub info: std::collections::HashMap<String, InfoType>,
    /// The information about the target choreography
    pub target_info: (TargetLocation, TargetInfoType),
    /// The struct is parametrized by the location set (`L`).
    pub location_set: std::marker::PhantomData<L>,
    // pub transport_channel: TransportChannel<L>,

}

/// A Transport channel used between multiple `Transport`s.
pub struct TransportChannel<L: crate::core::HList, T: Send + Sync>{
    /// The location set where the channel is defined on.
    pub location_set: std::marker::PhantomData<L>,
    queue_map: Arc<T>,
}

impl<L, T> Clone for TransportChannel<L, T>
where
    L: crate::core::HList,
    T: Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            location_set: self.location_set.clone(),
            queue_map: self.queue_map.clone(),  // This clones the Arc, not the underlying data
        }
    }
}

/// This macro makes a `TransportConfig`.
// #[macro_export]
// macro_rules! transport_config {
//     ( $( $loc:ident : $val:expr ),* $(,)? ) => {
//         {
//             let mut config = std::collections::HashMap::new();
//             $(
//                 config.insert($loc::name().to_string(), $val);
//             )*

//             $crate::transport::TransportConfig::<$crate::LocationSet!($( $loc ),*), _> {
//                 info: config,
//                 location_set: core::marker::PhantomData
//             }
//         }
//     };
// }


/// This macro makes a `TransportConfig`; V2.
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

            // println!("{}, {}", target_info.unwrap().0, target.info.unwrap().1);

            $crate::transport::TransportConfig::<$crate::LocationSet!($( $loc ),*), _, _, _> {
                info: config,
                location_set: core::marker::PhantomData,
                target_info: ($choreography_loc, target_info.unwrap()),
            }
        }
    };
}