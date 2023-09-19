//! Built-in transports.

pub mod http;
pub mod local;

/// A generic struct for configuration of `Transport`.
#[derive(Clone)]
pub struct TransportConfig<L: crate::core::HList, InfoType> {
    /// The information about locations
    pub info: std::collections::HashMap<String, InfoType>,
    /// The struct is parametrized by the location set (`L`).
    pub location_set: std::marker::PhantomData<L>,
}

/// This macro makes a `TransportConfig`.
#[macro_export]
macro_rules! transport_config {
    ( $( $loc:ident : $val:expr ),* $(,)? ) => {
        {
            let mut config = std::collections::HashMap::new();
            $(
                config.insert($loc::name().to_string(), $val);
            )*

            $crate::transport::TransportConfig::<$crate::LocationSet!($( $loc ),*), _> {
                info: config,
                location_set: core::marker::PhantomData
            }
        }
    };
}
