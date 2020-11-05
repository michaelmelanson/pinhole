# Pinhole

The web is great, but it was designed around showing hyperlinked documents. What if the web technologies were designed a) with the benefit of hindsight, and b) primarily for applications instead of documents?

## Status

Set your expectations very low, and then move them lower. This is just a proof of concept.

## Usage

You'll need a Rust development environment. See the [Getting Started instructions](https://www.rust-lang.org/learn/get-started) on Rust website for details about setting that up.


### Starting a server

To run the TodoMVC server in a terminal:

```
cargo run --bin pinhole-todomvc-example
```

Currently it's hardcoded to always listens on port 8080.

### Starting the client

In a separate terminal, run the Pinhole Client:

```
cargo run --bin pinhole-client
```

Currently it's hardcoded to connect to a server on `0.0.0.0:8080`, so it should now connect to your server.

### What you'll see

When the client connects, it will first show a login page. Entering an email and password then clicking Sign In will send you to a list page (authentication is faked, but you will see in the terminal that the server receives the information you enter). 

On the list page you will see a couple todo items. You can click their checkboxes and an action will be sent to the server. It doesn't currently persist the changes.

### Development environment

If you install Cargo Watch with `cargo install cargo-watch`, then you can start a hot reloading server like this:

```
cargo watch -x 'run --bin pinhole-todomvc-example'
```

Now you can leave that running in a terminal. It will watch for code changes, and recompile and restart your server as necessary.

## Goal

The goal is to explore what would happen if we took the best ideas out of the Web as a delivery platform, then started from scratch on a platform for delivering applications rather than documents.

## Design features

### Protocol

Pinhole maintains a persistent TCP connection to allow for bidirectional messaging. It does not have a request-response cycle: instead, either the client or the server can message each other at any time. The server can, for example, send multiple view updates as loading progresses or in response to server-side events.

The protocol is designed so that all state is maintained client-side so that this connection can be terminated and reconnected at any time with minimal user impact, and so that the server is compatible with load balancers without needing sticky sessions. 

The messages are transported by length-prefixed [CBOR (Concise Binary Object Representation)](https://tools.ietf.org/html/rfc7049) datagrams. This was chosen because it has flexible, JSON-like semantics but it's compact and fast to generate parse.

#### Client-to-server messages

* **`Load`:** Request that the server send the UI state for a new URL. The server should then start processing that route and respond with a message such as **`Render`** to update the display, or **`RedirectTo`** to send the client to yet another URL. A **`Load`** message is sent whenever a client reconnects.
* **`Action`:** Notify the server that an action has taken place, such as a button being clicked or other form element being changed.

#### Server-to-client messages

* **`Render`:** Tell the client to update its display to show a new document.
* **`RedirectTo`:** Request that the client switch to a new URL. The client will respond with a **`Load`** message for the new URL. The _current URL_ is persisted client-side.
* **`Store`:** Tell the client to update its storage with a key-value pair.

### Actions

Some view components, such as buttons and input fields, have events for when they are clicked or modified. When these events occur, the client will send an **`Action`** message to the server.

Actions are used in Pinhole whenever you would use a POST, PUT, PATCH, DELETE request in HTTP. URL navigations are used whenever you would use a GET request in HTTP.

### State management

#### Connection state

The client keeps the following state locally about its connection:

* Current URL
* Latest document to be rendered.

#### Storage

The storage system is modelled after the Cookie system in HTTP where keys are written by the server, stored on the client, and sent back to the server in requests. But it's significantly improved to fix some of the problems with cookies, and to also fill in the use cases for persistent cookies, session cookies, local storage, and form state.

It's currently only partly done, but when completed:

* Data can be stored in one of three scopes: persistent (saved across app restarts), session (cleared after app restart), or local (cleared on page navigation). Only session storage is currently implemented.
* Form elements persist their values in storage at the 'local' scope.
* Stored data is sent to the server in **`Action`** messages. Actions can choose exactly which a set of they want sent, to avoid the problems HTTP has with cookie bloat (not yet implemented).

### View layer

Pinhole's client uses [Iced](https://github.com/hecrj/iced) for rendering its views. When the client receives a **`Render`** message, it updates its _current document_ and from then on renders that document on each frame.

## Roadmap and future plans

### Roadmap
* Add Transport level security (TLS) to get HTTPS-like encryption and security.
* Finish implementing the storage system.
* Figure out how storage data should be sent on page navigations. The way it works for actions is great, and should work similarly for navigations where only the keys the server cares about should be sent. But how should the client find out about this?
* Add more node types -- media, grouping, links (then again, we have buttons so maybe HTML-like links aren't necessary?).
* Add a style system.
* Add UI chrome -- a navigation bar? status bar?

### Ideas and open questions
* Embed an extension language so servers can be written in e.g. Javascript via Deno.
* Client-side action handlers by shipping Javascript bundles.
* Polling? Server asks client to refresh page at some point in the future.
* Subscriptions. Rough sketch: Server sends client a subscription list, which client then subscribes to. When events occur server-side on one of these channels, server asks client to refresh.

### Anti-plans
These are ideas that I specifically don't plan on implemementing.

* A React-like component model. I think any encapsulation like that can be done server-side on top of the existing model.
* Incremental page updates. Currently the whole-page update model is very simple and works a lot like Turbolinks, which is good enough for almost any page.
