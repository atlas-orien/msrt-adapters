#![allow(missing_docs)]

use std::time::Duration;

use srt::{Config, Engine};
use srt_host_tokio::HostDriver;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::TcpStream,
    time::sleep,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let addr = "127.0.0.1:5555";
    println!("connecting to {addr} ...");

    let stream = TcpStream::connect(addr)
        .await
        .expect("failed to connect qemu serial tcp socket");

    println!("connected");

    let mut driver = HostDriver::new(stream, Engine::new(Config::default()));
    let mut rx_buf = [0u8; 256];
    let mut now_ms: u64 = 0;

    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    println!("type message and press Enter to send");

    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if text.trim().is_empty() {
                            continue;
                        }

                        match driver.send_message(text.as_bytes()) {
                            Ok(message_id) => println!("sent message_id={}", message_id.get()),
                            Err(err) => println!("send_message error: {:?}", err.kind()),
                        }
                    }
                    Ok(None) => {
                        println!("stdin closed");
                        break;
                    }
                    Err(err) => {
                        println!("stdin error: {err}");
                        break;
                    }
                }
            }
            _ = sleep(Duration::from_millis(1)) => {
                now_ms = now_ms.wrapping_add(1);

                if let Err(err) = driver.poll_once(now_ms, &mut rx_buf).await {
                    println!("poll_once error: {:?}", err.kind());
                    continue;
                }

                while let Some(message) = driver.poll_message() {
                    match core::str::from_utf8(message.as_bytes()) {
                        Ok(text) => println!("recv: {text}"),
                        Err(_) => println!("recv bytes: {:?}", message.as_bytes()),
                    }
                }

                while let Some(failed) = driver.poll_send_failed() {
                    println!(
                        "send_failed: channel={} message_id={} reason={:?}",
                        failed.channel_id.get(),
                        failed.message_id.get(),
                        failed.reason,
                    );
                }
            }
        }
    }
}
