use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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
