use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    TokioUartFrontend, UartBackend, UartBackendEvent, UartFrontendEvent, UartIo, UartIoError,
    UartIoResult,
};

#[derive(Clone, Debug)]
struct MockUart {
    rx: Rc<RefCell<VecDeque<u8>>>,
    tx: Rc<RefCell<VecDeque<u8>>>,
}

impl MockUart {
    fn pair() -> (Self, Self) {
        let a_to_b = Rc::new(RefCell::new(VecDeque::new()));
        let b_to_a = Rc::new(RefCell::new(VecDeque::new()));
        (
            Self {
                rx: Rc::clone(&b_to_a),
                tx: Rc::clone(&a_to_b),
            },
            Self {
                rx: a_to_b,
                tx: b_to_a,
            },
        )
    }
}

impl UartIo for MockUart {
    type Error = ();

    fn read(&mut self, buf: &mut [u8]) -> UartIoResult<usize, Self::Error> {
        let mut rx = self.rx.borrow_mut();
        if rx.is_empty() {
            return Err(UartIoError::WouldBlock);
        }

        let mut len = 0;
        while len < buf.len() {
            let Some(byte) = rx.pop_front() else {
                break;
            };
            buf[len] = byte;
            len += 1;
        }

        Ok(len)
    }

    fn write_all(&mut self, bytes: &[u8]) -> UartIoResult<(), Self::Error> {
        self.tx.borrow_mut().extend(bytes);
        Ok(())
    }
}

#[test]
fn backend_trait_can_exchange_with_backend_trait_for_protocol_smoke() {
    let (frontend_uart, backend_uart) = MockUart::pair();
    let mut frontend = crate::UartBackend::new(frontend_uart);
    let mut backend = UartBackend::new(backend_uart);

    // Passive/passive cannot establish a real active session, but this still
    // validates that the no-std IO trait path compiles and ticks without data.
    assert!(matches!(frontend.tick(0).unwrap(), UartBackendEvent::Idle));
    assert!(matches!(backend.tick(0).unwrap(), UartBackendEvent::Idle));
}

#[tokio::test]
async fn tokio_frontend_exchanges_with_no_std_backend_mock() {
    let (host, mut device_side) = tokio::io::duplex(4096);
    let mut frontend = TokioUartFrontend::new(host);
    let (backend_uart, mut bridge_uart) = MockUart::pair();
    let mut backend = UartBackend::new(backend_uart);

    frontend.connect().unwrap();

    for step in 0..128_u64 {
        pump_tokio_to_mock(&mut device_side, &mut bridge_uart).await;
        let _ = backend.tick(step).unwrap();
        pump_mock_to_tokio(&mut bridge_uart, &mut device_side).await;
        let _ = frontend.tick().await.unwrap();
    }

    frontend.send(b"hello uart").unwrap();
    let mut delivered = None;

    for step in 128..512_u64 {
        pump_tokio_to_mock(&mut device_side, &mut bridge_uart).await;
        match backend.tick(step).unwrap() {
            UartBackendEvent::Message(message) if message.as_bytes() != [0] => {
                delivered = Some(message.as_bytes().to_vec());
                break;
            }
            _ => {}
        }
        pump_mock_to_tokio(&mut bridge_uart, &mut device_side).await;
        match frontend.tick().await.unwrap() {
            UartFrontendEvent::SendFailed(_) => panic!("frontend send failed"),
            UartFrontendEvent::Message(_)
            | UartFrontendEvent::TransportUnavailable
            | UartFrontendEvent::Idle => {}
        }
    }

    assert_eq!(delivered.as_deref(), Some(&b"hello uart"[..]));
}

async fn pump_tokio_to_mock(stream: &mut tokio::io::DuplexStream, uart: &mut MockUart) {
    let mut buf = [0; 512];
    loop {
        match tokio::time::timeout(Duration::from_millis(1), stream.read(&mut buf)).await {
            Ok(Ok(0)) => return,
            Ok(Ok(n)) => uart.write_all(&buf[..n]).unwrap(),
            Ok(Err(error)) => panic!("tokio read failed: {error}"),
            Err(_) => return,
        }
    }
}

async fn pump_mock_to_tokio(uart: &mut MockUart, stream: &mut tokio::io::DuplexStream) {
    let mut buf = [0; 512];
    loop {
        match uart.read(&mut buf) {
            Ok(0) => return,
            Ok(n) => {
                stream.write_all(&buf[..n]).await.unwrap();
                stream.flush().await.unwrap();
            }
            Err(UartIoError::WouldBlock) => {
                tokio::time::sleep(Duration::from_millis(1)).await;
                return;
            }
            Err(error) => panic!("mock read failed: {error:?}"),
        }
    }
}
