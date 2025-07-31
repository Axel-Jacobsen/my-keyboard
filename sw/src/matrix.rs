//! Watches the keyboard matrix for key presses and reports them.

use embassy_rp::gpio::{Input, Output};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Sender;
use embassy_time::{Duration, Timer};

use crate::types::{Key, Side};

const ROW_DELAY: Duration = Duration::from_hz(1000);

pub async fn report(
    rows: &mut [Output<'_>; 4],
    cols: &mut [Input<'_>; 8],
    side: Side,
    reporter: Sender<'static, ThreadModeRawMutex, Key, 64>,
) {
    loop {
        for (row_id, row) in rows.iter_mut().enumerate() {
            row.set_high();
            Timer::after(ROW_DELAY).await;

            for (col_id, col) in cols.iter().enumerate() {
                if col.is_high() {
                    log::debug!("col {} is high", col_id);
                    let key = Key {
                        side,
                        row_id,
                        col_id,
                    };
                    reporter.send(key).await;
                }
            }

            row.set_low();
        }
    }
}
