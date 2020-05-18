use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
  Load { path: String },
  Action { name: String }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
  Render { document: Document },
  RedirectTo { path: String }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
  Empty,
  Container { children: Vec<Box<Node>> },
  Text { text: String },
  Button { text: String, action: String }

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
