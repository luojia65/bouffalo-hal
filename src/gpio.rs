//! General Purpose Input/Output.
#[cfg(feature = "glb-v1")]
use crate::glb::v1;
#[cfg(any(doc, feature = "glb-v2"))]
use crate::glb::v2;
#[cfg(any(feature = "glb-v1", feature = "glb-v2"))]
use crate::GLB;
use base_address::BaseAddress;
use core::marker::PhantomData;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

/// Individual GPIO pin.
///
/// Generic Purpose Input/Output, or GPIO, is a standard interface used to connect the
/// microcontroller to external hardware. There are multiple GPIO pins on one chip,
/// which are different due to the signal of the internal connections. This structure
/// distinguishes them using constant generic number `N`.
///
/// This structure represents a code ownership of Generic Purpose Input/Output pin.
/// GPIO pin structures are mutually exclusive, meaning that code in different contexts
/// are allowed use the same GPIO pin at the same time. Ownerships pass from structure
/// members to their wrappers; if one peripheral owns this pin, another peripheral
/// cannot use it at the same time.
///
/// GPIO pin structure has an alternate mode parameter `M`. Alternate modes can be
/// switched by using functions begin with `into_`. Those functions internally operate
/// on hardware (specifically, GLB peripheral) to change pin function, and convert this
/// structure to `Pin` type with different alternate mode generic parameters. With new
/// alternate mode types, `Pin` structure would now match the demand of creating new
/// peripheral structures, or include specific functions for developers to use.
///
/// # Examples
///
/// A simple usage of GPIO pin is digital signal output.
///
/// ```no_run
/// # use base_address::Static;
/// # use bl_soc::gpio::Pads;
/// # pub struct Peripherals { gpio: Pads<Static<0x20000000>> }
/// # fn main() -> ! {
/// #   let p: Peripherals = unsafe { core::mem::transmute(()) };
/// use embedded_hal::digital::{OutputPin, PinState};
///
/// // Switch io8 pin into floating output mode to prepare setting its state.
/// // It's connected to an LED to be controlled on hardware.
/// let mut led = p.gpio.io8.into_floating_output();
/// // Define a software LED state to be written into hardware.
/// let mut led_state = PinState::High;
/// loop {
///     // We use function `set_state` to write GPIO electronic level.
///     // This function is provided by floating output pin through embedded-hal
///     // crate; we can also use `set_high` or `set_low` here.
///     led.set_state(led_state).ok();
///     // Now we flip software LED state to make the LED blink in next loop.
///     led_state = !led_state;
///     // Delay for some time for human eyes to discover the blink.
///     for _ in 0..100_000 {
///         unsafe { core::arch::asm!("nop") }
///     }
/// }
/// # }
/// ```
///
/// Peripheral structures usually limits the alternate mode of a GPIO pin. It must be
/// adjusted to the corresponding type before it can be used to create such structures.
///
/// ```no_run
/// # use base_address::{BaseAddress, Static};
/// # use embedded_time::rate::*;
/// # use bl_soc::{
/// #     clocks::Clocks,
/// #     gpio::{Pads, Pad, Alternate},
/// #     uart::{BitOrder, Config, Parity, StopBits, WordLength},
/// #     UART,
/// # };
/// # use embedded_io::Write;
/// # pub struct Serial<PINS> { pins: PINS }
/// # impl<PINS> Serial<PINS> {
/// #     pub fn new(_: UART<impl BaseAddress, 0>, _: Config, _: Baud,
/// # #[cfg(feature = "glb-v2")] _: PINS, _: &Clocks, _: &GLB<impl BaseAddress>)
/// #     -> Self { unimplemented!() }
/// #     pub fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> Result<(), ()> { unimplemented!() }
/// #     pub fn flush(&mut self) -> Result<(), ()> { unimplemented!() }
/// # }
/// # pub struct GLB<A: BaseAddress> {
/// #     base: A,
/// # }
/// # pub struct Peripherals {
/// #     gpio: Pads<Static<0x20000000>>,
/// #     glb: GLB<Static<0x20000000>>,
/// #     uart0: UART<Static<0x2000A000>, 0>,
/// # }
/// # fn main() {
/// # let p: Peripherals = unsafe { core::mem::transmute(()) };
/// # let clocks = Clocks { xtal: Hertz(40_000_000) };
/// // Prepare UART transmit and receive pins by converting io14 and io15 into
/// // UART signal alternate mode.
/// # #[cfg(feature = "glb-v2")]
/// let tx = p.gpio.io14.into_uart();
/// # #[cfg(feature = "glb-v2")]
/// let rx = p.gpio.io15.into_uart();
/// # let sig2 = ();
/// # let sig3 = ();
/// # let config = Config {
/// #     bit_order: BitOrder::LsbFirst,
/// #     parity: Parity::None,
/// #     stop_bits: StopBits::One,
/// #     word_length: WordLength::Eight,
/// # };
/// // Create the serial structure. Note that if we don't have tx and rx GPIO
/// // alternate mode set correctly, code here won't compile for type mismatch.
/// # #[cfg(feature = "glb-v2")]
/// let mut serial = Serial::new(
///     p.uart0,
///     config,
///     2000000.Bd(),
///     ((tx, sig2), (rx, sig3)),
///     &clocks,
///     &p.glb,
/// );
/// # #[cfg(not(feature = "glb-v2"))]
/// # let mut serial = Serial { pins: () };
/// // Now that we have a working serial structure, we write something with it.
/// writeln!(serial, "Hello world!").ok();
/// serial.flush().ok();
/// # }
/// ```
pub struct Pad<A: BaseAddress, const N: usize, M: Alternate> {
    #[cfg(any(feature = "glb-v1", feature = "glb-v2"))]
    pub(crate) base: GLB<A>,
    #[cfg(not(any(feature = "glb-v1", feature = "glb-v2")))]
    pub(crate) _base_not_implemented: PhantomData<A>,
    pub(crate) _mode: PhantomData<M>,
}

