#![allow(missing_docs)]

use std::time::Duration;

use srt_host_tokio::HostDriver;
use tokio::{net::TcpStream, time::sleep};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5555".to_string());

    let stream = TcpStream::connect(&addr).await.expect("connect failed");

    println!("connected {addr}");

    let mut driver = HostDriver::new(stream);
    let mut rx_buf = [0u8; 256];

    driver
        .send_message(b"qemu smoke ping")
        .expect("send_message failed");

    let mut now_ms = 0_u64;
    for _ in 0..5000 {
        now_ms = now_ms.wrapping_add(1);
        driver
            .poll_once(now_ms, &mut rx_buf)
            .await
            .expect("poll_once failed");

        if let Some(message) = driver.poll_message() {
            if let Ok(text) = core::str::from_utf8(message.as_bytes()) {
                println!("recv={text}");
            } else {
                println!("recv bytes len={}", message.as_bytes().len());
            }
            return;
        }

        if let Some(failed) = driver.poll_send_failed() {
            panic!(
                "send_failed channel={} message_id={} reason={:?}",
                failed.channel_id(),
                failed.message_id(),
                failed.reason(),
            );
        }

        sleep(Duration::from_millis(1)).await;
    }

    panic!("timeout waiting message/ack");
}
