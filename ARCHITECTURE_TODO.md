# Pinhole Architecture - TODO

Architectural improvements identified in October 2025 assessment.

## Priority 0 - Critical

- [x] **Protocol Versioning**
  - Implemented capability-based versioning with URI scheme
  - ClientHello/ServerHello handshake for capability negotiation
  - Can renegotiate capabilities at any time during connection
  - Context.assert_capability() for runtime enforcement
  - Location: `pinhole-protocol/src/capabilities.rs`, `messages.rs`

- [x] **Logging and Observability**
  - Migrated from `log` to `tracing` throughout codebase
  - Added structured logging with key-value fields
  - Added tracing spans for connections and message processing
  - Tracks messages_processed per connection
  - Location: All modules, primary in `pinhole-framework/src/lib.rs`

## Priority 1 - Important

- [x] **Error Handling Granularity**
  - Added `Unauthorized` (401), `Forbidden` (403), `TooManyRequests` (429) to `ErrorCode`
  - Added `ErrorCode::is_retryable()` and an HTTP-style `Display` impl (e.g. "429 Too Many Requests")
  - Client now auto-retries the in-flight request with a fixed backoff on a retryable error, instead of surfacing it to the UI
  - Location: `pinhole-protocol/src/messages.rs`, `pinhole-client/src/network.rs`

- [x] **Route Matching**
  - Implemented path parameter extraction with `:param` syntax (`/resources/:id/subpage`)
  - RoutePattern matches paths and extracts parameters into Params HashMap
  - Updated Route trait to accept Params in action() and render() methods
  - Updated Application::route() to return (Route, Params) tuple
  - Comprehensive test coverage (6 router tests)
  - Location: `pinhole-framework/src/router.rs`, `application.rs`, `route.rs`

- [x] **Storage Type System**
  - Added Number, Array, Object types with JSON-compatible semantics
  - Support nested structures with accessor methods
  - From implementations for convenient conversions
  - Comprehensive test coverage
  - Location: `pinhole-protocol/src/storage.rs`

- [x] **State Synchronisation**
  - Added per-key `StateVersion`/`VersionMap` alongside storage on the wire
  - `Context::store()` computes a strictly increasing version per key from
    what the client sent; `StorageManager::apply_remote_store()` drops any
    `Store` whose version isn't strictly greater than what's already known,
    guarding against the double-action race (two in-flight requests built
    from the same stale snapshot)
  - `Route`/`Context.storage` API is unchanged — `StateMap` stays plain,
    versions are wire/StorageManager-only metadata
  - Location: `pinhole-protocol/src/storage.rs`, `messages.rs`,
    `pinhole-client/src/storage.rs`, `network.rs`,
    `pinhole-framework/src/context.rs`, `lib.rs`

## Priority 2 - Nice to Have

- [x] **Stylesheet Completeness**
  - Added `AlignItems` (cross-axis child alignment within a Row/Column,
    via iced's `align_x`/`align_y`) and `FlexGrow` (per-child proportional
    growth, via `Length::FillPortion`, resolved from the child's own
    classes when a Container assembles its children)
  - `justify-content` deliberately not added as a separate rule —
    `AlignChildrenX`/`AlignChildrenY` already fill that role (positioning
    the Row/Column block within its parent Container); iced has no native
    `space-between`-style distribution to add beyond that
  - Added `:hover`/`:focus` support: `StylesheetClass` gains
    `hover_rules`/`focus_rules`, layered on top of base rules when a
    `Button`/`TextInput`'s iced `Status` indicates that state (computed
    inside their style closures now, not just once outside)
  - Added `@media`-style responsive rules: `StylesheetClass.media_rules`
    (min/max width/height), evaluated against the client's actual window
    size, plumbed from a new `iced::window::resize_events()` subscription
    in `pinhole-client/src/main.rs` down through `UiNode::view()`
  - `ComputedStyle::compute()` is unchanged (base rules only);
    `compute_with_context()` is the new richer entry point layering
    base < media < hover < focus
  - Location: `pinhole-protocol/src/stylesheet/*.rs`,
    `pinhole-client/src/stylesheet.rs` + `stylesheet/*.rs`,
    `pinhole-client/src/ui_node.rs`, `main.rs`

- [x] **Health Check Protocol**
  - Made part of the protocol itself rather than an out-of-band HTTP
    prober, since every connection already requires a full TLS handshake
    anyway: `ClientToServerMessage::HealthCheck` /
    `ServerToClientMessage::HealthCheckResult { healthy: bool }`, usable at
    any time (not gated on capability negotiation)
  - `healthy` reflects a new `ServerHealth` (shared `Arc<AtomicBool>`), so
    a client polling on an already-open connection learns the server is
    draining before it actually exits
  - Location: `pinhole-protocol/src/messages.rs`, `capabilities.rs`,
    `pinhole-framework/src/health.rs` (new)

