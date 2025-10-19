use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Colour {
    RGBA(f32, f32, f32, f32), // Each component is in the range [0.0, 1.0]
}
