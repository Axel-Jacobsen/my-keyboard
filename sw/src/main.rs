#![no_std]
#![no_main]

mod matrix;
mod types;

use crate::types::{Key, Side};

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::Spawner;
use embassy_futures::join::join5;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcState};
use embassy_usb::class::hid::{HidReaderWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Config, Handler};
use static_cell::StaticCell;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[defmt::panic_handler]
fn panic() -> ! {
    defmt::panic!()
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);

    let config = {
        let mut config = Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("HID keyboard example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config
    };

    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let mut builder = Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            MSOS_DESCRIPTOR.init([0; 256]),
            CONTROL_BUF.init([0; 64]),
        );

        static DEVICE_HANDLER: StaticCell<StubDeviceHandler> = StaticCell::new();
        builder.handler(DEVICE_HANDLER.init(StubDeviceHandler::new()));
        builder
    };

    let config = embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 10,
        max_packet_size: 64,
    };

    let hid = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        HidReaderWriter::<_, 1, 8>::new(&mut builder, state, config)
    };

    static LEFT_ROWS: StaticCell<[Output; 4]> = StaticCell::new();
    let left_rows = LEFT_ROWS.init([
        Output::new(p.PIN_15, Level::Low),
        Output::new(p.PIN_14, Level::Low),
        Output::new(p.PIN_13, Level::Low),
        Output::new(p.PIN_12, Level::Low),
    ]);
    static LEFT_COLS: StaticCell<[Input; 8]> = StaticCell::new();
    let left_cols = LEFT_COLS.init([
        Input::new(p.PIN_0, Pull::None),
        Input::new(p.PIN_1, Pull::None),
        Input::new(p.PIN_2, Pull::None),
        Input::new(p.PIN_3, Pull::None),
        Input::new(p.PIN_4, Pull::None),
        Input::new(p.PIN_5, Pull::None),
        Input::new(p.PIN_6, Pull::None),
        Input::new(p.PIN_7, Pull::None),
    ]);

    static REPORT_CHANNEL: StaticCell<Channel<ThreadModeRawMutex, Key, 64>> = StaticCell::new();
    let reporting_channel = REPORT_CHANNEL.init(Channel::new());

    let left_matrix_fut =
        matrix::report(left_rows, left_cols, Side::Left, reporting_channel.sender());

    let key_receiver = reporting_channel.receiver();

    let (reader, mut writer) = hid.split();
    let in_fut = async {
        const RELEASE: KeyboardReport = KeyboardReport {
            keycodes: [0; 6],
            leds: 0,
            modifier: 0,
            reserved: 0,
        };

        loop {
            log::debug!("waiting for key");
            let key = key_receiver.receive().await;
            log::debug!("received key");

            // Send 'L' or 'R'
            let _ = writer
                .write(&[match key.side {
                    Side::Left => b'L',
                    Side::Right => b'R',
                }])
                .await;

            // Send ASCII digits of row_id
            let _ = writer.write(&[b'0' + (key.row_id / 10) as u8]).await;
            let _ = writer.write(&[b'0' + (key.row_id % 10) as u8]).await;

            // Send ASCII digits of col_id
            let _ = writer.write(&[b'0' + (key.col_id / 10) as u8]).await;
            let _ = writer.write(&[b'0' + (key.col_id % 10) as u8]).await;

            // Optional: newline for readability
            let _ = writer.write(&[b'\n']).await;

            let _ = writer.write_serialize(&RELEASE).await;
        }
    };

    let mut request_handler = StubRequestHandler {};
    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    static LOGGER_STATE: StaticCell<CdcState> = StaticCell::new();
    let logger_class = CdcAcmClass::new(&mut builder, LOGGER_STATE.init(CdcState::new()), 64);
    let log_fut = embassy_usb_logger::with_class!(1024, log::LevelFilter::Debug, logger_class);

    let mut usb = builder.build();
    let usb_fut = usb.run();

    join5(log_fut, left_matrix_fut, usb_fut, in_fut, out_fut).await;
}

struct StubRequestHandler {}

impl RequestHandler for StubRequestHandler {
    fn get_report(&mut self, _id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        None
    }

    fn set_report(&mut self, _id: ReportId, _data: &[u8]) -> OutResponse {
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, _id: Option<ReportId>, _dur: u32) {}

    fn get_idle_ms(&mut self, _id: Option<ReportId>) -> Option<u32> {
        None
    }
}

struct StubDeviceHandler {
    configured: AtomicBool,
}

impl StubDeviceHandler {
    fn new() -> Self {
        StubDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for StubDeviceHandler {
    fn enabled(&mut self, _enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
    }

    fn addressed(&mut self, _addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
    }
}
