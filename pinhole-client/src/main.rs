#![recursion_limit="1024"]
mod network;
mod system;

use kv_log_macro as log;

use async_std::task;

use imgui::*;

use network::{NetworkSessionEvent, NetworkSession};
use pinhole_protocol::document::{Node, Document, FormValue as RemoteFormValue, FormState as RemoteFormState};
use std::collections::HashMap;


enum LocalFormValue {
  String(ImString),
  Boolean(bool)
}

impl LocalFormValue {
  fn boolean(&mut self) -> &mut bool {
    match self {
      LocalFormValue::Boolean(b) => b,
      _ => {
        *self = LocalFormValue::String(ImString::new(""));
        self.boolean()
      }
    }
  }

  fn string(&mut self) -> &mut ImString {
    match self {
      LocalFormValue::String(s) => s,
      _ => {
        *self = LocalFormValue::String(ImString::new(""));
        self.string()
      }
    }
  }
}

type LocalFormState = HashMap<String, LocalFormValue>;
fn convert_form_state(local_state: &LocalFormState) -> RemoteFormState {
  let mut remote_state: RemoteFormState = HashMap::new();

  for (key, value) in local_state {
    remote_state.insert(key.clone(), match value {
      LocalFormValue::String(s) => RemoteFormValue::String(s.to_string()),
      LocalFormValue::Boolean(b) => RemoteFormValue::Boolean(*b)
    });
  }

  remote_state
}

fn main() {
  femme::with_level(::log::LevelFilter::Debug);

  let address = "127.0.0.1:8080".to_string();

  log::info!("📌 Pinhole starting up...");
  let system = system::init("Pinhole");

  let mut network_session = NetworkSession::new(address);
  task::block_on(network_session.load(&"/".to_string()));

  let mut document = Document(Node::Text { text: "Loading...".to_string() });

  let mut form_state: LocalFormState = HashMap::new();

  system.main_loop(move |_, ui| {

    while let Some(event) = network_session.try_recv() {
      match event {
        NetworkSessionEvent::DocumentUpdated(new_document) => {
          document = new_document;
          log::debug!("Document updated", { document: format!("{:?}", document) });
        },
      }
    }

    let colour_token = ui.push_style_colors(&[
      (StyleColor::WindowBg, [1.0, 1.0, 1.0, 1.0]),
      (StyleColor::Text, [0.0, 0.0, 0.0, 1.0])
    ]);

    let style_var = ui.push_style_vars(&[
      StyleVar::WindowRounding(0.0),
    ]);


    let document = document.clone();

    let window = Window::new(im_str!("Pinhole"))
      .position([0., 0.], Condition::Always)
      .size(ui.window_size(), Condition::Always)
      .draw_background(false)
      .no_decoration();

    if let Some(window) = window.begin(ui) {
      render_node(ui, &mut network_session, &mut form_state, &document.0);

      window.end(ui);
    }
    
    style_var.pop(ui);
    colour_token.pop(ui);
  });
}

fn render_node<'a, 'b>(ui: &'a mut Ui, network_session: &mut NetworkSession, form_state: &mut LocalFormState, node: &'b Node) {
  match node {
    Node::Empty => {},
    Node::Text { text } => ui.text(text),
    Node::Button { label, on_click } => {
      if ui.button(&ImString::from(label.clone()), [100., 30.]) {
        task::block_on(network_session.action(&on_click, &convert_form_state(form_state)));
      }
    },

    Node::Checkbox { id, label, checked, on_change } => {
      let value = form_state.entry(id.clone()).or_insert(LocalFormValue::Boolean(*checked));

      if ui.checkbox(&ImString::from(label.clone()), &mut value.boolean()) {
        task::block_on(network_session.action(&on_change, &convert_form_state(form_state)));
      }
    },

    Node::Container { children } => {
      let group = ui.begin_group();

      for node in children {
        render_node(ui, network_session, form_state, node);
      }

      group.end(ui);
    },

    Node::Input { label, id, password} => {
      let value = form_state.entry(id.clone()).or_insert(LocalFormValue::String(ImString::from("".to_string())));
      
      // imgui puts labels on the right side normally for whatever reason
      // this series of steps places it at the left.
      ui.align_text_to_frame_padding();
      ui.text(label);
      ui.same_line(100.);

      // imgui uses the label to identify a field.
      // the '##' prefix acts like a comment -- the '##' and everything after
      // it is not shown but makes the label unique.
      ui.input_text(&ImString::new(format!("##{}", id)), value.string())
        .resize_buffer(true)
        .password(*password)
        .auto_select_all(true)
        .build();

    }
  }
}