/// Alternate type state.
pub trait Alternate {
    /// Function number for this alternate type state.
    #[cfg(feature = "glb-v2")]
    const F: v2::Function;
}

/// Input mode (type state).
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Output mode (type state).
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Disabled (type state).
pub struct Disabled;

/// Pulled down (type state).
pub struct PullDown;

/// Pulled up (type state).
pub struct PullUp;

/// Floating (type state).
pub struct Floating;

impl<MODE> Alternate for Input<MODE> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::Gpio;
}

impl<MODE> Alternate for Output<MODE> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::Gpio;
}

impl Alternate for Disabled {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::Gpio;
}

impl<A: BaseAddress, const N: usize, M> ErrorType for Pad<A, N, Input<M>> {
    type Error = core::convert::Infallible;
}

impl<A: BaseAddress, const N: usize, M> ErrorType for Pad<A, N, Output<M>> {
    type Error = core::convert::Infallible;
}

impl<A: BaseAddress, const N: usize, M> InputPin for Pad<A, N, Input<M>> {
    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                Ok(self.base.gpio_input_value.read() & (1 << N) != 0)
            } else if #[cfg(feature = "glb-v2")] {
                Ok(self.base.gpio_input[N >> 5].read() & (1 << (N & 0x1F)) != 0)
            } else {
                unimplemented!()
            }
        }
    }
    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                Ok(self.base.gpio_input_value.read() & (1 << N) == 0)
            } else if #[cfg(feature = "glb-v2")] {
                Ok(self.base.gpio_input[N >> 5].read() & (1 << (N & 0x1F)) == 0)
            } else {
                unimplemented!()
            }
        }
    }
}

