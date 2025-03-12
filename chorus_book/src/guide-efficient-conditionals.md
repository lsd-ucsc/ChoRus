# Efficient Conditionals with Conclaves and MLVs

## `broadcast` incurs unnecessary communication

In [the previous section](./guide-choreography.html#broadcast), we discussed how the `broadcast` operator can be used to implement a conditional behavior in a choreography. In short, the `broadcast` operator sends a located value from a source location to all other locations, making the value available at all locations. The resulting value is a normal (not `Located`) value and it can be used to make a branch.

However, the `broadcast` operator can incur unnecessary communication when not all locations need to receive the value. Consider a simple key-value store where a _client_ sends either a `Get` or `Put` request to a _primary_ server, and the primary server forwards the request to a _backup_ server if the request is a `Put`. The backup server does not need to receive the request if the request is a `Get`.

Using the `broadcast` operator, this protocol can be implemented as follows:

```rust
{{#include ./header.txt}}
#
# fn read_request() -> Request {
#     Request::Put("key".to_string(), "value".to_string())
# }
# fn get_value(key: &Key) -> Option<Value> {
#     Some("value".to_string())
# }
# fn set_value(key: &Key, value: &Value) {
#     println!("Saved key: {} and value: {}", key, value);
# }
#
#[derive(ChoreographyLocation)]
struct Client;

#[derive(ChoreographyLocation)]
struct Primary;

#[derive(ChoreographyLocation)]
struct Backup;

type Key = String;
type Value = String;

#[derive(Serialize, Deserialize)]
enum Request {
    Get(Key),
    Put(Key, Value),
}

#[derive(Serialize, Deserialize)]
enum Response {
    GetOk(Option<Value>),
    PutOk,
}

struct KeyValueStoreChoreography;

impl Choreography<Located<Response, Client>> for KeyValueStoreChoreography {
    type L = LocationSet!(Client, Primary, Backup);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Client> {
        // Read the request from the client
        let request_at_client: Located<Request, Client> = op.locally(Client, |_| read_request());
        // Send the request to the primary server
        let request_at_primary: Located<Request, Primary> =
            op.comm(Client, Primary, &request_at_client);
        // Check if the request is a `Put`
        let is_put_at_primary: Located<bool, Primary> = op.locally(Primary, |un| {
            matches!(un.unwrap(&request_at_primary), Request::Put(_, _))
        });
        // Broadcast the `is_put_at_primary` to all locations so it can be used for branching
        let is_put: bool = op.broadcast(Primary, is_put_at_primary); // <-- Incurs unnecessary communication
        // Depending on the request, set or get the value
        let response_at_primary = if is_put {
            let request_at_backup: Located<Request, Backup> =
                op.comm(Primary, Backup, &request_at_primary);
            op.locally(Backup, |un| match un.unwrap(&request_at_backup) {
                Request::Put(key, value) => set_value(key, value),
                _ => (),
            });
            op.locally(Primary, |_| Response::PutOk)
        } else {
            op.locally(Primary, |un| {
                let key = match un.unwrap(&request_at_primary) {
                    Request::Get(key) => key,
                    _ => &"".to_string(),
                };
                Response::GetOk(get_value(key))
            })
        };
        // Send the response from the primary to the client
        let response_at_client = op.comm(Primary, Client, &response_at_primary);
        response_at_client
    }
}
```

While this implementation works, it incurs unnecessary communication. When we branch on `is_put`, we broadcast the value to all locations. This is necessary to make sure that the value is available at all locations so it can be used as a normal, non-located value. However, notice that the client does not need to receive the value. Regardless of whether the request is a `Put` or `Get`, the client should wait for the response from the primary server.

## Changing the census with `conclave`

To avoid unnecessary communication, we can use the `conclave` operator. The `conclave` operator is similar to [the `call` operator](./guide-higher-order-choreography.html) but executes a sub-choreography only at locations that are included in its location set. Inside the sub-choreography, `broadcast` only sends the value to the locations that are included in the location set. This allows us to avoid unnecessary communication.

Let's refactor the previous example using the `conclave` operator. We define a sub-choreography `HandleRequestChoreography` that describes how the primary and backup servers (but not the client) handle the request and use the `conclave` operator to execute the sub-choreography.

```rust
{{#include ./header.txt}}
#
# fn read_request() -> Request {
#     Request::Put("key".to_string(), "value".to_string())
# }
# fn get_value(key: &Key) -> Option<Value> {
#     Some("value".to_string())
# }
# fn set_value(key: &Key, value: &Value) {
#     println!("Saved key: {} and value: {}", key, value);
# }
#
# #[derive(ChoreographyLocation)]
# struct Client;
#
# #[derive(ChoreographyLocation)]
# struct Primary;
#
# #[derive(ChoreographyLocation)]
# struct Backup;
#
# type Key = String;
# type Value = String;
#
# #[derive(Serialize, Deserialize)]
# enum Request {
#     Get(Key),
#     Put(Key, Value),
# }
#
# #[derive(Serialize, Deserialize)]
# enum Response {
#     GetOk(Option<Value>),
#     PutOk,
# }
#
struct HandleRequestChoreography {
    request: Located<Request, Primary>,
}

// This sub-choreography describes how the primary and backup servers handle the request
impl Choreography<Located<Response, Primary>> for HandleRequestChoreography {
    type L = LocationSet!(Primary, Backup);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Primary> {
        let is_put_request: Located<bool, Primary> = op.locally(Primary, |un| {
            matches!(un.unwrap(&self.request), Request::Put(_, _))
        });
        let is_put: bool = op.broadcast(Primary, is_put_request);
        let response_at_primary = if is_put {
            let request_at_backup: Located<Request, Backup> =
                op.comm(Primary, Backup, &self.request);
            op.locally(Backup, |un| match un.unwrap(&request_at_backup) {
                Request::Put(key, value) => set_value(key, value),
                _ => (),
            });
            op.locally(Primary, |_| Response::PutOk)
        } else {
            op.locally(Primary, |un| {
                let key = match un.unwrap(&self.request) {
                    Request::Get(key) => key,
                    _ => &"".to_string(),
                };
                Response::GetOk(get_value(key))
            })
        };
        response_at_primary
    }
}

struct KeyValueStoreChoreography;

impl Choreography<Located<Response, Client>> for KeyValueStoreChoreography {
    type L = LocationSet!(Client, Primary, Backup);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Client> {
        let request_at_client: Located<Request, Client> = op.locally(Client, |_| read_request());
        let request_at_primary: Located<Request, Primary> =
            op.comm(Client, Primary, &request_at_client);
        // Execute the sub-choreography only at the primary and backup servers
        let response: MultiplyLocated<Located<Response, Primary>, LocationSet!(Primary, Backup)> =
            op.conclave(HandleRequestChoreography {
                request: request_at_primary,
            });
        let response_at_primary: Located<Response, Primary> = response.flatten();
        let response_at_client = op.comm(Primary, Client, &response_at_primary);
        response_at_client
    }
}
```

In this refactored version, the `HandleRequestChoreography` sub-choreography describes how the primary and backup servers handle the request. The `conclave` operator executes the sub-choreography only at the primary and backup servers. The `broadcast` operator inside the sub-choreography sends the value only to the primary and backup servers, avoiding unnecessary communication.

The `conclave` operator returns a return value of the sub-choreography wrapped as a `MultiplyLocated` value. Since `HandleRequestChoreography` returns a `Located<Response, Primary>`, the return value of the `conclave` operator is a `MultiplyLocated<Located<Response, Primary>, LocationSet!(Primary, Backup)>`. To get the located value at the primary server, we can use the `locally` operator to unwrap the `MultiplyLocated` value on the primary. Since this is a common pattern, we provide the `flatten` method on `MultiplyLocated` to simplify this operation.

With the `conclave` operator, we can avoid unnecessary communication and improve the efficiency of the choreography.

## Reusing Knowledge of Choice in Conclaves

The key idea behind the `conclave` operator is that a normal value inside a choreography is equivalent to a (multiply) located value at all locations executing the choreography. This is why a normal value in a sub-choreography becomes a multiply located value at all locations executing the sub-choreography when returned from the `conclave` operator.

It is possible to perform this conversion in the opposite direction as well. If we have a multiply located value at some locations, and those are the only locations executing the choreography, then we can obtain a normal value out of the multiply located value. This is useful when we want to reuse the already known information about a choice in a conclave.

Inside a choreography, we can use the `naked` operator to convert a multiply located value at locations `S` to a normal value if the census of the choreography is a subset of `S`.

For example, the above choreography can be written as follows:

```rust
{{#include ./header.txt}}
#
# fn read_request() -> Request {
#     Request::Put("key".to_string(), "value".to_string())
# }
# fn get_value(key: &Key) -> Option<Value> {
#     Some("value".to_string())
# }
# fn set_value(key: &Key, value: &Value) {
#     println!("Saved key: {} and value: {}", key, value);
# }
#
# #[derive(ChoreographyLocation)]
# struct Client;
#
# #[derive(ChoreographyLocation)]
# struct Primary;
#
# #[derive(ChoreographyLocation)]
# struct Backup;
#
# type Key = String;
# type Value = String;
#
# #[derive(Serialize, Deserialize)]
# enum Request {
#     Get(Key),
#     Put(Key, Value),
# }
#
# #[derive(Serialize, Deserialize)]
# enum Response {
#     GetOk(Option<Value>),
#     PutOk,
# }
#
struct HandleRequestChoreography {
    request: Located<Request, Primary>,
    is_put: MultiplyLocated<bool, LocationSet!(Primary, Backup)>,
}

impl Choreography<Located<Response, Primary>> for HandleRequestChoreography {
    type L = LocationSet!(Primary, Backup);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Primary> {
        // obtain a normal boolean because {Primary, Backup} is the census of the choreography
        let is_put: bool = op.naked(self.is_put);
        let response_at_primary = if is_put {
            // ...
#             let request_at_backup: Located<Request, Backup> =
#                 op.comm(Primary, Backup, &self.request);
#             op.locally(Backup, |un| match un.unwrap(&request_at_backup) {
#                 Request::Put(key, value) => set_value(key, value),
#                 _ => (),
#             });
#             op.locally(Primary, |_| Response::PutOk)
        } else {
            // ...
#             op.locally(Primary, |un| {
#                 let key = match un.unwrap(&self.request) {
#                     Request::Get(key) => key,
#                     _ => &"".to_string(),
#                 };
#                 Response::GetOk(get_value(key))
#             })
        };
        response_at_primary
    }
}

struct KeyValueStoreChoreography;

impl Choreography<Located<Response, Client>> for KeyValueStoreChoreography {
    type L = LocationSet!(Client, Primary, Backup);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<Response, Client> {
        let request_at_client: Located<Request, Client> = op.locally(Client, |_| read_request());
        let request_at_primary: Located<Request, Primary> =
            op.comm(Client, Primary, &request_at_client);
        let is_put_at_primary: Located<bool, Primary> = op.locally(Primary, |un| {
            matches!(un.unwrap(&request_at_primary), Request::Put(_, _))
        });
        // get a MLV by multicasting the boolean to the census of the sub-choreography
        let is_put: MultiplyLocated<bool, LocationSet!(Primary, Backup)> = op.multicast(
            Primary,
            <LocationSet!(Primary, Backup)>::new(),
            &is_put_at_primary,
        );
        let response: MultiplyLocated<Located<Response, Primary>, LocationSet!(Primary, Backup)> =
            op.conclave(HandleRequestChoreography {
                is_put,
                request: request_at_primary,
            });
        let response_at_primary: Located<Response, Primary> = response.flatten();
        let response_at_client = op.comm(Primary, Client, &response_at_primary);
        response_at_client
    }
}
```

In this version, we first `multicast` the boolean value to the census of the sub-choreography (`Primary` and `Client`) and we pass the MLV to the sub-choreography. Inside the sub-choreography, we use the `naked` operator to obtain a normal boolean value. This allows us to reuse the already known information about the choice in the sub-choreography.
