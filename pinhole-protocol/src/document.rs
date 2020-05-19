use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action { 
  pub name: String, 
  pub args: HashMap<String, String> 
}

impl Action {
  pub fn named(name: impl ToString) -> Action {
    Action::new(name, HashMap::default())
  }

  pub fn new(name: impl ToString, args: HashMap<String, String>) -> Action {
    Action { name: name.to_string(), args }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Scope {
  /// Persisted across restarts
  Persistent,

  /// Cleared when the application is restarted
  Session,

  /// Cleared on navigation
  Local
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormValue {
  String(String),
  Boolean(bool)
}

pub type FormState = HashMap<String, FormValue>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
  Load { path: String, storage: HashMap<String, String> },
  Action { path: String, action: Action, form_state: FormState }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
  Render { document: Document },
  RedirectTo { path: String },
  Store { scope: Scope, key: String, value: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
  Empty,
  Container { children: Vec<Box<Node>> },
  Text { text: String },
  Button { label: String, on_click: Action },
  Checkbox { id: String, label: String, checked: bool, on_change: Action },
  Input { label: String, id: String, password: bool },
}

impl Node {
  pub fn boxed(self) -> Box<Node> {
    Box::new(self)
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document(pub Node);

impl Document {
  pub fn empty() -> Document {
    Document(Node::Empty)
  }
}