impl<A: BaseAddress, const N: usize, M> OutputPin for Pad<A, N, Output<M>> {
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let val = self.base.gpio_output_value.read();
                unsafe { self.base.gpio_output_value.write(val & !(1 << N)) };
                Ok(())
            } else if #[cfg(feature = "glb-v2")] {
                unsafe { self.base.gpio_clear[N >> 5].write(1 << (N & 0x1F)) };
                Ok(())
            } else {
                unimplemented!()
            }
        }
    }
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let val = self.base.gpio_output_value.read();
                unsafe { self.base.gpio_output_value.write(val | (1 << N)) };
                Ok(())
            } else if #[cfg(feature = "glb-v2")] {
                unsafe { self.base.gpio_set[N >> 5].write(1 << (N & 0x1F)) };
                Ok(())
            } else {
                unimplemented!()
            }
        }
    }
}

// This part of implementation using `embedded_hal_027` is designed for backward compatibility of
// ecosystem crates, as some of them depends on embedded-hal v0.2.7 traits.
// We encourage ecosystem developers to use embedded-hal v1.0.0 traits; after that, this part of code
// would be removed in the future.
impl<A: BaseAddress, const N: usize, M> embedded_hal_027::digital::v2::OutputPin
    for Pad<A, N, Output<M>>
{
    type Error = core::convert::Infallible;
    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        <Self as OutputPin>::set_low(self)
    }
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        <Self as OutputPin>::set_high(self)
    }
}

// We do not support StatefulOutputPin and ToggleableOutputPin here, because the hardware does not
// have such functionality to read back the previously set pin state.
// It is recommended that users add a variable to store the pin state if necessary; see examples/gpio-demo.

impl<A: BaseAddress, const N: usize, M> Pad<A, N, Input<M>> {
    /// Enable schmitt trigger.
    #[inline]
    pub fn enable_schmitt(&mut self) {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_config[N >> 1].read().enable_schmitt(N & 0x1);
                unsafe { self.base.gpio_config[N >> 1].write(config) };
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N].read().enable_schmitt();
                unsafe { self.base.gpio_config[N].write(config) };
            } else {
                unimplemented!()
            }
        }
    }
    /// Disable schmitt trigger.
    #[inline]
    pub fn disable_schmitt(&mut self) {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_config[N >> 1].read().disable_schmitt(N & 0x1);
                unsafe { self.base.gpio_config[N >> 1].write(config) };
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N].read().disable_schmitt();
                unsafe { self.base.gpio_config[N].write(config) };
            } else {
                unimplemented!()
            }
        }
    }
    /// Clear interrupt flag.
    #[inline]
    pub fn clear_interrupt(&mut self) {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                unsafe { self.base.gpio_interrupt_clear.write(1 << N) };
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N].read().clear_interrupt();
                unsafe { self.base.gpio_config[N].write(config) };
            } else {
                unimplemented!()
            }
        }
    }
    /// Check if interrupt flag is set.
    #[inline]
    pub fn has_interrupt(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                self.base.gpio_interrupt_state.read() & (1 << N) != 0
            } else if #[cfg(feature = "glb-v2")] {
                self.base.gpio_config[N].read().has_interrupt()
            } else {
                unimplemented!()
            }
        }
    }
    /// Mask interrupt.
    #[inline]
    pub fn mask_interrupt(&mut self) {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_interrupt_mask.read() | (1 << N);
                unsafe { self.base.gpio_interrupt_mask.write(config) };
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N].read().mask_interrupt();
                unsafe { self.base.gpio_config[N].write(config) };
            } else {
                unimplemented!()
            }
        }
    }
    /// Unmask interrupt.
    #[inline]
    pub fn unmask_interrupt(&mut self) {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_interrupt_mask.read() & !(1 << N);
                unsafe { self.base.gpio_interrupt_mask.write(config) };
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N].read().unmask_interrupt();
                unsafe { self.base.gpio_config[N].write(config) };
            } else {
                unimplemented!()
            }
        }
    }
}

