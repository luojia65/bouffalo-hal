use super::typestate::{Floating, Input, Output, PullDown, PullUp, Uart};
use crate::glb::{Drive, Pull, v1};
use core::marker::PhantomData;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

/// Raw GPIO pad of BL602 and BL702.
pub struct Padv1<'a, const N: usize, M> {
    base: &'a v1::RegisterBlock,
    _mode: PhantomData<M>,
}

impl<'a, const N: usize, M> Padv1<'a, N, Input<M>> {
    /// Enable schmitt trigger.
    #[inline]
    pub fn enable_schmitt(&mut self) {
        let config = self.base.gpio_config[N >> 1].read().enable_schmitt(N & 0x1);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
    }
    /// Disable schmitt trigger.
    #[inline]
    pub fn disable_schmitt(&mut self) {
        let config = self.base.gpio_config[N >> 1]
            .read()
            .disable_schmitt(N & 0x1);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
    }
    /// Clear interrupt flag.
    #[inline]
    pub fn clear_interrupt(&mut self) {
        unsafe { self.base.gpio_interrupt_clear.write(1 << N) };
    }
    /// Check if interrupt flag is set.
    #[inline]
    pub fn has_interrupt(&self) -> bool {
        self.base.gpio_interrupt_state.read() & (1 << N) != 0
    }
    /// Mask interrupt.
    #[inline]
    pub fn mask_interrupt(&mut self) {
        let config = self.base.gpio_interrupt_mask.read() | (1 << N);
        unsafe { self.base.gpio_interrupt_mask.write(config) };
    }
    /// Unmask interrupt.
    #[inline]
    pub fn unmask_interrupt(&mut self) {
        let config = self.base.gpio_interrupt_mask.read() & !(1 << N);
        unsafe { self.base.gpio_interrupt_mask.write(config) };
    }
}

impl<'a, const N: usize, M> Padv1<'a, N, Output<M>> {
    /// Get drive strength of this pin.
    #[inline]
    pub fn drive(&self) -> Drive {
        self.base.gpio_config[N >> 1].read().drive(N & 0x1)
    }
    /// Set drive strength of this pin.
    #[inline]
    pub fn set_drive(&mut self, val: Drive) {
        let config = self.base.gpio_config[N >> 1].read().set_drive(N & 0x1, val);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
    }
}

impl<'a, const N: usize, M> Padv1<'a, N, Input<M>> {
    /// Get interrupt mode.
    #[inline]
    pub fn interrupt_mode(&self) -> v1::InterruptMode {
        self.base.gpio_interrupt_mode[N >> 1]
            .read()
            .interrupt_mode(N & 0x1)
    }
    /// Set interrupt mode.
    #[inline]
    pub fn set_interrupt_mode(&mut self, val: v1::InterruptMode) {
        let config = self.base.gpio_interrupt_mode[N >> 1]
            .read()
            .set_interrupt_mode(N & 0x1, val);
        unsafe { self.base.gpio_interrupt_mode[N >> 1].write(config) };
    }
}

