use std::{
    collections::VecDeque,
    env, io,
    pin::Pin,
    str,
    task::{Context, Poll},
    time::Instant,
};

use msrt_host_tokio::{HostAdapter, HostEventHandler, HostTaskError, ReceivedMessage};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::time::{Duration, sleep};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

const LOG_CHANNEL: u8 = 1;
const RECONNECT_DELAY: Duration = Duration::from_secs(1);
const SEND_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug)]
struct NoiseConfig {
    rx_drop_ppm: u32,
    rx_flip_ppm: u32,
    tx_drop_ppm: u32,
    tx_flip_ppm: u32,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            rx_drop_ppm: 0,
            rx_flip_ppm: 0,
            tx_drop_ppm: 0,
            tx_flip_ppm: 0,
        }
    }
}

struct NoisySerial {
    inner: SerialStream,
    config: NoiseConfig,
    rng: Lcg,
    rx_pending: VecDeque<u8>,
    tx_pending: Vec<u8>,
    tx_pos: usize,
    tx_original_len: usize,
}

impl NoisySerial {
    fn new(inner: SerialStream, config: NoiseConfig) -> Self {
        Self {
            inner,
            config,
            rng: Lcg::new(0x51_52_54_31),
            rx_pending: VecDeque::new(),
            tx_pending: Vec::new(),
            tx_pos: 0,
            tx_original_len: 0,
        }
    }

    fn transform_byte(&mut self, direction: Direction, byte: u8) -> Option<u8> {
        let (drop_ppm, flip_ppm) = match direction {
            Direction::Rx => (self.config.rx_drop_ppm, self.config.rx_flip_ppm),
            Direction::Tx => (self.config.tx_drop_ppm, self.config.tx_flip_ppm),
        };

        if self.rng.hits(drop_ppm) {
            println!("[noise {direction}] drop byte=0x{byte:02x}");
            return None;
        }

        let mut out = byte;
        if self.rng.hits(flip_ppm) {
            let bit = (self.rng.next_u32() % 8) as u8;
            out ^= 1 << bit;
            println!("[noise {direction}] flip bit={bit} 0x{byte:02x}->0x{out:02x}");
        }

        Some(out)
    }

    fn fill_read_buf(&mut self, buf: &mut ReadBuf<'_>) {
        while buf.remaining() > 0 {
            let Some(byte) = self.rx_pending.pop_front() else {
                break;
            };
            buf.put_slice(&[byte]);
        }
    }

    fn prepare_write(&mut self, bytes: &[u8]) {
        self.tx_pending.clear();
        self.tx_pos = 0;
        self.tx_original_len = bytes.len();

        for byte in bytes {
            if let Some(out) = self.transform_byte(Direction::Tx, *byte) {
                self.tx_pending.push(out);
            }
        }
    }

    fn poll_pending_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<usize>> {
        while self.tx_pos < self.tx_pending.len() {
            let pos = self.tx_pos;
            let chunk: Vec<u8> = self.tx_pending[pos..].to_vec();
            match Pin::new(&mut self.inner).poll_write(cx, &chunk) {
                Poll::Ready(Ok(0)) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "serial write returned zero",
                    )));
                }
                Poll::Ready(Ok(n)) => self.tx_pos += n,
                Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
                Poll::Pending => return Poll::Pending,
            }
        }

        let original_len = self.tx_original_len;
        self.tx_pending.clear();
        self.tx_pos = 0;
        self.tx_original_len = 0;
        Poll::Ready(Ok(original_len))
    }
}

