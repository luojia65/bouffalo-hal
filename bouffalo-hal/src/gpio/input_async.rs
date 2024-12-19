use super::input::Input;
use crate::glb::{v2, RegisterBlock};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll},
};

const MAX_PADS: usize = 46;

#[derive(Debug)]
pub struct GpioState {
    pads: [atomic_waker::AtomicWaker; MAX_PADS],
    ref_to_glb: AtomicUsize,
}

impl GpioState {
    /// Creates the set of wakers for GPIO peripheral.
    #[inline]
    pub const fn new() -> GpioState {
        GpioState {
            pads: [const { atomic_waker::AtomicWaker::new() }; MAX_PADS],
            ref_to_glb: AtomicUsize::new(0),
        }
    }
    /// Use this waker set to handle interrupt.
    #[inline]
    pub fn on_interrupt(&self) {
        let glb = unsafe { &*(self.ref_to_glb.load(Ordering::Acquire) as *const RegisterBlock) };
        for (pad_id, pad_waker) in self.pads.iter().enumerate() {
            if let Some(waker) = pad_waker.take() {
                let has_interrupt = match () {
                    #[cfg(feature = "glb-v1")]
                    () => (glb.gpio_interrupt_mask.read() & (1 << pad_id)) != 0,
                    #[cfg(feature = "glb-v2")]
                    () => glb.gpio_config[pad_id].read().has_interrupt(),
                };
                if has_interrupt {
                    waker.wake();
                }
                match () {
                    #[cfg(feature = "glb-v1")]
                    () => todo!(),
                    #[cfg(feature = "glb-v2")]
                    () => unsafe { glb.gpio_config[pad_id].modify(|v| v.clear_interrupt()) },
                };
            }
        }
    }
}

pub struct AsyncInput<'a, const N: usize, M> {
    pad: Input<'a, N, M>,
    registry: &'a GpioState,
}

struct InputFuture<'r, const N: usize, M> {
    pad: &'r Input<'r, N, M>,
    registry: &'r atomic_waker::AtomicWaker,
}

impl<const N: usize, M> Future for InputFuture<'_, N, M> {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.pad.has_interrupt() {
            Poll::Ready(())
        } else {
            self.registry.register(cx.waker());
            Poll::Pending
        }
    }
}

impl<'a, const N: usize, M> embedded_hal::digital::ErrorType for AsyncInput<'a, N, M> {
    type Error = core::convert::Infallible;
}

impl<'a, const N: usize, M> embedded_hal_async::digital::Wait for AsyncInput<'a, N, M> {
    #[inline]
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        match () {
            #[cfg(feature = "glb-v1")]
            () => self
                .pad
                .inner
                .set_interrupt_mode(v1::InterruptMode::SyncHighLevel),
            #[cfg(feature = "glb-v2")]
            () => self
                .pad
                .inner
                .set_interrupt_mode(v2::InterruptMode::SyncHighLevel),
        }
        InputFuture {
            pad: &self.pad,
            registry: &self.registry.pads[N],
        }
        .await;
        Ok(())
    }

    #[inline]
    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}
