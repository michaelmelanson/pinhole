use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum FontWeight {
    Normal,
    Thin,
    ExtraLight,
    Light,
    Medium,
    Bold,
    ExtraBold,
    Black,
}