impl AsyncRead for NoisySerial {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.fill_read_buf(buf);
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }

        let mut raw = [0u8; 128];
        let mut raw_buf = ReadBuf::new(&mut raw);
        match Pin::new(&mut self.inner).poll_read(cx, &mut raw_buf) {
            Poll::Ready(Ok(())) => {
                for byte in raw_buf.filled() {
                    if let Some(out) = self.transform_byte(Direction::Rx, *byte) {
                        self.rx_pending.push_back(out);
                    }
                }
                self.fill_read_buf(buf);
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for NoisySerial {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.tx_pending.is_empty() {
            self.prepare_write(buf);
        }

        self.poll_pending_write(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if !self.tx_pending.is_empty() {
            match self.as_mut().poll_pending_write(cx) {
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
                Poll::Pending => return Poll::Pending,
            }
        }

        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.as_mut().poll_flush(cx) {
            Poll::Ready(Ok(())) => Pin::new(&mut self.inner).poll_shutdown(cx),
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Rx,
    Tx,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rx => f.write_str("rx"),
            Self::Tx => f.write_str("tx"),
        }
    }
}

struct Lcg(u32);

impl Lcg {
    const fn new(seed: u32) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.0
    }

    fn hits(&mut self, chance_ppm: u32) -> bool {
        chance_ppm > 0 && self.next_u32() % 1_000_000 < chance_ppm
    }
}

struct LogPrinter;

impl HostEventHandler for LogPrinter {
    fn handle_message(&mut self, message: ReceivedMessage) {
        match message.channel_id_u8() {
            LOG_CHANNEL => print_log(message),
            channel => println!(
                "[mcu message channel={channel} #{}] {:02x?}",
                message.message_id(),
                message.as_bytes()
            ),
        }
    }

    fn handle_error(&mut self, error: HostTaskError) {
        eprintln!("[host event] {error:?}");
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
async fn main() -> io::Result<()> {
    let args = Args::parse();

    loop {
        match run_session(&args).await {
            Ok(()) => {}
            Err(error) => eprintln!("failed to open {}: {error}; retrying ...", args.port_path),
        }

        sleep(RECONNECT_DELAY).await;
    }
}

async fn run_session(args: &Args) -> io::Result<()> {
    println!(
        "opening {} at {} baud, noise={:?}",
        args.port_path, args.baud_rate, args.noise
    );

    let serial = tokio_serial::new(&args.port_path, args.baud_rate).open_native_async()?;
    let serial = NoisySerial::new(serial, args.noise);
    let mut adapter = HostAdapter::new(serial);
    let mut handler = LogPrinter;
    let mut rx_buf = [0u8; 256];
    let started = Instant::now();
    let mut next_send = Duration::ZERO;
    let mut ping_id = 0u32;

    loop {
        let elapsed = started.elapsed();
        let now_ms = elapsed.as_millis() as u64;

        if elapsed >= next_send {
            let payload = format!("host ping {ping_id}");
            println!("[host send] {payload}");
            if let Err(error) = adapter.send_message(payload.as_bytes()) {
                eprintln!("[host send error] {error:?}");
            }
            ping_id = ping_id.wrapping_add(1);
            next_send += SEND_INTERVAL;
        }

        adapter
            .poll_once_dispatch(now_ms, &mut rx_buf, &mut handler)
            .await;
    }
}

struct Args {
    port_path: String,
    baud_rate: u32,
    noise: NoiseConfig,
}

impl Args {
    fn parse() -> Self {
        let mut values = env::args().skip(1);
        let port_path = values
            .next()
            .unwrap_or_else(|| "/dev/cu.usbmodem2103".to_owned());
        let baud_rate = values
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(115_200);
        let mut noise = NoiseConfig::default();

        while let Some(flag) = values.next() {
            let Some(value) = values.next() else {
                eprintln!("missing value for {flag}");
                break;
            };
            let chance = chance_ppm(&value);
            match flag.as_str() {
                "--rx-drop" => noise.rx_drop_ppm = chance,
                "--rx-flip" => noise.rx_flip_ppm = chance,
                "--tx-drop" => noise.tx_drop_ppm = chance,
                "--tx-flip" => noise.tx_flip_ppm = chance,
                _ => eprintln!("unknown flag ignored: {flag}"),
            }
        }

        Self {
            port_path,
            baud_rate,
            noise,
        }
    }
}

fn chance_ppm(value: &str) -> u32 {
    let Ok(chance) = value.parse::<f64>() else {
        return 0;
    };
    (chance.clamp(0.0, 1.0) * 1_000_000.0) as u32
}
