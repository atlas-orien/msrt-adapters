#![allow(clippy::std_instead_of_core)]

use std::env;
use std::thread;
use std::time::{Duration, Instant};

use msrt_udp::{EngineConfig, UdpServer, UdpServerEvent};

const DEFAULT_BIND: &str = "127.0.0.1:9000";
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const LOOP_SLEEP: Duration = Duration::from_millis(10);

fn main() -> msrt_udp::Result<()> {
    let bind = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_BIND.to_string());
    let mut server = UdpServer::<16>::bind_with_config(&bind, demo_config())?;
    println!("server listening on {}", server.local_addr()?);

    let mut next_idle_sweep = Instant::now() + Duration::from_secs(1);

    loop {
        match server.tick()? {
            UdpServerEvent::Message { peer, message } if message.as_bytes() != [0] => {
                let text = String::from_utf8_lossy(message.as_bytes());
                println!("{peer}: {text}");
                let reply = format!("echo from server: {text}");
                let _ = server.send_to(peer, reply.as_bytes())?;
            }
            UdpServerEvent::Message { .. } | UdpServerEvent::Idle => {}
            UdpServerEvent::SendFailed { peer, failed } => {
                println!("{peer}: send failed: {failed:?}; disconnecting peer");
                server.disconnect(peer);
            }
        }

        if next_idle_sweep <= Instant::now() {
            let disconnected = server.disconnect_idle(IDLE_TIMEOUT.as_millis() as u64);
            if disconnected > 0 {
                println!("disconnected {disconnected} idle peer(s)");
            }
            next_idle_sweep = Instant::now() + Duration::from_secs(1);
        }

        thread::sleep(LOOP_SLEEP);
    }
}

fn demo_config() -> EngineConfig {
    EngineConfig {
        retransmit_timeout_ms: 250,
        max_retransmit_attempts: 8,
        ..EngineConfig::default()
    }
}
