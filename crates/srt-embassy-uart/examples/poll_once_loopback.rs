#![allow(missing_docs)]

use core::convert::Infallible;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use embedded_io_async::{ErrorType, Read, Write};
use futures::executor::block_on;
use srt::{Config, Engine};
use srt_embassy_uart::UartDriver;

#[derive(Clone, Default)]
struct SharedPipe {
    a_to_b: Arc<Mutex<VecDeque<u8>>>,
    b_to_a: Arc<Mutex<VecDeque<u8>>>,
}

#[derive(Clone)]
struct MockUart {
    inbound: Arc<Mutex<VecDeque<u8>>>,
    outbound: Arc<Mutex<VecDeque<u8>>>,
}

impl SharedPipe {
    fn endpoints(&self) -> (MockUart, MockUart) {
        let a = MockUart {
            inbound: Arc::clone(&self.b_to_a),
            outbound: Arc::clone(&self.a_to_b),
        };
        let b = MockUart {
            inbound: Arc::clone(&self.a_to_b),
            outbound: Arc::clone(&self.b_to_a),
        };
        (a, b)
    }
}

impl ErrorType for MockUart {
    type Error = Infallible;
}

impl Read for MockUart {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut queue = self.inbound.lock().expect("inbound mutex poisoned");

        if queue.is_empty() {
            return Ok(0);
        }

        let mut i = 0;
        while i < buf.len() {
            let Some(byte) = queue.pop_front() else {
                break;
            };
            buf[i] = byte;
            i += 1;
        }

        Ok(i)
    }
}

impl Write for MockUart {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut queue = self.outbound.lock().expect("outbound mutex poisoned");
        for b in buf {
            queue.push_back(*b);
        }
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn main() {
    block_on(async {
        let pipe = SharedPipe::default();
        let (uart_a, uart_b) = pipe.endpoints();

        let mut a = UartDriver::new(uart_a, Engine::new(Config::default()));
        let mut b = UartDriver::new(uart_b, Engine::new(Config::default()));

        let _ = a
            .send_message(b"hello from a")
            .expect("send_message failed");

        let mut rx_a = [0u8; 128];
        let mut rx_b = [0u8; 128];

        let mut delivered = false;
        for now_ms in 0..200_u64 {
            a.poll_once(now_ms, &mut rx_a)
                .await
                .expect("a poll_once failed");
            b.poll_once(now_ms, &mut rx_b)
                .await
                .expect("b poll_once failed");

            if let Some(message) = b.poll_message() {
                println!(
                    "b received: {}",
                    core::str::from_utf8(message.as_bytes()).unwrap()
                );
                delivered = true;
                break;
            }

            if let Some(failed) = a.poll_send_failed() {
                panic!("send failed unexpectedly: {failed:?}");
            }
        }

        assert!(delivered, "message was not delivered in test loop");
    });
}
