#![no_std]
#![no_main]

mod matrix;
mod types;

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::{Duration, Timer};
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

        static DEVICE_HANDLER: StaticCell<MyDeviceHandler> = StaticCell::new();
        builder.handler(DEVICE_HANDLER.init(MyDeviceHandler::new()));
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

    let mut usb = builder.build();
    let usb_fut = usb.run();

    let mut signal_pin = Input::new(p.PIN_7, Pull::None);
    let _row_pin = Output::new(p.PIN_15, Level::High);

    signal_pin.set_schmitt(false);

    let (reader, mut writer) = hid.split();

    let in_fut = async {
        const RELEASE: KeyboardReport = KeyboardReport {
            keycodes: [0; 6],
            leds: 0,
            modifier: 0,
            reserved: 0,
        };

        loop {
            signal_pin.wait_for_rising_edge().await;
            let press = KeyboardReport {
                keycodes: [5, 0, 0, 0, 0, 0],
                ..RELEASE
            };

            let _ = writer.write_serialize(&press).await;
            Timer::after(Duration::from_millis(1)).await;
            let _ = writer.write_serialize(&RELEASE).await;
        }
    };

    let mut request_handler = MyRequestHandler {};
    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    join(usb_fut, join(in_fut, out_fut)).await;
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
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

struct MyDeviceHandler {
    configured: AtomicBool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for MyDeviceHandler {
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