#[cfg(feature = "glb-v1")]
impl<A: BaseAddress, const N: usize, M> Pad<A, N, Input<M>> {
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

#[cfg(feature = "glb-v2")]
impl<A: BaseAddress, const N: usize, M> Pad<A, N, Input<M>> {
    /// Get interrupt mode.
    #[inline]
    pub fn interrupt_mode(&self) -> v2::InterruptMode {
        self.base.gpio_config[N].read().interrupt_mode()
    }
    /// Set interrupt mode.
    #[inline]
    pub fn set_interrupt_mode(&mut self, val: v2::InterruptMode) {
        let config = self.base.gpio_config[N].read().set_interrupt_mode(val);
        unsafe { self.base.gpio_config[N].write(config) };
    }
}

#[cfg(feature = "glb-v1")]
impl<A: BaseAddress, const N: usize, M> Pad<A, N, Output<M>> {
    /// Get drive strength of this pin.
    #[inline]
    pub fn drive(&self) -> v1::Drive {
        self.base.gpio_config[N >> 1].read().drive(N & 0x1)
    }
    /// Set drive strength of this pin.
    #[inline]
    pub fn set_drive(&mut self, val: v1::Drive) {
        let config = self.base.gpio_config[N >> 1].read().set_drive(N & 0x1, val);
        unsafe { self.base.gpio_config[N >> 1].write(config) };
    }
}

#[cfg(feature = "glb-v2")]
impl<A: BaseAddress, const N: usize, M> Pad<A, N, Output<M>> {
    /// Get drive strength of this pin.
    #[inline]
    pub fn drive(&self) -> v2::Drive {
        self.base.gpio_config[N].read().drive()
    }
    /// Set drive strength of this pin.
    #[inline]
    pub fn set_drive(&mut self, val: v2::Drive) {
        let config = self.base.gpio_config[N].read().set_drive(val);
        unsafe { self.base.gpio_config[N].write(config) };
    }
}

#[cfg(feature = "glb-v2")]
impl<A: BaseAddress, const N: usize, M: Alternate> Pad<A, N, M> {
    /// Configures the pin to operate as a SPI pin.
    #[inline]
    pub fn into_spi<const I: usize>(self) -> Pad<A, N, Spi<I>>
    where
        Spi<I>: Alternate,
    {
        let config = v2::GpioConfig::RESET_VALUE
            .enable_input()
            .disable_output()
            .enable_schmitt()
            .set_pull(v2::Pull::Up)
            .set_drive(v2::Drive::Drive0)
            .set_function(Spi::<I>::F);
        unsafe {
            self.base.gpio_config[N].write(config);
        }

        Pad {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

/// Serial Peripheral Interface mode (type state).
pub struct Spi<const F: usize>;

impl Alternate for Spi<0> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::Spi0;
}

impl Alternate for Spi<1> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::Spi1;
}

impl<A: BaseAddress, const N: usize, M: Alternate> Pad<A, N, M> {
    /// Configures the pin to operate as a pull up output pin.
    #[inline]
    pub fn into_pull_up_output(self) -> Pad<A, N, Output<PullUp>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_config[N >> 1]
                    .read()
                    .set_function(N & 0x1, v1::Function::Gpio)
                    .disable_input(N & 0x1)
                    .set_pull(N & 0x1, v1::Pull::Up);
                unsafe { self.base.gpio_config[N >> 1].write(config) };
                let val = self.base.gpio_output_enable.read();
                unsafe { self.base.gpio_output_enable.write(val | (1 << N)) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N]
                    .read()
                    .set_function(v2::Function::Gpio)
                    .set_mode(v2::Mode::SetClear)
                    .disable_input()
                    .enable_output()
                    .set_pull(v2::Pull::Up);
                unsafe { self.base.gpio_config[N].write(config) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else {
                unimplemented!()
            }
        }
    }
    /// Configures the pin to operate as a pull down output pin.
    #[inline]
    pub fn into_pull_down_output(self) -> Pad<A, N, Output<PullDown>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_config[N >> 1]
                    .read()
                    .set_function(N & 0x1, v1::Function::Gpio)
                    .disable_input(N & 0x1)
                    .set_pull(N & 0x1, v1::Pull::Down);
                unsafe { self.base.gpio_config[N >> 1].write(config) };
                let val = self.base.gpio_output_enable.read();
                unsafe { self.base.gpio_output_enable.write(val | (1 << N)) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N]
                    .read()
                    .set_function(v2::Function::Gpio)
                    .set_mode(v2::Mode::SetClear)
                    .disable_input()
                    .enable_output()
                    .set_pull(v2::Pull::Down);
                unsafe { self.base.gpio_config[N].write(config) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else {
                unimplemented!()
            }
        }
    }
    /// Configures the pin to operate as a floating output pin.
    #[inline]
    pub fn into_floating_output(self) -> Pad<A, N, Output<Floating>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_config[N >> 1]
                    .read()
                    .set_function(N & 0x1, v1::Function::Gpio)
                    .disable_input(N & 0x1)
                    .set_pull(N & 0x1, v1::Pull::None);
                unsafe { self.base.gpio_config[N >> 1].write(config) };
                let val = self.base.gpio_output_enable.read();
                unsafe { self.base.gpio_output_enable.write(val | (1 << N)) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N]
                    .read()
                    .set_function(v2::Function::Gpio)
                    .set_mode(v2::Mode::SetClear)
                    .disable_input()
                    .enable_output()
                    .set_pull(v2::Pull::None);
                unsafe { self.base.gpio_config[N].write(config) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else {
                unimplemented!()
            }
        }
    }
    /// Configures the pin to operate as a pull up input pin.
    #[inline]
    pub fn into_pull_up_input(self) -> Pad<A, N, Input<PullUp>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_config[N >> 1]
                    .read()
                    .set_function(N & 0x1, v1::Function::Gpio)
                    .enable_input(N & 0x1)
                    .set_pull(N & 0x1, v1::Pull::Up);
                unsafe { self.base.gpio_config[N >> 1].write(config) };
                let val = self.base.gpio_output_enable.read();
                unsafe { self.base.gpio_output_enable.write(val & !(1 << N)) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N]
                    .read()
                    .set_function(v2::Function::Gpio)
                    .set_mode(v2::Mode::SetClear)
                    .enable_input()
                    .disable_output()
                    .set_pull(v2::Pull::Up);
                unsafe { self.base.gpio_config[N].write(config) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else {
                unimplemented!()
            }
        }
    }
    /// Configures the pin to operate as a pull down input pin.
    #[inline]
    pub fn into_pull_down_input(self) -> Pad<A, N, Input<PullDown>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_config[N >> 1]
                    .read()
                    .set_function(N & 0x1, v1::Function::Gpio)
                    .enable_input(N & 0x1)
                    .set_pull(N & 0x1, v1::Pull::Down);
                unsafe { self.base.gpio_config[N >> 1].write(config) };
                let val = self.base.gpio_output_enable.read();
                unsafe { self.base.gpio_output_enable.write(val & !(1 << N)) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N]
                    .read()
                    .set_function(v2::Function::Gpio)
                    .set_mode(v2::Mode::SetClear)
                    .enable_input()
                    .disable_output()
                    .set_pull(v2::Pull::Down);
                unsafe { self.base.gpio_config[N].write(config) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else {
                unimplemented!()
            }
        }
    }
    /// Configures the pin to operate as a floating input pin.
    #[inline]
    pub fn into_floating_input(self) -> Pad<A, N, Input<Floating>> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "glb-v1")] {
                let config = self.base.gpio_config[N >> 1]
                    .read()
                    .set_function(N & 0x1, v1::Function::Gpio)
                    .enable_input(N & 0x1)
                    .set_pull(N & 0x1, v1::Pull::None);
                    unsafe { self.base.gpio_config[N >> 1].write(config) };
                let val = self.base.gpio_output_enable.read();
                unsafe { self.base.gpio_output_enable.write(val & !(1 << N)) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else if #[cfg(feature = "glb-v2")] {
                let config = self.base.gpio_config[N]
                    .read()
                    .set_function(v2::Function::Gpio)
                    .set_mode(v2::Mode::SetClear)
                    .enable_input()
                    .disable_output()
                    .set_pull(v2::Pull::None);
                unsafe { self.base.gpio_config[N].write(config) };
                Pad {
                    base: self.base,
                    _mode: PhantomData,
                }
            } else {
                unimplemented!()
            }
        }
    }
}

