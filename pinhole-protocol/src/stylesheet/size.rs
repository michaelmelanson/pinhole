use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Size {
    Auto,
    Fixed(Length),
    Fill,
}