impl<'a, const N: usize, M> Padv1<'a, N, M> {
    /// Configures the pin to operate as a pull up output pin.
    #[inline]
    pub fn into_pull_up_output(self) -> Padv1<'a, N, Output<PullUp>> {
        let config = self.base.gpio_config[N >> 1]
            .read()
            .set_function(N & 0x1, v1::Function::Gpio)
            .disable_input(N & 0x1)
            .set_pull(N & 0x1, Pull::Up);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
        let val = self.base.gpio_output_enable.read();
        unsafe { self.base.gpio_output_enable.write(val | (1 << N)) };
        Padv1 {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configures the pin to operate as a pull down output pin.
    #[inline]
    pub fn into_pull_down_output(self) -> Padv1<'a, N, Output<PullDown>> {
        let config = self.base.gpio_config[N >> 1]
            .read()
            .set_function(N & 0x1, v1::Function::Gpio)
            .disable_input(N & 0x1)
            .set_pull(N & 0x1, Pull::Down);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
        let val = self.base.gpio_output_enable.read();
        unsafe { self.base.gpio_output_enable.write(val | (1 << N)) };
        Padv1 {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configures the pin to operate as a floating output pin.
    #[inline]
    pub fn into_floating_output(self) -> Padv1<'a, N, Output<Floating>> {
        let config = self.base.gpio_config[N >> 1]
            .read()
            .set_function(N & 0x1, v1::Function::Gpio)
            .disable_input(N & 0x1)
            .set_pull(N & 0x1, Pull::None);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
        let val = self.base.gpio_output_enable.read();
        unsafe { self.base.gpio_output_enable.write(val | (1 << N)) };
        Padv1 {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configures the pin to operate as a pull up input pin.
    #[inline]
    pub fn into_pull_up_input(self) -> Padv1<'a, N, Input<PullUp>> {
        let config = self.base.gpio_config[N >> 1]
            .read()
            .set_function(N & 0x1, v1::Function::Gpio)
            .enable_input(N & 0x1)
            .set_pull(N & 0x1, Pull::Up);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
        let val = self.base.gpio_output_enable.read();
        unsafe { self.base.gpio_output_enable.write(val & !(1 << N)) };
        Padv1 {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configures the pin to operate as a pull down input pin.
    #[inline]
    pub fn into_pull_down_input(self) -> Padv1<'a, N, Input<PullDown>> {
        let config = self.base.gpio_config[N >> 1]
            .read()
            .set_function(N & 0x1, v1::Function::Gpio)
            .enable_input(N & 0x1)
            .set_pull(N & 0x1, Pull::Down);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
        let val = self.base.gpio_output_enable.read();
        unsafe { self.base.gpio_output_enable.write(val & !(1 << N)) };
        Padv1 {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configures the pin to operate as a floating input pin.
    #[inline]
    pub fn into_floating_input(self) -> Padv1<'a, N, Input<Floating>> {
        let config = self.base.gpio_config[N >> 1]
            .read()
            .set_function(N & 0x1, v1::Function::Gpio)
            .enable_input(N & 0x1)
            .set_pull(N & 0x1, Pull::None);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
        let val = self.base.gpio_output_enable.read();
        unsafe { self.base.gpio_output_enable.write(val & !(1 << N)) };
        Padv1 {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

impl<'a, const N: usize, M> Padv1<'a, N, M> {
    /// Configures the pin to operate as UART signal.
    #[inline]
    pub fn into_uart(self) -> Padv1<'a, N, Uart> {
        let config = self.base.gpio_config[N >> 1]
            .read()
            .set_function(N & 0x1, v1::Function::Uart)
            .enable_input(N & 0x1)
            .set_pull(N & 0x1, Pull::None);
        unsafe { self.base.gpio_config[N].write(config) };
        Padv1 {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

impl<'a, const N: usize, M> ErrorType for Padv1<'a, N, Input<M>> {
    type Error = core::convert::Infallible;
}

impl<'a, const N: usize, M> ErrorType for Padv1<'a, N, Output<M>> {
    type Error = core::convert::Infallible;
}

impl<'a, const N: usize, M> InputPin for Padv1<'a, N, Input<M>> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.base.gpio_input_value.read() & (1 << N) != 0)
    }
    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.base.gpio_input_value.read() & (1 << N) == 0)
    }
}

impl<'a, const N: usize, M> OutputPin for Padv1<'a, N, Output<M>> {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        let val = self.base.gpio_output_value.read();
        unsafe { self.base.gpio_output_value.write(val & !(1 << N)) };
        Ok(())
    }
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        let val = self.base.gpio_output_value.read();
        unsafe { self.base.gpio_output_value.write(val | (1 << N)) };
        Ok(())
    }
}

// Macro internal functions, do not use.
impl<'a, const N: usize> Padv1<'a, N, super::typestate::Disabled> {
    #[doc(hidden)]
    #[inline]
    pub fn __from_glb(base: &'a v1::RegisterBlock) -> Self {
        Self {
            base,
            _mode: PhantomData,
        }
    }
}
