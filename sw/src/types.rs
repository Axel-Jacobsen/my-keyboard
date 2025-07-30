//! Types for the keyboard.

/// Side of the keyboard
pub enum Side {
    Left,
    Right,
}

/// A key on a keyboard
pub struct Key {
    side: Side,
    row: u8,
    col: u8,
}
