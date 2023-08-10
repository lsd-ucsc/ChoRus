# Located Values

As we have seen in the [Choreography](./guide-choreography.md) section, a located value is a value that is available only at a specific location. In this section, we will discuss located values in more detail.

## `Located` struct

The `Located` struct represents a located value. It is a generic struct that takes two type parameters: a type parameter `V` that represents the type of the value, and a type parameter `L1` that represents the location where the value is available.

```rust,ignore
pub struct Located<V, L1>
where
    L1: ChoreographyLocation,
{
    // ...
}
```

The `Located` struct can be in one of the two states: `Local` and `Remote`. The `Local` state represents a located value that is available at the current location. The `Remote` state represents a located value that is available at a different location.

## `Portable` trait

Located values can be sent from one location to another using the `comm` operator and unwrapped using the `broadcast` operator if the value type implements the `Portable` trait.

```rust,ignore
trait Portable: Serialize + DeserializeOwned {}
```

The `Portable` is defined as above. The `Serialize` and `DeserializeOwned` traits are from the `serde` crate and are used to serialize and deserialize the value for communication.

The `chorus_lib` crate re-exports the `Serialize` and `Deserialize` from `serde`. In many cases, those traits can automatically be derived using the `#[derive(Serialize, Deserialize, Clone)]` attribute.

For the complete list of types that supports automatic derivation of `Serialize` and `Deserialize`, see the [serde documentation](https://serde.rs/data-model.html#types). The documentation also explains how to implement `Serialize` and `Deserialize` for custom types.
