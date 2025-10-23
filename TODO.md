# Pinhole TODO List

This file contains a list of tasks to be completed for the Pinhole project. Be sure to read the sections at the end, below the `---` divider for additional information. When completing a task, check it off.

## Critical Priority - Production Blockers

### Security Issues
- [x] Add TLS/encryption support for all network communications
- [x] Add message size limits to prevent DoS attacks (`Vec::resize` vulnerability in `pinhole-client/src/network.rs`)

### Error Handling
- [x] Create structured error types (replace `Box<dyn Error>`)
- [x] Implement error recovery instead of closing connections on any error
- [x] Add error message protocol variants (ServerToClientMessage::Error)
- [x] Replace silent `let _ =` failures with proper error handling
- [x] Add retry logic for transient network errors
- [x] Remove panic risks (e.g., `expect("Can't fire actions without a path set")` in `pinhole-client/src/network.rs`)
- [x] Add error messages to UI layer
- [x] Distinguish between recoverable and fatal errors

### Testing
- [ ] Add protocol serialization/deserialization tests
- [ ] Add integration tests for client-server communication
- [ ] Add malformed message handling tests
- [ ] Add concurrent connection tests
- [ ] Add security vulnerability tests (fuzzing, boundary conditions)
- [ ] Add action dispatch tests
- [ ] Add route matching tests
- [ ] Add stylesheet application tests
- [ ] Add UI rendering tests
- [ ] Add storage corruption recovery tests
- [ ] Target 70%+ code coverage

### Race Conditions & Concurrency
- [ ] Remove `block_on` calls from UI thread (`pinhole-client/src/main.rs`)
- [ ] Refactor to fully async message passing
- [ ] Add bounded channels with backpressure handling
- [ ] Add synchronization for concurrent storage access
- [ ] Fix storage directory creation race condition

## High Priority

### Protocol Improvements
- [ ] Add protocol versioning mechanism
- [ ] Add framing recovery for corrupted messages
- [ ] Add message type validation
- [ ] Add upper bound checks for message length (`u32::from_le_bytes` in `pinhole-client/src/network.rs`)
- [ ] Implement proper EOF vs error distinction in receive functions
- [ ] Add heartbeat/keepalive messages
- [ ] Add connection timeout configuration

### Storage Layer
- [ ] Implement atomic writes for persistent storage (temp file + rename pattern)
- [ ] Fix origin sanitization to prevent collisions
- [ ] Implement storage locking for multi-client scenarios
- [ ] Add storage encryption at rest
- [ ] Add storage backup and restore functionality
- [ ] Implement delta updates instead of full state clones

### Performance
- [ ] Implement delta/incremental document updates (not full re-renders)
- [ ] Reduce clone overhead (StateMap cloned per action)
- [ ] Optimize routing with hash-based lookup instead of linear O(n)
- [ ] Implement UI diffing/reconciliation
- [ ] Profile and optimize CBOR serialization hotspots

## Medium Priority

### Incomplete Features
- [ ] Complete stylesheet application (many StyleRule variants ignored)
- [ ] Add debug warnings for unsupported stylesheet rules
- [ ] Implement or remove Action.keys field (currently unused)
- [ ] Remove StateValue::Empty variant or implement usage
- [ ] Implement or remove Layout/Position/Size types (currently dead code)
- [ ] Add support for conditional rendering
- [ ] Add support for loops/iteration in UI
- [ ] Implement form validation framework

### Configuration & Deployment
- [ ] Make server address configurable (currently hardcoded `127.0.0.1:8080`)
- [ ] Add configuration file support (TOML/YAML)
- [ ] Add CLI argument parsing
- [ ] Add environment variable support
- [ ] Make port configurable
- [ ] Add graceful shutdown handling
- [ ] Add signal handlers (SIGTERM, SIGINT)

### Architecture
- [ ] Add middleware pattern for cross-cutting concerns (auth, logging, metrics)
- [ ] Implement session replication for horizontal scaling
- [ ] Design and implement load balancer compatibility
- [ ] Separate network/storage/UI concerns in client codebase
- [ ] Create common utilities crate to reduce duplication
- [ ] Add plugin/extension system for custom node types

### Observability
- [ ] Add structured logging throughout (tracing crate)
- [ ] Add metrics collection (Prometheus-compatible)
- [ ] Add distributed tracing support
- [ ] Add performance monitoring
- [ ] Add connection state monitoring
- [ ] Add error rate tracking
- [ ] Create debugging tools/dashboard

## Low Priority

### Code Quality
- [ ] Remove unused `#[allow(dead_code)]` annotations
- [ ] Fix inconsistent naming (StateValue::Empty vs StateValue::String(""))
- [ ] Add inline documentation for public APIs
- [ ] Add module-level documentation
- [ ] Add examples for each public API
- [ ] Run clippy and fix all warnings
- [ ] Format code with rustfmt
- [ ] Add CI/CD pipeline (GitHub Actions)

### Example Application
- [ ] Implement real persistence for TodoMVC example
- [ ] Add email validation in example
- [ ] Remove hardcoded test data
- [ ] Implement real authentication flow
- [ ] Add more example applications
- [ ] Create tutorial documentation

### Developer Experience
- [ ] Add cargo-watch instructions to README
- [ ] Add development environment setup guide
- [ ] Add debugging guide
- [ ] Add architecture documentation
- [ ] Add contribution guidelines
- [ ] Add code of conduct
- [ ] Create API documentation site

### UI/UX Improvements
- [ ] Add navigation bar UI chrome
- [ ] Add status bar
- [ ] Add loading indicators
- [ ] Add error notifications to UI
- [ ] Add connection status indicator
- [ ] Support for media nodes (images, video, audio)
- [ ] Add accessibility features (ARIA-like)
- [ ] Add keyboard navigation support

## Future Enhancements (Roadmap Items)

### Core Features
- [ ] Add more node types (media, grouping, links)
- [ ] Implement complete style system (CSS-like)
- [ ] Add animation support
- [ ] Add responsive design support
- [ ] Add dark mode support
- [ ] Add internationalization (i18n) support

### Advanced Features
- [ ] Embed extension language (Deno/JavaScript)
- [ ] Client-side action handlers with JavaScript bundles
- [ ] Polling mechanism (server-requested refresh)
- [ ] Server-sent events / subscriptions system
- [ ] WebSocket alternative transport
- [ ] HTTP/3 QUIC transport option

### Ecosystem
- [ ] Create package manager for Pinhole applications
- [ ] Build standard library of components
- [ ] Create developer tools (debugger, inspector)
- [ ] Create migration tools from web apps

## Anti-Goals (Explicitly NOT Planned)
- React-like component model (server-side encapsulation sufficient)
- Incremental page updates (full-page model works for most apps)
- DOM-like API (keep simple node tree model)

---

## Priority Legend
- **Critical Priority**: Must be fixed before any production use
- **High Priority**: Significantly impacts quality, security, or performance
- **Medium Priority**: Improves developer experience or feature completeness
- **Low Priority**: Polish and nice-to-haves

## File References
- Protocol: `pinhole-protocol/src/`
- Server: `pinhole-framework/src/`
- Client: `pinhole-client/src/`
- Example: `todomvc-example/src/`

Last Updated: 2025-10-22
