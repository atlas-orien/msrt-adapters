use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::rc::Rc;

use crate::{AdapterEvent, StdBackend, StdFrontend};

#[derive(Clone, Debug)]
struct Duplex {
    rx: Rc<RefCell<VecDeque<u8>>>,
    tx: Rc<RefCell<VecDeque<u8>>>,
}

impl Duplex {
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

impl Read for Duplex {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut rx = self.rx.borrow_mut();
        if rx.is_empty() {
            return Err(std::io::ErrorKind::WouldBlock.into());
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
}

impl Write for Duplex {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tx.borrow_mut().extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn frontend_and_backend_exchange_message() {
    let (frontend_io, backend_io) = Duplex::pair();
    let mut frontend = StdFrontend::new(frontend_io);
    let mut backend = StdBackend::new(backend_io);

    frontend.connect().unwrap();
    drive(&mut frontend, &mut backend);

    frontend.send(b"hello std").unwrap();
    let message = drive_until_message(&mut frontend, &mut backend);

    assert_eq!(message, b"hello std");
}

fn drive(frontend: &mut StdFrontend<Duplex>, backend: &mut StdBackend<Duplex>) {
    for _ in 0..16 {
        let _ = frontend.tick().unwrap();
        let _ = backend.tick().unwrap();
    }
}

fn drive_until_message(
    frontend: &mut StdFrontend<Duplex>,
    backend: &mut StdBackend<Duplex>,
) -> Vec<u8> {
    for _ in 0..32 {
        let _ = frontend.tick().unwrap();
        match backend.tick().unwrap() {
            AdapterEvent::Message(message) if message.as_bytes() != [0] => {
                return message.as_bytes().to_vec();
            }
            _ => {}
        }
    }
    panic!("message was not delivered");
}