- [x] **Graceful Shutdown**
  - `accept_loop` now `select!`s between `listener.accept()` and a
    `SIGINT`/`SIGTERM` future; on signal it stops accepting, marks
    `ServerHealth` draining, and awaits in-flight connections (tracked via
    `tokio::task::JoinSet`) up to `GRACEFUL_SHUTDOWN_TIMEOUT` (30s) before
    aborting stragglers
  - Verified live: SIGTERM to a running server correctly drained an open
    connection and exited cleanly only after it closed
  - Location: `pinhole-framework/src/lib.rs`

- [x] **Nix Packaging**
  - Added `packages.<system>.*` flake outputs for all three binaries
    (`pinhole-client`, `pinhole-todomvc-example`,
    `pinhole-styling-showcase-example`), built once as a shared workspace
    derivation and extracted per-binary to avoid rebuilding shared deps
  - `pinhole-client` (GUI) is wrapped with its Vulkan/Wayland/X11
    `LD_LIBRARY_PATH` on Linux so it runs standalone outside the devShell;
    on macOS it's built as a proper `Pinhole.app` bundle instead (using the
    `apple-sdk` package, since per-framework `darwin.apple_sdk.frameworks.*`
    packages have been removed from nixpkgs) — **still unverified on real
    macOS hardware**; the new `build-macos` CI job (below) is meant to
    close that gap once its prerequisite Namespace setup is done
  - `x86_64-darwin` dropped from supported systems — the pinned nixpkgs
    channel no longer supports it
  - Verified live on Linux: all three packages build and run (including a
    full SIGTERM graceful-shutdown cycle on the packaged server binary)
  - Location: `flake.nix`

- [x] **GitHub Actions CI**
  - `.github/workflows/ci.yml`: four jobs, all via `nix develop --command
    cargo ...` / `nix build` so CI uses the exact same dependency set as
    local dev (no separate apt package list to drift out of sync):
    - `check` / `build`: fmt-check + build + test + clippy, and `nix
      build` of all three packages, on `namespace-profile-default`
    - `check-macos` / `build-macos`: same split, on
      `namespace-profile-macos` — `build-macos` is what actually exercises
      the `pinhole-client` `.app` bundle on real Apple hardware
  - Both Namespace profiles require one-time manual setup only the repo
    owner can do: connect the GitHub org at
    https://cloud.namespace.so/workspace/ghrunners and create runner
    profiles named exactly `default` and `macos` there (profile OS/arch is
    set in Namespace's dashboard, not from the workflow)
  - Fixed 3 pre-existing files whose committed formatting disagreed with
    the current nixpkgs rustfmt pin, since otherwise the new fmt-check step
    would fail immediately for reasons unrelated to any real change
  - Verified structurally with `actionlint` (zero findings, with
    `.github/actionlint.yaml` allow-listing the two custom labels) and by
    running `check`/`build` locally end-to-end with `act`, which surfaced
    two environment-specific false positives rather than real problems:
    `test_readonly_filesystem` failed because `act`'s default image runs
    as root (confirmed via `docker run ... id` → `uid=0`), and root
    ignores the read-only permission bits the test relies on; separately,
    `nix build` compiled and packaged every binary successfully but then
    failed Nix's own sandbox purity check (`/homeless-shelter`), which is
    a known limitation of running Nix's sandbox (needs Linux user
    namespaces) inside `act`'s restricted container — confirmed directly
    via `unshare --user` failing in the same container even with
    `--cap-add=SYS_ADMIN`. Real GitHub-hosted/Namespace runners use a
    non-root user and full VM-level namespace support, so neither should
    reproduce there. Stronger evidence the packaging itself is correct:
    genuine (non-containerized) `nix build` of all three packages on this
    machine succeeded and the binaries ran correctly
  - `check-macos`/`build-macos` are not runnable via `act` at all (no
    macOS runner support), so their first real confirmation will be an
    actual push once the Namespace `macos` profile is in place
  - Location: `.github/workflows/ci.yml`, `.github/actionlint.yaml`

- [ ] **Build and Deployment Tooling**
  - Add `Dockerfile` for reproducible builds

- [ ] **Improved client**
  - Application selector, based on list of known applications.
  - Flow for adding applications
  - Health check pip

- [ ] **Testing Gaps**
  - Add `proptest` for fuzzing message serialisation
  - Add load tests (1000 concurrent connections)
  - Add chaos tests (random disconnects, partial messages)

- [ ] **Performance Optimisations**
  - Use `HashMap` or trie for route lookup
  - Implement message pooling with `bytes::BytesMut`
  - Batch storage writes (flush every 100ms)
  - Cache serialised documents with ETag-like versioning (e.g. Rails style freshness declarations)
  - Add benchmarks to track regressions

## Priority 3 - Future

- [ ] **Documentation**
  - Add module-level docs to all public modules
  - Add doc comments to all public APIs
  - Create architecture decision records (ADRs)
  - Write deployment guide and security considerations

- [ ] **Incremental Rendering** _(currently an anti-plan)_
  - Add patch messages for partial updates
  - Consider JSON Patch (RFC 6902)
  - Only if design goals change from full-page model
