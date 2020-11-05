use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document(pub Node);

impl Document {
    pub fn empty() -> Document {
        Document(Node::Empty)
    }
}
