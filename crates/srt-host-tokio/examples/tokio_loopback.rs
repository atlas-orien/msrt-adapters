#![allow(missing_docs)]

use srt_host_tokio::HostDriver;
use tokio::io::duplex;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (io_a, io_b) = duplex(2048);

    let mut a = HostDriver::new(io_a);
    let mut b = HostDriver::new(io_b);

    a.send_message(b"hello from host-a").expect("send failed");

    let mut rx_a = [0u8; 256];
    let mut rx_b = [0u8; 256];

    for now_ms in 0..200_u64 {
        a.poll_once(now_ms, &mut rx_a).await.expect("a poll failed");
        b.poll_once(now_ms, &mut rx_b).await.expect("b poll failed");

        if let Some(message) = b.poll_message() {
            println!("host-b received: {}", core::str::from_utf8(message.as_bytes()).unwrap());
            return;
        }
    }

    panic!("message not delivered");
}
