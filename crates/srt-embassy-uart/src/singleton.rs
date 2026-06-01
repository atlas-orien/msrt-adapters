/// Defines a single global SRT UART API for one MCU UART link.
///
/// The generated public API is intentionally small: initialize once, then use
/// `send_message` and `debug` from application code. The generated `run_task`
/// owns the UART driver loop and should be spawned by the application startup.
/// owns the UART driver loop and should be spawned by the application startup.
#[macro_export]
macro_rules! define_srt_uart {
    ($uart_ty:ty) => {
        static SRT_DRIVER: ::critical_section::Mutex<
            ::core::cell::RefCell<Option<$crate::UartDriver<$uart_ty>>>,
        > = ::critical_section::Mutex::new(::core::cell::RefCell::new(None));

        pub fn init(uart: $uart_ty) -> $crate::Result<()> {
            ::critical_section::with(|cs| {
                let mut slot = SRT_DRIVER.borrow(cs).borrow_mut();
                if slot.is_some() {
                    return Err($crate::Error::new($crate::ErrorKind::AlreadyInitialized));
                }
                *slot = Some($crate::UartDriver::new(uart));
                Ok(())
            })
        }

        pub fn send_message(message: &[u8]) -> $crate::Result<()> {
            ::critical_section::with(|cs| {
                let mut slot = SRT_DRIVER.borrow(cs).borrow_mut();
                let Some(driver) = slot.as_mut() else {
                    return Err($crate::Error::new($crate::ErrorKind::NotInitialized));
                };
                driver.send_message(message)
            })
        }

        pub fn debug(message: &[u8]) -> $crate::Result<()> {
            ::critical_section::with(|cs| {
                let mut slot = SRT_DRIVER.borrow(cs).borrow_mut();
                let Some(driver) = slot.as_mut() else {
                    return Err($crate::Error::new($crate::ErrorKind::NotInitialized));
                };
                driver.debug(message)
            })
        }

        pub async fn run_task(mut now: impl FnMut() -> u64, rx_buf: &mut [u8])
        where
            $uart_ty: ::embedded_io_async::Read + ::embedded_io_async::Write,
        {
            loop {
                let mut driver = ::critical_section::with(|cs| {
                    SRT_DRIVER
                        .borrow(cs)
                        .borrow_mut()
                        .take()
                        .expect("SRT UART is not initialized")
                });

                let _ = driver.poll_once(now(), rx_buf).await;

                ::critical_section::with(|cs| {
                    *SRT_DRIVER.borrow(cs).borrow_mut() = Some(driver);
                });
            }
        }

        #[cfg(feature = "std")]
        #[doc(hidden)]
        pub async fn __poll_once_for_test(now_ms: u64, rx_buf: &mut [u8]) -> $crate::Result<()>
        where
            $uart_ty: ::embedded_io_async::Read + ::embedded_io_async::Write,
        {
            let mut driver = ::critical_section::with(|cs| {
                SRT_DRIVER
                    .borrow(cs)
                    .borrow_mut()
                    .take()
                    .ok_or_else(|| $crate::Error::new($crate::ErrorKind::NotInitialized))
            })?;

            let result = driver.poll_once(now_ms, rx_buf).await;

            ::critical_section::with(|cs| {
                *SRT_DRIVER.borrow(cs).borrow_mut() = Some(driver);
            });

            result
        }
    };
}
