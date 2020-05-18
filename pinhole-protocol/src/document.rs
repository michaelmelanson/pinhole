use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
  Load(String)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
  UpdateDocument(Document)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
  Empty,
  Text(String),
  Container(Vec<Box<Node>>)
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
