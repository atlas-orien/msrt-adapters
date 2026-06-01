use core::ops::{Deref, DerefMut};

use srt_adapter_core::AdapterDriver;

/// Drives SRT engine state over an async UART-like byte stream.
#[derive(Debug)]
pub struct UartDriver<Uart>(AdapterDriver<Uart>);

impl<Uart> UartDriver<Uart> {
    /// Creates a UART driver with default SRT engine config.
    #[must_use]
    pub fn new(uart: Uart) -> Self {
        Self(AdapterDriver::new(uart))
    }

    /// Releases only the UART object.
    #[must_use]
    pub fn into_uart(self) -> Uart {
        self.0.into_io()
    }
}

impl<Uart> Deref for UartDriver<Uart> {
    type Target = AdapterDriver<Uart>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Uart> DerefMut for UartDriver<Uart> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
