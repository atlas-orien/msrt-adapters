#![allow(missing_docs)]

use srt_host_tokio::HostAdapter;
use tokio::io::duplex;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (io_a, io_b) = duplex(2048);

    let mut a = HostAdapter::new(io_a);
    let mut b = HostAdapter::new(io_b);

    a.send_message(b"hello from host-a").expect("send failed");

    let mut rx_a = [0u8; 256];
    let mut rx_b = [0u8; 256];

    for now_ms in 0..200_u64 {
        a.poll_once_dispatch(
            now_ms,
            &mut rx_a,
            |_| panic!("host-a received message unexpectedly"),
            |error| panic!("host-a task error unexpectedly: {error:?}"),
        )
        .await;

        let mut received = false;
        b.poll_once_dispatch(
            now_ms,
            &mut rx_b,
            |message| {
                println!(
                    "host-b received: {}",
                    core::str::from_utf8(message.as_bytes()).unwrap()
                );
                received = true;
            },
            |error| panic!("host-b task error unexpectedly: {error:?}"),
        )
        .await;

        if received {
            return;
        }
    }

    panic!("message not delivered");
}
