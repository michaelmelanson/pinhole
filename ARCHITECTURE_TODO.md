# Pinhole Architecture - TODO

Architectural improvements identified in October 2025 assessment.

## Priority 0 - Critical

- [ ] **Protocol Versioning**
  - Add version field to message envelope
  - Implement version negotiation during connection
  - Support multiple protocol versions simultaneously
  - Location: `pinhole-protocol/src/messages.rs`

- [ ] **Logging and Observability**
  - Adopt `tracing` crate for structured logging
  - Add request/response middleware with correlation IDs
  - Add metrics: connection count, request latency, error rates
  - Location: `pinhole-framework/src/lib.rs`

- [ ] **Request Validation**
  - Implement typed actions with serde validation
  - Add validation decorators (email, length, range)
  - Generate automatic error responses for bad data
  - Location: `pinhole-protocol/src/action.rs`

- [ ] **Authentication Framework**
  - Add authenticated route trait with user identity
  - Implement session management
  - Add CSRF protection and rate limiting
  - Location: New framework module

## Priority 1 - Important

- [ ] **Error Handling Granularity**
  - Add structured error types with HTTP-like status codes
  - Distinguish retryable vs fatal errors
  - Support 401, 403, 429, etc. (not just 400/404/500)
  - Location: `pinhole-protocol/src/messages.rs`, `pinhole-client/src/error.rs`

- [ ] **Route Matching**
  - Implement path parameter extraction (`/user/{id}`)
  - Add query string parsing
  - Use trie-based router for O(log n) lookups
  - Location: `pinhole-framework/src/application.rs`

- [ ] **Storage Type System**
  - Add Number, Array, Object types (JSON-like)
  - Support nested structures
  - Location: `pinhole-protocol/src/storage.rs`

- [ ] **State Synchronisation**
  - Add state version numbers for optimistic concurrency
  - Implement conflict detection
  - Persist session storage with explicit expiry (don't clear on reconnect)
  - Location: `pinhole-client/src/network.rs`

- [ ] **Documentation**
  - Add module-level docs to all public modules
  - Add doc comments to all public APIs
  - Create architecture decision records (ADRs)
  - Write deployment guide and security considerations

## Priority 2 - Nice to Have

- [ ] **Stylesheet Completeness**
  - Add flexbox properties (justify-content, align-items, flex-grow)
  - Add pseudo-class support (`:hover`, `:focus`)
  - Add responsive rules (`@media`)
  - Location: `pinhole-protocol/src/stylesheet.rs`

- [ ] **Build and Deployment Tooling**
  - Add `Dockerfile` for reproducible builds
  - Add GitHub Actions workflow
  - Add health check route (`/health`)
  - Add graceful shutdown handling

- [ ] **Testing Gaps**
  - Add `proptest` for fuzzing message serialisation
  - Add load tests (1000 concurrent connections)
  - Add chaos tests (random disconnects, partial messages)

- [ ] **Performance Optimisations**
  - Use `HashMap` or trie for route lookup
  - Implement message pooling with `bytes::BytesMut`
  - Batch storage writes (flush every 100ms)
  - Cache serialised documents with ETag-like versioning
  - Add benchmarks to track regressions

## Priority 3 - Future

- [ ] **Incremental Rendering** _(currently an anti-plan)_
  - Add patch messages for partial updates
  - Consider JSON Patch (RFC 6902)
  - Only if design goals change from full-page model
