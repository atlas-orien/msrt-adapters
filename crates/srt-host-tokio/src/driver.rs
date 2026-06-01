use core::ops::{Deref, DerefMut};

use srt_adapter_core::AdapterDriver;

#[derive(Debug)]
pub struct HostDriver<Io>(AdapterDriver<Io>);

impl<Io> HostDriver<Io> {
    #[must_use]
    pub fn new(io: Io) -> Self {
        Self(AdapterDriver::new(io))
    }

    #[must_use]
    pub fn into_io(self) -> Io {
        self.0.into_io()
    }
}

impl<Io> Deref for HostDriver<Io> {
    type Target = AdapterDriver<Io>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Io> DerefMut for HostDriver<Io> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
