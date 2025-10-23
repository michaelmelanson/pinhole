use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Layout {
    pub horizontal: Sizing,
    pub vertical: Sizing,
}

impl Layout {
    pub const fn horizontal(&self, horizontal: Sizing) -> Layout {
        Layout {
            horizontal,
            ..*self
        }
    }

    pub const fn vertical(&self, vertical: Sizing) -> Layout {
        Layout { vertical, ..*self }
    }

    pub const fn centred(&self) -> Layout {
        Layout {
            horizontal: Sizing {
                position: Position::Centre,
                ..self.horizontal
            },
            vertical: Sizing {
                position: Position::Centre,
                ..self.vertical
            },
            ..*self
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Sizing {
    pub position: Position,
    pub size: Size,
}

impl Sizing {
    pub const fn centred(&self) -> Self {
        Sizing {
            position: Position::Centre,
            ..*self
        }
    }

    pub const fn size(&self, size: Size) -> Self {
        Sizing { size, ..*self }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Position {
    Start,
    End,
    Centre,
}

impl Default for Position {
    fn default() -> Self {
        Position::Start
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Size {
    Auto,
    Fixed(u16),
    Fill,
}

impl Default for Size {
    fn default() -> Self {
        Size::Fill
    }
}
