#![allow(missing_docs)]

use core::{
    convert::Infallible,
    sync::atomic::{AtomicU64, Ordering},
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use embedded_io_async::{ErrorType, Read, Write};
use futures::executor::block_on;
use msrt_embassy_uart::{ReceivedMessage, UartEventHandler, UartTaskError};

mod msrt_uart {
    use super::MockUart;

    msrt_embassy_uart::define_msrt_uart!(MockUart);
}

static NOW_MS: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Default)]
struct MockUart {
    rx: Arc<Mutex<VecDeque<u8>>>,
    tx: Arc<Mutex<VecDeque<u8>>>,
}

#[derive(Default)]
struct App {
    handled_ticks: u32,
}

impl UartEventHandler for App {
    fn handle_message(&mut self, message: ReceivedMessage) {
        match message.channel_id_u8() {
            0 => self.handle_command(message.as_bytes()),
            1 => self.handle_debug_log(message.as_bytes()),
            other => panic!("unexpected MSRT channel: {other}"),
        }
    }

    fn handle_error(&mut self, error: UartTaskError) {
        match error {
            UartTaskError::Adapter(error) => panic!("MSRT UART adapter error: {:?}", error.kind()),
            UartTaskError::SendFailed(failed) => panic!("MSRT UART send failed: {failed:?}"),
        }
    }
}

impl App {
    fn handle_command(&mut self, payload: &[u8]) {
        println!("handle command: {}", core::str::from_utf8(payload).unwrap());
    }

    fn handle_debug_log(&mut self, payload: &[u8]) {
        println!(
            "handle debug log: {}",
            core::str::from_utf8(payload).unwrap()
        );
    }

    fn tick(&mut self) {
        self.handled_ticks += 1;
    }
}

impl ErrorType for MockUart {
    type Error = Infallible;
}

impl Read for MockUart {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut rx = self.rx.lock().expect("mock uart rx mutex poisoned");
        let mut len = 0;

        while len < buf.len() {
            let Some(byte) = rx.pop_front() else {
                break;
            };
            buf[len] = byte;
            len += 1;
        }

        Ok(len)
    }
}

impl Write for MockUart {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut tx = self.tx.lock().expect("mock uart tx mutex poisoned");
        tx.extend(buf.iter().copied());
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {
    block_on(async {
        let uart = MockUart::default();
        let mut app = App::default();
        let mut rx_buf = [0u8; 128];

        msrt_uart::init(uart).expect("MSRT UART link init failed");
        msrt_uart::send_message(b"device ready").expect("send_message failed");
        msrt_uart::debug(b"boot ok").expect("debug failed");

        // On real MCU firmware this task is spawned by Embassy and runs forever:
        // embassy_executor::Spawner::spawn(msrt_task(&mut rx_buf, &mut app)).unwrap();
        // We do not run it here because this example is meant to show the MCU shape
        // without requiring a concrete board executor.
        let _ = (&mut rx_buf, msrt_task);

        app.tick();
        println!(
            "MCU app initialized MSRT UART link; ticks={}",
            app.handled_ticks
        );
    });
}

fn now_ms() -> u64 {
    NOW_MS.fetch_add(1, Ordering::Relaxed)
}

#[allow(dead_code)]
async fn msrt_task(rx_buf: &mut [u8], app: &mut App) {
    msrt_uart::run_task(now_ms, rx_buf, app).await;
}
