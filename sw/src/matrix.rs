//! Watches the keyboard matrix for key presses and reports them.

use embassy_rp::gpio::Pin;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Sender;
use embassy_time::{Duration, Timer};

use crate::types::{Key, Side};

pub struct Matrix<'a, P>
where
    P: Pin,
{
    rows: &'a [P],
    cols: &'a [P],
    side: Side,
    reporter: Sender<'static, ThreadModeRawMutex, Key, 64>,
}

impl<'a, P> Matrix<'a, P>
where
    P: Pin,
{
    pub fn new(
        rows: &'a [P],
        cols: &'a [P],
        side: Side,
        reporter: Sender<'static, ThreadModeRawMutex, Key, 64>,
    ) -> Self {
        Self {
            rows,
            cols,
            side,
            reporter,
        }
    }
}
