#![allow(missing_docs)]

use core::convert::Infallible;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use embedded_io_async::{ErrorType, Read, Write};
use futures::executor::block_on;
use srt_embassy_uart::{UartAdapter, UartTaskError};

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

        let mut a = UartAdapter::new(uart_a);
        let mut b = UartAdapter::new(uart_b);

        a.send_message(b"hello from a")
            .expect("send_message failed");
        a.debug(b"log from a").expect("debug failed");

        let mut rx_a = [0u8; 128];
        let mut rx_b = [0u8; 128];

        let mut got_default = false;
        let mut got_log = false;
        for now_ms in 0..200_u64 {
            a.poll_once_dispatch(
                now_ms,
                &mut rx_a,
                |_| panic!("a received message unexpectedly"),
                |error| {
                    if let UartTaskError::SendFailed(failed) = error {
                        panic!("a send failed unexpectedly: {failed:?}");
                    }
                    panic!("a task error unexpectedly: {error:?}");
                },
            )
            .await;

            b.poll_once_dispatch(
                now_ms,
                &mut rx_b,
                |message| match message.channel_id_u8() {
                    0 => {
                        assert_eq!(message.as_bytes(), b"hello from a");
                        got_default = true;
                    }
                    1 => {
                        assert_eq!(message.as_bytes(), b"log from a");
                        got_log = true;
                    }
                    other => panic!("unexpected channel: {other}"),
                },
                |error| panic!("b task error unexpectedly: {error:?}"),
            )
            .await;

            if got_default && got_log {
                println!("b received both default and log channels");
                return;
            }
        }

        assert!(got_default, "default-channel message was not delivered");
        assert!(got_log, "log-channel message was not delivered");
    });
}
