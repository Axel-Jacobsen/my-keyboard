//! Types for the keyboard.

/// Side of the keyboard
#[derive(Copy, Clone)]
pub enum Side {
    Left,
    Right,
}

/// A key on a keyboard
pub struct Key {
    pub side: Side,
    pub row_id: usize,
    pub col_id: usize,
}
