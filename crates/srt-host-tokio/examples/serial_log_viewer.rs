use std::{env, str, time::Instant};

use msrt_host_tokio::{ErrorKind, HostAdapter, HostEventHandler, HostTaskError, ReceivedMessage};
use tokio::time::{Duration, sleep};
use tokio_serial::SerialPortBuilderExt;

const LOG_CHANNEL: u8 = 1;
const RECONNECT_DELAY: Duration = Duration::from_secs(1);

struct LogPrinter {
    disconnected: bool,
}

impl LogPrinter {
    const fn new() -> Self {
        Self {
            disconnected: false,
        }
    }
}

impl HostEventHandler for LogPrinter {
    fn handle_message(&mut self, message: ReceivedMessage) {
        match message.channel_id_u8() {
            LOG_CHANNEL => print_log(message),
            channel => println!(
                "message channel={channel} message_id={} bytes={:02x?}",
                message.message_id(),
                message.as_bytes()
            ),
        }
    }

    fn handle_error(&mut self, error: HostTaskError) {
        match error {
            HostTaskError::Adapter(error)
                if matches!(
                    error.kind(),
                    ErrorKind::IoRead | ErrorKind::IoWrite | ErrorKind::IoFlush
                ) =>
            {
                eprintln!(
                    "serial link disconnected: {:?}; reconnecting ...",
                    error.kind()
                );
                self.disconnected = true;
            }
            HostTaskError::Adapter(error) => {
                eprintln!("host adapter error: {:?}", error.kind());
            }
            HostTaskError::SendFailed(failed) => {
                eprintln!("host send failed: {failed:?}");
            }
        }
    }
}

fn print_log(message: ReceivedMessage) {
    match str::from_utf8(message.as_bytes()) {
        Ok(text) => println!("[mcu log #{}] {text}", message.message_id()),
        Err(_) => println!(
            "[mcu log #{}] <non-utf8> {:02x?}",
            message.message_id(),
            message.as_bytes()
        ),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = env::args().skip(1);
    let port_path = args
        .next()
        .unwrap_or_else(|| "/dev/cu.usbmodem2103".to_owned());
    let baud_rate = args
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(115_200);

    loop {
        match run_session(&port_path, baud_rate).await {
            Ok(()) => {}
            Err(error) => eprintln!("failed to open {port_path}: {error}; retrying ..."),
        }

        sleep(RECONNECT_DELAY).await;
    }
}

async fn run_session(port_path: &str, baud_rate: u32) -> std::io::Result<()> {
    println!("opening {port_path} at {baud_rate} baud");

    let serial = tokio_serial::new(port_path, baud_rate).open_native_async()?;
    let mut adapter = HostAdapter::new(serial);
    let mut handler = LogPrinter::new();
    let mut rx_buf = [0u8; 256];
    let started = Instant::now();

    while !handler.disconnected {
        let now_ms = started.elapsed().as_millis() as u64;
        adapter
            .poll_once_dispatch(now_ms, &mut rx_buf, &mut handler)
            .await;
    }

    Ok(())
}
