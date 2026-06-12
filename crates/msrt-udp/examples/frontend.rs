#![allow(clippy::std_instead_of_core)]

use std::env;
use std::io::ErrorKind;
use std::thread;
use std::time::{Duration, Instant};

use msrt_udp::{EngineConfig, UdpClient, UdpClientEvent};

const DEFAULT_SERVER: &str = "127.0.0.1:9000";
const SEND_INTERVAL: Duration = Duration::from_secs(1);
const LOOP_SLEEP: Duration = Duration::from_millis(10);

fn main() -> msrt_udp::Result<()> {
    let server = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_SERVER.to_string());

    let mut client = UdpClient::bind_with_config("127.0.0.1:0", &server, demo_config())?;
    println!(
        "frontend local={} remote={}",
        client.local_addr()?,
        client.peer_addr()?
    );

    reconnect(&mut client)?;
    let mut sequence = 0_u64;
    let mut next_send = Instant::now();

    loop {
        if next_send <= Instant::now() {
            let payload = format!("hello udp {sequence}");
            if client.send(payload.as_bytes())? {
                println!("queued: {payload}");
                sequence = sequence.wrapping_add(1);
            }
            next_send = Instant::now() + SEND_INTERVAL;
        }

        match client.tick()? {
            UdpClientEvent::Message(message) if message.as_bytes() != [0] => {
                println!("message: {}", String::from_utf8_lossy(message.as_bytes()));
            }
            UdpClientEvent::Message(_) | UdpClientEvent::Idle => {}
            UdpClientEvent::SendFailed(failed) => {
                println!("send failed: {failed:?}; reconnecting");
                client.disconnect();
                reconnect(&mut client)?;
                next_send = Instant::now();
            }
            UdpClientEvent::TransportUnavailable { kind } => {
                println!("transport unavailable: {kind:?}; reconnecting");
                client.disconnect();
                reconnect(&mut client)?;
                next_send = Instant::now() + reconnect_delay(kind);
            }
        }

        thread::sleep(LOOP_SLEEP);
    }
}

fn reconnect_delay(kind: ErrorKind) -> Duration {
    match kind {
        ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset => Duration::from_millis(250),
        _ => Duration::from_secs(1),
    }
}

fn reconnect(client: &mut UdpClient) -> msrt_udp::Result<()> {
    client.connect()?;
    println!("session started");
    Ok(())
}

fn demo_config() -> EngineConfig {
    EngineConfig {
        retransmit_timeout_ms: 250,
        max_retransmit_attempts: 8,
        ..EngineConfig::default()
    }
}
