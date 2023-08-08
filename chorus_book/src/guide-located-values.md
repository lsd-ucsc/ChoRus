# Located Values

As we have seen in the [Choreography](./guide-choreography.md) section, a located value is a value that is available only at a specific location. In this section, we will discuss located values in more detail.

## `Located` struct

The `Located` struct represents a located value. It is a generic struct that takes two type parameters: a type parameter `V` that represents the type of the value, and a type parameter `L1` that represents the location where the value is available.

```rust,ignore
pub struct Located<V, L1>
where
    V: ChoreographicValue,
    L1: ChoreographyLocation,
{
    // ...
}
```

The `Located` struct can be in one of the two states: `Local` and `Remote`. The `Local` state represents a located value that is available at the current location. The `Remote` state represents a located value that is available at a different location.

## `ChoreographicValue` trait

Values that can be used as located values must implement the `ChoreographicValue` trait. This trait ensures that the value can be sent to a different location.

```rust,ignore
trait ChoreographicValue: Serialize + DeserializeOwned + Clone {}
```

The `ChoreographicValue` is defined as above. The `Serialize` and `DeserializeOwned` traits are from the `serde` crate and are used to serialize and deserialize the value for communication. The `Clone` trait is used to clone the value.

The `chorus_lib` crate re-exports the `Serialize` and `Deserialize` from `serde`. In many cases, those traits can automatically be derived using the `#[derive(Serialize, Deserialize, Clone)]` attribute.

For the complete list of types that supports automatic derivation of `Serialize` and `Deserialize`, see the [serde documentation](https://serde.rs/data-model.html#types). The documentation also explains how to implement `Serialize` and `Deserialize` for custom types.
