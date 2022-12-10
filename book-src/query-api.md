# Query API

The Query API provides hook-based access to APIs with automatic caching
and request deduplication backed by Bounce’s state management mechanism.

#### Note

Bounce does not provide an implementation of HTTP Client.

You can use reqwest or gloo-net if your backend is using Restful API.

For GraphQL servers, you can use graphql-client in conjunction with reqwest.

### Query

A query is a state cached by an Input and queried automatically upon initialisation of the state and re-queried when the input changes.

Queries are usually tied to idempotent methods like GET, which means that they should be side-effect free and can be cached.

If your endpoint modifies data, you need to use a [mutation](#mutation).

API Reference:

- [`use_query_value`](https://docs.rs/bounce/latest/bounce/query/fn.use_query_value.html)

### Mutation

A hook to run a mutation and subscribes to its result.

A mutation is a state that is not started until the run method is invoked.
Mutations are usually used to modify data on the server.

API Reference:

- [`use_mutation`](https://docs.rs/bounce/latest/bounce/query/fn.use_mutation.html)
