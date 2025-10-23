use serde::{Deserialize, Serialize};

use crate::{node::Node, stylesheet::Stylesheet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub node: Node,
    pub stylesheet: Stylesheet,
}

impl Document {
    pub fn empty() -> Document {
        Document {
            node: Node::Empty,
            stylesheet: Stylesheet::default(),
        }
    }
}
