# Pinhole

The web is great, but it was designed around showing hyperlinked documents. What if the web technologies were designed a) with the benefit of hindsight, and b) primarily for applications instead of documents?

## Goal

The goal is to explore what would happen if we started from scratch delivery platform designed around applications, taking the best parts of the Web.

## Design features

### Protocol

Pinhole maintains a persistent TCP connection to allow for bidirectional messaging. The protocol is designed so that this connection can be terminated and reconnected at any time without losing state.

The messages are transported by length-prefixed [CBOR (Concise Binary Object Representation)](https://tools.ietf.org/html/rfc7049) datagrams. This was chosen because it has flexible, JSON-like semantics but it's compact and fast to generate parse.

#### Client-to-server messages

* **`Load`:** Request that the server send the UI state for a 
* **`Action`:** Notify the server that an action has taken place, such as a button being clicked or other form element being changed.

#### Server-to-client messages

* **`Render`:** Tell the client to update its display to show a new document.
* **`RedirectTo`:** Request that the client switch to a new URL. The client will respond with a **`Load`** message for the new URL.
* **`Store`:** Tell the client to update its storage with a key-value pair.

### Actions

Some view components, such as buttons and input fields, have events for when they are clicked or modified. When these events occur, the client will send an **`Action`** message to the server.

### State management

#### Connection state

The client keeps the following state locally about its connection:

* Current URL
* Latest document to be rendered.

#### Storage

The storage system is modelled after the Cookie system in HTTP, where keys are written by the server and stored on the client. But it's significantly improved to fix some of the problems with cookies, and to also capture the use cases for persistent cookies, session cookies, local storage, and form state.

It's currently only partly done, but when completed:

* Data can be stored in one of three scopes: persistent (saved across app restarts), session (cleared after app restart), or local (cleared on page navigation). Only session storage is currently implemented.
* Form elements persist their values in storage at the 'local' scope.
* Stored data is sent to the server in actions. Actions can choose which a set of they want sent, to avoid the problems HTTP has with cookie bloat (not yet implemented).

### View layer

Pinhole's client uses [Dear Imgui](https://github.com/ocornut/imgui) for rendering its views. When the client receives a **`Render`** message, it updates its _current document_ and from then on renders that document on each frame.

## Future plans

* Transport level security (TLS) to get HTTPS-like encryption and security.
* Embed an extension language so servers can be written in e.g. Javascript?
* Consider a different view layer? Dear Imgui is interesting but it's not as powerful as HTML / CSS.
