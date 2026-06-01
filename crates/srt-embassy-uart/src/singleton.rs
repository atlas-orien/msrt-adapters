/// Defines a single global SRT UART API for one MCU UART link.
///
/// The generated public API is intentionally small: initialize once, then use
/// `send_message` and `debug` from application code. The generated `run_task`
/// owns the UART adapter loop and should be spawned by the application startup.
#[macro_export]
macro_rules! define_srt_uart {
    ($uart_ty:ty) => {
        static SRT_ADAPTER: ::critical_section::Mutex<
            ::core::cell::RefCell<Option<$crate::UartAdapter<$uart_ty>>>,
        > = ::critical_section::Mutex::new(::core::cell::RefCell::new(None));

        pub fn init(uart: $uart_ty) -> $crate::Result<()> {
            ::critical_section::with(|cs| {
                let mut slot = SRT_ADAPTER.borrow(cs).borrow_mut();
                if slot.is_some() {
                    return Err($crate::Error::new($crate::ErrorKind::AlreadyInitialized));
                }
                *slot = Some($crate::UartAdapter::new(uart));
                Ok(())
            })
        }

        pub fn send_message(message: &[u8]) -> $crate::Result<()> {
            ::critical_section::with(|cs| {
                let mut slot = SRT_ADAPTER.borrow(cs).borrow_mut();
                let Some(adapter) = slot.as_mut() else {
                    return Err($crate::Error::new($crate::ErrorKind::NotInitialized));
                };
                adapter.send_message(message)
            })
        }

        pub fn debug(message: &[u8]) -> $crate::Result<()> {
            ::critical_section::with(|cs| {
                let mut slot = SRT_ADAPTER.borrow(cs).borrow_mut();
                let Some(adapter) = slot.as_mut() else {
                    return Err($crate::Error::new($crate::ErrorKind::NotInitialized));
                };
                adapter.debug(message)
            })
        }

        pub async fn run_task<MessageHandler, ErrorHandler>(
            mut now: impl FnMut() -> u64,
            rx_buf: &mut [u8],
            mut handle_message: MessageHandler,
            mut handle_error: ErrorHandler,
        ) where
            $uart_ty: ::embedded_io_async::Read + ::embedded_io_async::Write,
            MessageHandler: FnMut($crate::ReceivedMessage),
            ErrorHandler: FnMut($crate::UartTaskError),
        {
            loop {
                let mut adapter = ::critical_section::with(|cs| {
                    SRT_ADAPTER
                        .borrow(cs)
                        .borrow_mut()
                        .take()
                        .expect("SRT UART is not initialized")
                });

                adapter
                    .poll_once_dispatch(now(), rx_buf, &mut handle_message, &mut handle_error)
                    .await;

                ::critical_section::with(|cs| {
                    *SRT_ADAPTER.borrow(cs).borrow_mut() = Some(adapter);
                });
            }
        }

        #[cfg(feature = "std")]
        #[doc(hidden)]
        pub async fn __poll_once_for_test(now_ms: u64, rx_buf: &mut [u8]) -> $crate::Result<()>
        where
            $uart_ty: ::embedded_io_async::Read + ::embedded_io_async::Write,
        {
            let mut adapter = ::critical_section::with(|cs| {
                SRT_ADAPTER
                    .borrow(cs)
                    .borrow_mut()
                    .take()
                    .ok_or_else(|| $crate::Error::new($crate::ErrorKind::NotInitialized))
            })?;

            let result = adapter.poll_once(now_ms, rx_buf).await;

            ::critical_section::with(|cs| {
                *SRT_ADAPTER.borrow(cs).borrow_mut() = Some(adapter);
            });

            result
        }
    };
}
