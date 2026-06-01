#![allow(missing_docs)]

use core::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    task::Wake,
};

use embedded_io_async::{ErrorType, Read, Write};
use srt_embassy_uart::{ReceivedMessage, UartDriver};

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
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
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
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        let mut queue = self.outbound.lock().expect("outbound mutex poisoned");
        for b in buf {
            queue.push_back(*b);
        }
        Ok(buf.len())
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }
}

mod mcu_api {
    use super::MockUart;

    srt_embassy_uart::define_srt_uart!(MockUart);
}

fn main() {
    let pipe = SharedPipe::default();
    let (uart_a, uart_b) = pipe.endpoints();

    mcu_api::init(uart_a).expect("init failed");
    let mut host = UartDriver::new(uart_b);

    mcu_api::send_message(b"hello from mcu").expect("send_message failed");
    mcu_api::debug(b"debug from mcu").expect("debug failed");

    let mut mcu_rx = [0u8; 128];
    let mut host_rx = [0u8; 128];
    let mut got_default = false;
    let mut got_log = false;

    for now_ms in 0..200_u64 {
        block_on(mcu_api::__poll_once_for_test(now_ms, &mut mcu_rx)).expect("mcu poll_once failed");
        block_on(host.poll_once(now_ms, &mut host_rx)).expect("host poll_once failed");

        while let Some(message) = host.poll_message() {
            assert_expected_message(message, &mut got_default, &mut got_log);
        }

        if got_default && got_log {
            println!("singleton api delivered message and debug channels");
            return;
        }
    }

    panic!("messages not delivered in test loop");
}

fn assert_expected_message(message: ReceivedMessage, got_default: &mut bool, got_log: &mut bool) {
    match message.channel_id_u8() {
        0 => {
            assert_eq!(message.as_bytes(), b"hello from mcu");
            *got_default = true;
        }
        1 => {
            assert_eq!(message.as_bytes(), b"debug from mcu");
            *got_log = true;
        }
        other => panic!("unexpected channel: {other}"),
    }
}

fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    let waker = std::task::Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    let mut future = core::pin::pin!(future);

    loop {
        match Future::poll(Pin::as_mut(&mut future), &mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => {}
        }
    }
}

struct NoopWake;

impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}
