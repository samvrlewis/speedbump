# Speedbump

Flexible and extensible rate limiting with batteries included implementations.

## Concepts
The crate is built around two (user-definable) traits.

- **Store**: Defines how the rate limiting state (e.g., counters, timestamps) is persisted. Implementations could
  include in-memory, Redis, or other storage backends.
- **Strategy**: Defines the rate limiting algorithm (e.g., fixed window, token bucket) and how it uses the persisted
  state to determine if a request should be allowed or rate-limited.

These traits are used within a `Limiter` struct, which orchestrates the retrieval of state from the `Store`
and the application of the `Strategy` to decide whether a request should be rejected or allowed. This
`Limiter` can then be integrated into middleware.

## Todo
- [ ] Documentation
- [ ] More examples
- More strategies:
- [ ] Sliding window
- [ ] Token bucket
- [ ] Leaky bucket
- More stores:
    - [ ] Redis
    - [ ] Memcached
- Middleware implementations:
    - [ ] Tower
    - [ ] Actix