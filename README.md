# Pinhole

The web is great, but it was designed around showing hyperlinked documents. What if the web were designed for applications instead?

## Goal

The goal is to explore what would happen if we started from scratch delivery platform designed around applications, taking the best parts of the Web.

## Design features

### Protocol

Pinhole maintains a persistent TCP connection to allow for bidirectional messaging. The protocol is designed so that this connection can be reconnected at any time without losing state.

The protocol is transported by length-prefixed [CBOR (Concise Binary Object Representation)](https://tools.ietf.org/html/rfc7049) messages.

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

(This part is mostly incomplete)

### View layer

Pinhole's client uses [Dear Imgui](https://github.com/ocornut/imgui) for rendering its views.
