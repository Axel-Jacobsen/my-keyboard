//! Types for the keyboard.

/// Side of the keyboard
#[derive(Copy, Clone, Debug)]
pub enum Side {
    Left,
    Right,
}

/// A key on a keyboard
#[derive(Debug)]
pub struct Key {
    pub side: Side,
    pub row_id: usize,
    pub col_id: usize,
}