/// UART alternate (type state).
pub struct Uart;

impl Alternate for Uart {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::Uart;
}

#[cfg(any(doc, feature = "glb-v2"))]
const UART_GPIO_CONFIG: v2::GpioConfig = v2::GpioConfig::RESET_VALUE
    .enable_input()
    .enable_output()
    .enable_schmitt()
    .set_drive(v2::Drive::Drive0)
    .set_pull(v2::Pull::Up)
    .set_function(v2::Function::Uart);

impl<A: BaseAddress, const N: usize, M: Alternate> Pad<A, N, M> {
    /// Configures the pin to operate as UART signal.
    #[cfg(any(doc, feature = "glb-v2"))]
    #[inline]
    pub fn into_uart(self) -> Pad<A, N, Uart> {
        unsafe { self.base.gpio_config[N].write(UART_GPIO_CONFIG) };
        Pad {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

/// Multi-media cluster UART alternate (type state).
pub struct MmUart;

impl Alternate for MmUart {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::MmUart;
}

impl<A: BaseAddress, const N: usize, M: Alternate> Pad<A, N, M> {
    /// Configures the pin to operate as multi-media cluster UART signal.
    #[cfg(any(doc, feature = "glb-v2"))]
    #[inline]
    pub fn into_mm_uart(self) -> Pad<A, N, MmUart> {
        unsafe {
            self.base.gpio_config[N].write(UART_GPIO_CONFIG.set_function(v2::Function::MmUart))
        };
        Pad {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

/// Pulse Width Modulation signal mode (type state).
pub struct Pwm<const F: usize>;

impl Alternate for Pwm<0> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::Pwm0;
}

impl Alternate for Pwm<1> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::Pwm1;
}

impl<A: BaseAddress, const N: usize, M: Alternate> Pad<A, N, M> {
    /// Configures the pin to operate as a pull up Pulse Width Modulation signal pin.
    #[cfg(any(doc, feature = "glb-v2"))]
    #[inline]
    pub fn into_pull_up_pwm<const I: usize>(self) -> Pad<A, N, Pwm<I>>
    where
        Pwm<I>: Alternate,
    {
        let config = v2::GpioConfig::RESET_VALUE
            .disable_input()
            .enable_output()
            .enable_schmitt()
            .set_drive(v2::Drive::Drive0)
            .set_pull(v2::Pull::Up)
            .set_function(Pwm::<I>::F);
        unsafe { self.base.gpio_config[N].write(config) };
        Pad {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configures the pin to operate as a pull down Pulse Width Modulation signal pin.
    #[cfg(any(doc, feature = "glb-v2"))]
    #[inline]
    pub fn into_pull_down_pwm<const I: usize>(self) -> Pad<A, N, Pwm<I>>
    where
        Pwm<I>: Alternate,
    {
        let config = v2::GpioConfig::RESET_VALUE
            .disable_input()
            .enable_output()
            .enable_schmitt()
            .set_drive(v2::Drive::Drive0)
            .set_pull(v2::Pull::Down)
            .set_function(Pwm::<I>::F);
        unsafe { self.base.gpio_config[N].write(config) };
        Pad {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configures the pin to operate as floating Pulse Width Modulation signal pin.
    #[cfg(any(doc, feature = "glb-v2"))]
    #[inline]
    pub fn into_floating_pwm<const I: usize>(self) -> Pad<A, N, Pwm<I>>
    where
        Pwm<I>: Alternate,
    {
        let config = v2::GpioConfig::RESET_VALUE
            .disable_input()
            .enable_output()
            .enable_schmitt()
            .set_drive(v2::Drive::Drive0)
            .set_pull(v2::Pull::None)
            .set_function(Pwm::<I>::F);
        unsafe { self.base.gpio_config[N].write(config) };
        Pad {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

/// Inter-Integrated Circuit mode (type state).
pub struct I2c<const F: usize>;

impl Alternate for I2c<0> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::I2c0;
}

impl Alternate for I2c<1> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::I2c1;
}

impl Alternate for I2c<2> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::I2c2;
}

impl Alternate for I2c<3> {
    #[cfg(feature = "glb-v2")]
    const F: v2::Function = v2::Function::I2c3;
}

impl<A: BaseAddress, const N: usize, M: Alternate> Pad<A, N, M> {
    /// Configures the pin to operate as an Inter-Integrated Circuit signal pin.
    #[cfg(any(doc, feature = "glb-v2"))]
    #[inline]
    pub fn into_i2c<const I: usize>(self) -> Pad<A, N, I2c<I>>
    where
        I2c<I>: Alternate,
    {
        let config = v2::GpioConfig::RESET_VALUE
            .enable_input()
            .enable_output()
            .enable_schmitt()
            .set_drive(v2::Drive::Drive0)
            .set_pull(v2::Pull::Up)
            .set_function(I2c::<I>::F);
        unsafe {
            self.base.gpio_config[N].write(config);
        }
        Pad {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

/// Available GPIO pins.
pub struct Pads<A: BaseAddress> {
    // GPIO I/O 0.
    pub io0: Pad<A, 0, Disabled>,
    // GPIO I/O 1.
    pub io1: Pad<A, 1, Disabled>,
    // GPIO I/O 2.
    pub io2: Pad<A, 2, Disabled>,
    // GPIO I/O 3.
    pub io3: Pad<A, 3, Disabled>,
    // GPIO I/O 4.
    pub io4: Pad<A, 4, Disabled>,
    // GPIO I/O 5.
    pub io5: Pad<A, 5, Disabled>,
    // GPIO I/O 6.
    pub io6: Pad<A, 6, Disabled>,
    // GPIO I/O 7.
    pub io7: Pad<A, 7, Disabled>,
    // GPIO I/O 8.
    pub io8: Pad<A, 8, Disabled>,
    // GPIO I/O 9.
    pub io9: Pad<A, 9, Disabled>,
    // GPIO I/O 10.
    pub io10: Pad<A, 10, Disabled>,
    // GPIO I/O 11.
    pub io11: Pad<A, 11, Disabled>,
    // GPIO I/O 12.
    pub io12: Pad<A, 12, Disabled>,
    // GPIO I/O 13.
    pub io13: Pad<A, 13, Disabled>,
    // GPIO I/O 14.
    pub io14: Pad<A, 14, Disabled>,
    // GPIO I/O 15.
    pub io15: Pad<A, 15, Disabled>,
    // GPIO I/O 16.
    pub io16: Pad<A, 16, Disabled>,
    // GPIO I/O 17.
    pub io17: Pad<A, 17, Disabled>,
    // GPIO I/O 18.
    pub io18: Pad<A, 18, Disabled>,
    // GPIO I/O 19.
    pub io19: Pad<A, 19, Disabled>,
    // GPIO I/O 20.
    pub io20: Pad<A, 20, Disabled>,
    // GPIO I/O 21.
    pub io21: Pad<A, 21, Disabled>,
    // GPIO I/O 22.
    pub io22: Pad<A, 22, Disabled>,
    // GPIO I/O 23.
    pub io23: Pad<A, 23, Disabled>,
    // GPIO I/O 24.
    pub io24: Pad<A, 24, Disabled>,
    // GPIO I/O 25.
    pub io25: Pad<A, 25, Disabled>,
    // GPIO I/O 26.
    pub io26: Pad<A, 26, Disabled>,
    // GPIO I/O 27.
    pub io27: Pad<A, 27, Disabled>,
    // GPIO I/O 28.
    pub io28: Pad<A, 28, Disabled>,
}
