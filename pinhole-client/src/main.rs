#![recursion_limit="1024"]
mod network;
mod system;

use async_std::task;

use imgui::*;

use network::{NetworkSessionEvent, NetworkSession};
use pinhole_protocol::document::{Node, Document};


fn main() {
  let system = system::init("Pinhole");

  let mut network_session = NetworkSession::new("127.0.0.1:8080".to_string());
  task::block_on(network_session.load(&"/".to_string()));

  let mut document = Document(Node::Text { text: "Loading...".to_string() });

  system.main_loop(move |_, ui| {

    while let Some(event) = network_session.try_recv() {
      match event {
        NetworkSessionEvent::DocumentUpdated(new_document) => {
          document = new_document;
          println!("Document updated: {:?}", document);
        },
      }
    }

    let colour_token = ui.push_style_colors(&[
      (StyleColor::WindowBg, [1.0, 1.0, 1.0, 1.0]),
      (StyleColor::Text, [0.0, 0.0, 0.0, 1.0])
    ]);

    let style_var = ui.push_style_vars(&[
      StyleVar::WindowRounding(0.0)
    ]);

    let document = document.clone();
    
    let window = Window::new(im_str!("Pinhole"))
      .position([0., 0.], Condition::Always)
      .size(ui.window_size(), Condition::Always)
      .draw_background(false)
      .no_decoration();

    if let Some(window) = window.begin(ui) {
      render_node(ui, &mut network_session, &document.0);

      window.end(ui);
    }
    
    style_var.pop(ui);
    colour_token.pop(ui);
  });
}


fn render_node<'a, 'b>(ui: &'a mut Ui, network_session: &mut NetworkSession, node: &'b Node) {
  match node {
    Node::Empty => {},
    Node::Text { text } => ui.text(text),
    Node::Button { text, on_click } => {
      if ui.button(&ImString::from(text.clone()), [100., 30.]) {
        task::block_on(network_session.action(&on_click))
      }
    },
    Node::Container { children } => {
      for node in children {
        render_node(ui, network_session, node);
      }
    }
  }
}