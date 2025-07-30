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

impl Key {
    pub fn to_ascii(&self) -> heapless::String<64> {
        use core::fmt::Write;
        let mut s = heapless::String::new();
        let _ = write!(
            &mut s,
            "{}{:02}-{:02}\n",
            match self.side {
                Side::Left => "L",
                Side::Right => "R",
            },
            self.row_id,
            self.col_id
        );
        s
    }
}
