//! Universal Asynchronous Receiver/Transmitter.
use crate::clocks::Clocks;
use crate::glb::{self, v2::UartSignal};
use crate::gpio::{MmUart, Pad, Uart};
use core::marker::PhantomData;
use core::ops::Deref;
use embedded_time::rate::{Baud, Extensions};
use volatile_register::{RO, RW, WO};

/// Universal Asynchronous Receiver/Transmitter registers.
#[repr(C)]
pub struct RegisterBlock {
    /// Transmit configuration.
    pub transmit_config: RW<TransmitConfig>,
    /// Receive configuration.
    pub receive_config: RW<ReceiveConfig>,
    /// Bit-period in clocks.
    pub bit_period: RW<BitPeriod>,
    /// Data format configuration.
    pub data_config: RW<DataConfig>,
    _reserved1: [u8; 0x10],
    /// Interrupt state register.
    pub interrupt_state: RO<InterruptState>,
    /// Interrupt mask register.
    pub interrupt_mask: RW<InterruptMask>,
    /// Clear interrupt register.
    pub interrupt_clear: WO<InterruptClear>,
    /// Interrupt enable register.
    pub interrupt_enable: RW<InterruptEnable>,
    /// Bus state.
    pub bus_state: RO<BusState>,
    _reserved2: [u8; 0x4c],
    /// First-in first-out queue configuration 0.
    pub fifo_config_0: RW<FifoConfig0>,
    /// First-in first-out queue configuration 1.
    pub fifo_config_1: RW<FifoConfig1>,
    /// Write data into first-in first-out queue.
    pub fifo_write: WO<u8>,
    _reserved3: [u8; 0x3],
    /// Read data from first-in first-out queue.
    pub fifo_read: RO<u8>,
}

/// Transmit configuration register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct TransmitConfig(u32);

// TODO: inherent associated types is unstable, put aliases here as WAR
/// Register fields aliases, defining the bit field shift and bit length
mod transmit_config {
    use crate::BitField;

    pub(crate) type Enable = BitField<1, 0, u32>;
    pub(crate) type ParityEnable = BitField<1, 4, u32>;
    pub(crate) type ParityMode = BitField<1, 5, u32>;
    pub(crate) type WordLength = BitField<3, 8, u32>;
}

impl TransmitConfig {
    const CTS: u32 = 1 << 1;
    const FREERUN: u32 = 1 << 2;
    const LIN_TRANSMIT: u32 = 1 << 3;
    const IR_TRANSMIT: u32 = 1 << 6;
    const IR_INVERSE: u32 = 1 << 7;
    const STOP_BITS: u32 = 0b11 << 11;
    const LIN_BREAK_BITS: u32 = 0b111 << 13;
    const TRANSFER_LENGTH: u32 = 0xffff << 16;

    /// Enable transmit.
    #[inline]
    pub const fn enable_txd(self) -> Self {
        Self(transmit_config::Enable::from(self.0).enable())
    }
    /// Disable transmit.
    #[inline]
    pub const fn disable_txd(self) -> Self {
        Self(transmit_config::Enable::from(self.0).disable())
    }
    /// Check if transmit is enabled.
    #[inline]
    pub const fn is_txd_enabled(self) -> bool {
        transmit_config::Enable::from(self.0).is_enabled()
    }
    /// Enable Clear-to-Send signal.
    #[inline]
    pub const fn enable_cts(self) -> Self {
        Self(self.0 | Self::CTS)
    }
    /// Disable Clear-to-Send signal.
    #[inline]
    pub const fn disable_cts(self) -> Self {
        Self(self.0 & !Self::CTS)
    }
    /// Check if Clear-to-Send signal is enabled.
    #[inline]
    pub const fn is_cts_enabled(self) -> bool {
        self.0 & Self::CTS != 0
    }
    /// Enable free-run mode.
    #[inline]
    pub const fn enable_freerun(self) -> Self {
        Self(self.0 | Self::FREERUN)
    }
    /// Disable free-run mode.
    #[inline]
    pub const fn disable_freerun(self) -> Self {
        Self(self.0 & !Self::FREERUN)
    }
    /// Check if free-run mode is enabled.
    #[inline]
    pub const fn is_freerun_enabled(self) -> bool {
        self.0 & Self::FREERUN != 0
    }
    /// Enable LIN protocol transmission.
    #[inline]
    pub const fn enable_lin_transmit(self) -> Self {
        Self(self.0 | Self::LIN_TRANSMIT)
    }
    /// Disable LIN protocol transmission.
    #[inline]
    pub const fn disable_lin_transmit(self) -> Self {
        Self(self.0 & !Self::LIN_TRANSMIT)
    }
    /// Check if LIN protocol transmission is enabled.
    #[inline]
    pub const fn is_lin_transmit_enabled(self) -> bool {
        self.0 & Self::LIN_TRANSMIT != 0
    }
    /// Set parity check mode.
    #[inline]
    pub const fn set_parity(self, parity: Parity) -> Self {
        let field_en = transmit_config::ParityEnable::from(self.0);

        match parity {
            Parity::Even => {
                let field_odd = transmit_config::ParityMode::from(field_en.enable());
                Self(field_odd.disable())
            }
            Parity::Odd => {
                let field_odd = transmit_config::ParityMode::from(field_en.enable());
                Self(field_odd.enable())
            }
            Parity::None => Self(field_en.disable()),
        }
    }
    /// Get parity check mode.
    #[inline]
    pub const fn parity(self) -> Parity {
        let field_en = transmit_config::ParityEnable::from(self.0);
        let field_odd = transmit_config::ParityMode::from(self.0);

        if !field_en.is_enabled() {
            Parity::None
        } else if !field_odd.is_enabled() {
            Parity::Even
        } else {
            Parity::Odd
        }
    }
    /// Enable IR transmission.
    #[inline]
    pub const fn enable_ir_transmit(self) -> Self {
        Self(self.0 | Self::IR_TRANSMIT)
    }
    /// Disable IR transmission.
    #[inline]
    pub const fn disable_ir_transmit(self) -> Self {
        Self(self.0 & !Self::IR_TRANSMIT)
    }
    /// Check if IR transmission is enabled.
    #[inline]
    pub const fn is_ir_transmit_enabled(self) -> bool {
        self.0 & Self::IR_TRANSMIT != 0
    }
    /// Invert transmit signal output in IR mode.
    #[inline]
    pub const fn enable_ir_inverse(self) -> Self {
        Self(self.0 | Self::IR_INVERSE)
    }
    /// Don't invert transmit signal output in IR mode.
    #[inline]
    pub const fn disable_ir_inverse(self) -> Self {
        Self(self.0 & !Self::IR_INVERSE)
    }
    /// Check if transmit signal output in IR mode is inverted.
    #[inline]
    pub const fn is_ir_inverse_enabled(self) -> bool {
        self.0 & Self::IR_INVERSE != 0
    }
    /// Set word length.
    #[inline]
    pub const fn set_word_length(self, val: WordLength) -> Self {
        let field = transmit_config::WordLength::from(self.0);
        let val = match val {
            WordLength::Five => 4,
            WordLength::Six => 5,
            WordLength::Seven => 6,
            WordLength::Eight => 7,
        };
        Self(field.set(val))
    }
    /// Get word length.
    #[inline]
    pub const fn word_length(self) -> WordLength {
        let field = transmit_config::WordLength::from(self.0);
        match field.get() {
            4 => WordLength::Five,
            5 => WordLength::Six,
            6 => WordLength::Seven,
            7 => WordLength::Eight,
            _ => unreachable!(),
        }
    }
    /// Set stop-bit configuration.
    #[inline]
    pub const fn set_stop_bits(self, val: StopBits) -> Self {
        let val = match val {
            StopBits::ZeroPointFive => 0,
            StopBits::One => 1,
            StopBits::OnePointFive => 2,
            StopBits::Two => 3,
        };
        Self(self.0 & !Self::STOP_BITS | val << 11)
    }
    /// Get stop-bit configuration.
    #[inline]
    pub const fn stop_bits(self) -> StopBits {
        let val = (self.0 & Self::STOP_BITS) >> 11;
        match val {
            0 => StopBits::ZeroPointFive,
            1 => StopBits::One,
            2 => StopBits::OnePointFive,
            3 => StopBits::Two,
            _ => unreachable!(),
        }
    }
    /// Set synchronize interval under LIN mode.
    ///
    /// # Parameters
    ///
    /// * `bits` - Interval in bits, the value should be 0 ~ 7.
    #[inline]
    pub const fn set_lin_break_bits(self, bits: u8) -> Self {
        Self(self.0 & !Self::LIN_BREAK_BITS | (bits as u32) << 13)
    }
    /// Get synchronize interval under LIN mode.
    ///
    /// Return value is 0 ~ 7, represent in bits.
    #[inline]
    pub const fn lin_break_bits(self) -> u8 {
        ((self.0 & Self::LIN_BREAK_BITS) >> 13) as u8
    }
    /// Trigger interrupt when specified length of data is sent.
    ///
    /// NOTE: This bit is not valid when it is running under free-run mode.
    #[inline]
    pub const fn set_transfer_length(self, length: u16) -> Self {
        Self(self.0 & !Self::TRANSFER_LENGTH | (length as u32) << 16)
    }
    /// Get the length of data that triggers the interrupt.
    #[inline]
    pub const fn transfer_length(self) -> u16 {
        ((self.0 & Self::TRANSFER_LENGTH) >> 16) as u16
    }
}

impl Default for TransmitConfig {
    #[inline]
    fn default() -> Self {
        Self(0x0000_8f00)
    }
}

/// Receive configuration register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct ReceiveConfig(u32);

mod receive_config {
    use crate::BitField;

    pub(crate) type Enable = BitField<1, 0, u32>;
    pub(crate) type ParityEnable = BitField<1, 4, u32>;
    pub(crate) type ParityMode = BitField<1, 5, u32>;
    pub(crate) type WordLength = BitField<3, 8, u32>;
}

impl ReceiveConfig {
    const ABR: u32 = 1 << 1;
    const LIN_RECEIVE: u32 = 1 << 3;
    const IR_RECEIVE: u32 = 1 << 6;
    const IR_INVERSE: u32 = 1 << 7;
    const DEGLICH: u32 = 1 << 11;
    const DEGLICH_CYCLE: u32 = 0xf << 12;
    const TRANSFER_LENGTH: u32 = 0xffff << 16;

    /// Enable receive.
    #[inline]
    pub const fn enable_rxd(self) -> Self {
        Self(receive_config::Enable::from(self.0).enable())
    }
    /// Disable receive.
    #[inline]
    pub const fn disable_rxd(self) -> Self {
        Self(receive_config::Enable::from(self.0).disable())
    }
    /// Check if receive is enabled.
    #[inline]
    pub const fn is_rxd_enabled(self) -> bool {
        receive_config::Enable::from(self.0).is_enabled()
    }
    /// Enable auto baud rate detection.
    #[inline]
    pub const fn enable_auto_baudrate(self) -> Self {
        Self(self.0 | Self::ABR)
    }
    /// Disable auto baud rate detection.
    #[inline]
    pub const fn disable_auto_baudrate(self) -> Self {
        Self(self.0 & !Self::ABR)
    }
    /// Check if auto baud rate detection is enabled.
    #[inline]
    pub const fn is_auto_baudrate_enabled(self) -> bool {
        self.0 & Self::ABR != 0
    }
    /// Enable LIN protocol receive.
    #[inline]
    pub const fn enable_lin_receive(self) -> Self {
        Self(self.0 | Self::LIN_RECEIVE)
    }
    /// Disable LIN protocol receive.
    #[inline]
    pub const fn disable_lin_receive(self) -> Self {
        Self(self.0 & !Self::LIN_RECEIVE)
    }
    /// Check if LIN protocol receive is enabled.
    #[inline]
    pub const fn is_lin_receive_enabled(self) -> bool {
        self.0 & Self::LIN_RECEIVE != 0
    }
    /// Set parity check mode.
    #[inline]
    pub const fn set_parity(self, parity: Parity) -> Self {
        let field_en = receive_config::ParityEnable::from(self.0);

        match parity {
            Parity::Even => {
                let field_odd = receive_config::ParityMode::from(field_en.enable());
                Self(field_odd.disable())
            }
            Parity::Odd => {
                let field_odd = receive_config::ParityMode::from(field_en.enable());
                Self(field_odd.enable())
            }
            Parity::None => Self(field_en.disable()),
        }
    }
    /// Get parity check mode.
    #[inline]
    pub const fn parity(self) -> Parity {
        let field_en = receive_config::ParityEnable::from(self.0);
        let field_odd = receive_config::ParityMode::from(self.0);

        if !field_en.is_enabled() {
            Parity::None
        } else if !field_odd.is_enabled() {
            Parity::Even
        } else {
            Parity::Odd
        }
    }
    /// Enable IR receive.
    #[inline]
    pub const fn enable_ir_receive(self) -> Self {
        Self(self.0 | Self::IR_RECEIVE)
    }
    /// Disable IR receive.
    #[inline]
    pub const fn disable_ir_receive(self) -> Self {
        Self(self.0 & !Self::IR_RECEIVE)
    }
    /// Check if IR receive is enabled.
    #[inline]
    pub const fn is_ir_receive_enabled(self) -> bool {
        self.0 & Self::IR_RECEIVE != 0
    }
    /// Invert receive signal output in IR mode.
    #[inline]
    pub const fn enable_ir_inverse(self) -> Self {
        Self(self.0 | Self::IR_INVERSE)
    }
    /// Don't invert receive signal output in IR mode.
    #[inline]
    pub const fn disable_ir_inverse(self) -> Self {
        Self(self.0 & !Self::IR_INVERSE)
    }
    /// Check if receive signal output in IR mode is inverted.
    #[inline]
    pub const fn is_ir_inverse_enabled(self) -> bool {
        self.0 & Self::IR_INVERSE != 0
    }
    /// Set word length.
    #[inline]
    pub const fn set_word_length(self, val: WordLength) -> Self {
        let field = receive_config::WordLength::from(self.0);
        let val = match val {
            WordLength::Five => 4,
            WordLength::Six => 5,
            WordLength::Seven => 6,
            WordLength::Eight => 7,
        };
        Self(field.set(val))
    }
    /// Get word length.
    #[inline]
    pub const fn word_length(self) -> WordLength {
        let field = receive_config::WordLength::from(self.0);
        match field.get() {
            4 => WordLength::Five,
            5 => WordLength::Six,
            6 => WordLength::Seven,
            7 => WordLength::Eight,
            _ => unreachable!(),
        }
    }
    /// Enable de-glitch function.
    #[inline]
    pub const fn enable_deglitch(self) -> Self {
        Self(self.0 | Self::DEGLICH)
    }
    /// Disable de-glitch function.
    #[inline]
    pub const fn disable_deglitch(self) -> Self {
        Self(self.0 & !Self::DEGLICH)
    }
    /// Check if de-glitch function is enabled.
    #[inline]
    pub const fn is_deglitch_enabled(self) -> bool {
        self.0 & Self::DEGLICH != 0
    }
    /// Set de-glich function cycle count.
    #[inline]
    pub const fn set_deglitch_cycles(self, val: u8) -> Self {
        Self(self.0 & !Self::DEGLICH_CYCLE | ((val as u32) << 12))
    }
    /// Get de-glich function cycle count.
    #[inline]
    pub const fn deglitch_cycles(self) -> u8 {
        ((self.0 & Self::DEGLICH_CYCLE) >> 12) as u8
    }
    /// Set the length of data that triggers the interrupt.
    #[inline]
    pub const fn set_transfer_length(self, length: u16) -> Self {
        Self(self.0 & !Self::TRANSFER_LENGTH | (length as u32) << 16)
    }
    /// Get the length of data that triggers the interrupt.
    #[inline]
    pub const fn transfer_length(self) -> u16 {
        ((self.0 & Self::TRANSFER_LENGTH) >> 16) as u16
    }
}

impl Default for ReceiveConfig {
    #[inline]
    fn default() -> Self {
        Self(0x0000_0700)
    }
}

/// Bit period configuration register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct BitPeriod(u32);

impl BitPeriod {
    const TRANSMIT: u32 = 0xffff;
    const RECEIVE: u32 = 0xffff << 16;

    /// Set transmit time interval.
    #[inline]
    pub const fn set_transmit_time_interval(self, val: u16) -> Self {
        Self(self.0 & !Self::TRANSMIT | val as u32)
    }
    /// Get transmit time interval.
    #[inline]
    pub const fn transmit_time_interval(self) -> u16 {
        (self.0 & Self::TRANSMIT) as u16
    }
    /// Set receive time interval.
    #[inline]
    pub const fn set_receive_time_interval(self, val: u16) -> Self {
        Self(self.0 & !Self::RECEIVE | ((val as u32) << 16))
    }
    /// Get receive time interval.
    #[inline]
    pub const fn receive_time_interval(self) -> u16 {
        ((self.0 & Self::RECEIVE) >> 16) as u16
    }
}

impl Default for BitPeriod {
    #[inline]
    fn default() -> Self {
        Self(0x00ff_00ff)
    }
}

/// Data configuration register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct DataConfig(u32);

impl DataConfig {
    const BIT_ORDER: u32 = 1 << 0;

    /// Set the bit order in each data word.
    #[inline]
    pub const fn set_bit_order(self, val: BitOrder) -> Self {
        match val {
            BitOrder::LsbFirst => Self(self.0 & !Self::BIT_ORDER),
            BitOrder::MsbFirst => Self(self.0 | Self::BIT_ORDER),
        }
    }
    /// Get the bit order in each data word.
    #[inline]
    pub const fn bit_order(self) -> BitOrder {
        if self.0 & Self::BIT_ORDER == 0 {
            BitOrder::LsbFirst
        } else {
            BitOrder::MsbFirst
        }
    }
}

impl Default for DataConfig {
    #[inline]
    fn default() -> Self {
        Self(0x0000_0000)
    }
}

/// Interrupt event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Interrupt {
    TransmitEnd = 0,
    ReceiveEnd = 1,
    TransmitFifoReady = 2,
    ReceiveFifoReady = 3,
    ReceiveTimeout = 4,
    ReceiveParityError = 5,
    TransmitFifoError = 6,
    ReceiveFifoError = 7,
    ReceiveSyncError = 8,
    ReceiveByteCountReached = 9,
    ReceiveAutoBaudrateByStartBit = 10,
    ReceiveAutoBaudrateByFiveFive = 11,
}

/// Interrupt state register.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct InterruptState(u32);

impl InterruptState {
    /// Check if there is an interrupt flag.
    #[inline]
    pub const fn has_interrupt(self, val: Interrupt) -> bool {
        (self.0 & (1 << (val as u32))) != 0
    }
}

/// Interrupt mask register.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct InterruptMask(u32);

impl InterruptMask {
    /// Set interrupt mask.
    #[inline]
    pub const fn mask_interrupt(self, val: Interrupt) -> Self {
        Self(self.0 | (1 << (val as u32)))
    }
    /// Clear interrupt mask.
    #[inline]
    pub const fn unmask_interrupt(self, val: Interrupt) -> Self {
        Self(self.0 & !(1 << (val as u32)))
    }
    /// Check if interrupt is masked.
    #[inline]
    pub const fn is_interrupt_masked(self, val: Interrupt) -> bool {
        (self.0 & (1 << (val as u32))) != 0
    }
}

/// Interrupt clear register.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct InterruptClear(u32);

impl InterruptClear {
    /// Clear interrupt.
    #[inline]
    pub const fn clear_interrupt(self, val: Interrupt) -> Self {
        Self(self.0 | (1 << (val as u32)))
    }
}

/// Interrupt enable register.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct InterruptEnable(u32);

impl InterruptEnable {
    /// Enable interrupt.
    #[inline]
    pub const fn enable_interrupt(self, val: Interrupt) -> Self {
        Self(self.0 | (1 << (val as u32)))
    }
    /// Disable interrupt.
    #[inline]
    pub const fn disable_interrupt(self, val: Interrupt) -> Self {
        Self(self.0 & !(1 << (val as u32)))
    }
    /// Check if interrupt is enabled.
    #[inline]
    pub const fn is_interrupt_enabled(self, val: Interrupt) -> bool {
        (self.0 & (1 << (val as u32))) != 0
    }
}

/// Bus state register.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct BusState(u32);

impl BusState {
    const TRANSMIT_BUSY: u32 = 1 << 0;
    const RECEIVE_BUSY: u32 = 1 << 1;

    /// Get if UART transmit bus is busy.
    #[inline]
    pub const fn transmit_busy(self) -> bool {
        self.0 & Self::TRANSMIT_BUSY != 0
    }
    /// Get if UART receive bus is busy.
    #[inline]
    pub const fn receive_busy(self) -> bool {
        self.0 & Self::RECEIVE_BUSY != 0
    }
}

/// First-in first-out queue configuration 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct FifoConfig0(u32);

impl FifoConfig0 {
    const TRANSMIT_DMA_ENABLE: u32 = 1 << 0;
    const RECEIVE_DMA_ENABLE: u32 = 1 << 1;
    const TRANSMIT_FIFO_CLEAR: u32 = 1 << 2;
    const RECEIVE_FIFO_CLEAR: u32 = 1 << 3;
    const TRANSMIT_FIFO_OVERFLOW: u32 = 1 << 4;
    const TRANSMIT_FIFO_UNDERFLOW: u32 = 1 << 5;
    const RECEIVE_FIFO_OVERFLOW: u32 = 1 << 6;
    const RECEIVE_FIFO_UNDERFLOW: u32 = 1 << 7;

    /// Enable transmit DMA.
    #[inline]
    pub const fn enable_transmit_dma(self) -> Self {
        Self(self.0 | Self::TRANSMIT_DMA_ENABLE)
    }
    /// Disable transmit DMA.
    #[inline]
    pub const fn disable_transmit_dma(self) -> Self {
        Self(self.0 & !Self::TRANSMIT_DMA_ENABLE)
    }
    /// Check if transmit DMA is enabled.
    #[inline]
    pub const fn is_transmit_dma_enabled(self) -> bool {
        self.0 & Self::TRANSMIT_DMA_ENABLE != 0
    }
    /// Enable receive DMA.
    #[inline]
    pub const fn enable_receive_dma(self) -> Self {
        Self(self.0 | Self::RECEIVE_DMA_ENABLE)
    }
    /// Disable receive DMA.
    #[inline]
    pub const fn disable_receive_dma(self) -> Self {
        Self(self.0 & !Self::RECEIVE_DMA_ENABLE)
    }
    /// Check if receive DMA is enabled.
    #[inline]
    pub const fn is_receive_dma_enabled(self) -> bool {
        self.0 & Self::RECEIVE_DMA_ENABLE != 0
    }
    /// Clear transmit FIFO.
    #[inline]
    pub const fn clear_transmit_fifo(self) -> Self {
        Self(self.0 | Self::TRANSMIT_FIFO_CLEAR)
    }
    /// Clear receive FIFO.
    #[inline]
    pub const fn clear_receive_fifo(self) -> Self {
        Self(self.0 | Self::RECEIVE_FIFO_CLEAR)
    }
    /// Check if transmit FIFO is overflow.
    #[inline]
    pub const fn transmit_fifo_overflow(self) -> bool {
        self.0 & Self::TRANSMIT_FIFO_OVERFLOW != 0
    }
    /// Check if transmit FIFO is underflow.
    #[inline]
    pub const fn transmit_fifo_underflow(self) -> bool {
        self.0 & Self::TRANSMIT_FIFO_UNDERFLOW != 0
    }
    /// Check if receive FIFO is overflow.
    #[inline]
    pub const fn receive_fifo_overflow(self) -> bool {
        self.0 & Self::RECEIVE_FIFO_OVERFLOW != 0
    }
    /// Check if receive FIFO is underflow.
    #[inline]
    pub const fn receive_fifo_underflow(self) -> bool {
        self.0 & Self::RECEIVE_FIFO_UNDERFLOW != 0
    }
}

/// First-in first-out queue configuration 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct FifoConfig1(u32);

impl FifoConfig1 {
    const TRANSMIT_COUNT: u32 = 0x3f;
    const RECEIVE_COUNT: u32 = 0x3f << 8;
    const TRANSMIT_THRESHOLD: u32 = 0x1f << 16;
    const RECEIVE_THRESHOLD: u32 = 0x1f << 24;

    /// Get number of empty spaces remained in transmit FIFO queue.
    #[inline]
    pub const fn transmit_available_bytes(self) -> u8 {
        (self.0 & Self::TRANSMIT_COUNT) as u8
    }
    /// Get number of available bytes received in receive FIFO queue.
    #[inline]
    pub const fn receive_available_bytes(self) -> u8 {
        ((self.0 & Self::RECEIVE_COUNT) >> 8) as u8
    }
    /// Set transmit FIFO threshold.
    #[inline]
    pub const fn set_transmit_threshold(self, val: u8) -> Self {
        Self(self.0 & !Self::TRANSMIT_THRESHOLD | ((val as u32) << 16))
    }
    /// Get transmit FIFO threshold.
    #[inline]
    pub const fn transmit_threshold(self) -> u8 {
        ((self.0 & Self::TRANSMIT_THRESHOLD) >> 16) as u8
    }
    /// Set receive FIFO threshold.
    #[inline]
    pub const fn set_receive_threshold(self, val: u8) -> Self {
        Self(self.0 & !Self::RECEIVE_THRESHOLD | ((val as u32) << 24))
    }
    /// Get receive FIFO threshold.
    #[inline]
    pub const fn receive_threshold(self) -> u8 {
        ((self.0 & Self::RECEIVE_THRESHOLD) >> 24) as u8
    }
}

/// Multiplex to Request-to-Send (type state).
pub struct MuxRts<const I: usize>;

/// Multiplex to Clear-to-Send (type state).
pub struct MuxCts<const I: usize>;

/// Multiplex to Transmit (type state).
pub struct MuxTxd<const I: usize>;

/// Multiplex to Receive (type state).
pub struct MuxRxd<const I: usize>;

impl<const I: usize> MuxRts<I> {
    #[inline]
    fn signal() -> UartSignal {
        match I {
            0 => UartSignal::Rts0,
            1 => UartSignal::Rts1,
            2 => UartSignal::Rts2,
            _ => unreachable!(),
        }
    }
}

impl<const I: usize> MuxCts<I> {
    #[inline]
    fn signal() -> UartSignal {
        match I {
            0 => UartSignal::Cts0,
            1 => UartSignal::Cts1,
            2 => UartSignal::Cts2,
            _ => unreachable!(),
        }
    }
}

impl<const I: usize> MuxTxd<I> {
    #[inline]
    fn signal() -> UartSignal {
        match I {
            0 => UartSignal::Txd0,
            1 => UartSignal::Txd1,
            2 => UartSignal::Txd2,
            _ => unreachable!(),
        }
    }
}

impl<const I: usize> MuxRxd<I> {
    #[inline]
    fn signal() -> UartSignal {
        match I {
            0 => UartSignal::Rxd0,
            1 => UartSignal::Rxd1,
            2 => UartSignal::Rxd2,
            _ => unreachable!(),
        }
    }
}

/// Global peripheral UART signal multiplexer.
///
/// This structure only owns the GLB signal multiplexer for signal number `N`.
pub struct UartMux<GLB, const N: usize, M> {
    base: GLB,
    _mode: PhantomData<M>,
}

impl<GLB: Deref<Target = glb::v2::RegisterBlock>, const N: usize, M> UartMux<GLB, N, M> {
    /// Configure the internal UART signal to Request-to-Send (RTS).
    #[inline]
    pub fn into_request_to_send<const U: usize>(self) -> UartMux<GLB, N, MuxRts<U>> {
        let config = self.base.uart_mux_group[N >> 3]
            .read()
            .set_signal(N & 0x7, MuxRts::<U>::signal());
        unsafe { self.base.uart_mux_group[N >> 3].write(config) };
        UartMux {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configure the internal UART signal to Transmit (TXD).
    #[inline]
    pub fn into_transmit<const U: usize>(self) -> UartMux<GLB, N, MuxTxd<U>> {
        let config = self.base.uart_mux_group[N >> 3]
            .read()
            .set_signal(N & 0x7, MuxTxd::<U>::signal());
        unsafe { self.base.uart_mux_group[N >> 3].write(config) };
        UartMux {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configure the internal UART signal to Receive (RXD).
    #[inline]
    pub fn into_receive<const U: usize>(self) -> UartMux<GLB, N, MuxRxd<U>> {
        let config = self.base.uart_mux_group[N >> 3]
            .read()
            .set_signal(N & 0x7, MuxRxd::<U>::signal());
        unsafe { self.base.uart_mux_group[N >> 3].write(config) };
        UartMux {
            base: self.base,
            _mode: PhantomData,
        }
    }
    /// Configure the internal UART signal to Clear-to-Send (CTS).
    #[inline]
    pub fn into_clear_to_send<const U: usize>(self) -> UartMux<GLB, N, MuxCts<U>> {
        let config = self.base.uart_mux_group[N >> 3]
            .read()
            .set_signal(N & 0x7, MuxCts::<U>::signal());
        unsafe { self.base.uart_mux_group[N >> 3].write(config) };
        UartMux {
            base: self.base,
            _mode: PhantomData,
        }
    }
}

/// Available UART signal multiplexers.
pub struct UartMuxes<GLB> {
    /// Multiplexer of UART signal 0.
    pub sig0: UartMux<GLB, 0, MuxRts<0>>,
    /// Multiplexer of UART signal 1.
    pub sig1: UartMux<GLB, 1, MuxRts<0>>,
    /// Multiplexer of UART signal 2.
    pub sig2: UartMux<GLB, 2, MuxRts<0>>,
    /// Multiplexer of UART signal 3.
    pub sig3: UartMux<GLB, 3, MuxRts<0>>,
    /// Multiplexer of UART signal 4.
    pub sig4: UartMux<GLB, 4, MuxRts<0>>,
    /// Multiplexer of UART signal 5.
    pub sig5: UartMux<GLB, 5, MuxRts<0>>,
    /// Multiplexer of UART signal 6.
    pub sig6: UartMux<GLB, 6, MuxRts<0>>,
    /// Multiplexer of UART signal 7.
    pub sig7: UartMux<GLB, 7, MuxRts<0>>,
    /// Multiplexer of UART signal 8.
    pub sig8: UartMux<GLB, 8, MuxRts<0>>,
    /// Multiplexer of UART signal 9.
    pub sig9: UartMux<GLB, 9, MuxRts<0>>,
    /// Multiplexer of UART signal 10.
    pub sig10: UartMux<GLB, 10, MuxRts<0>>,
    /// Multiplexer of UART signal 11.
    pub sig11: UartMux<GLB, 11, MuxRts<0>>,
}

/// Check if target gpio `Pin` is internally connected to UART signal index `I`.
pub trait HasUartSignal<const I: usize> {}

impl<GLB> HasUartSignal<0> for Pad<GLB, 0, Uart> {}
impl<GLB> HasUartSignal<1> for Pad<GLB, 1, Uart> {}
impl<GLB> HasUartSignal<2> for Pad<GLB, 2, Uart> {}
impl<GLB> HasUartSignal<3> for Pad<GLB, 3, Uart> {}
impl<GLB> HasUartSignal<4> for Pad<GLB, 4, Uart> {}
impl<GLB> HasUartSignal<5> for Pad<GLB, 5, Uart> {}
impl<GLB> HasUartSignal<6> for Pad<GLB, 6, Uart> {}
impl<GLB> HasUartSignal<7> for Pad<GLB, 7, Uart> {}
impl<GLB> HasUartSignal<8> for Pad<GLB, 8, Uart> {}
impl<GLB> HasUartSignal<9> for Pad<GLB, 9, Uart> {}
impl<GLB> HasUartSignal<10> for Pad<GLB, 10, Uart> {}
impl<GLB> HasUartSignal<11> for Pad<GLB, 11, Uart> {}
impl<GLB> HasUartSignal<0> for Pad<GLB, 12, Uart> {}
impl<GLB> HasUartSignal<1> for Pad<GLB, 13, Uart> {}
impl<GLB> HasUartSignal<2> for Pad<GLB, 14, Uart> {}
impl<GLB> HasUartSignal<3> for Pad<GLB, 15, Uart> {}
impl<GLB> HasUartSignal<4> for Pad<GLB, 16, Uart> {}
impl<GLB> HasUartSignal<5> for Pad<GLB, 17, Uart> {}
impl<GLB> HasUartSignal<6> for Pad<GLB, 18, Uart> {}
impl<GLB> HasUartSignal<7> for Pad<GLB, 19, Uart> {}
impl<GLB> HasUartSignal<8> for Pad<GLB, 20, Uart> {}
impl<GLB> HasUartSignal<9> for Pad<GLB, 21, Uart> {}
impl<GLB> HasUartSignal<10> for Pad<GLB, 22, Uart> {}
impl<GLB> HasUartSignal<11> for Pad<GLB, 23, Uart> {}
impl<GLB> HasUartSignal<0> for Pad<GLB, 24, Uart> {}
impl<GLB> HasUartSignal<1> for Pad<GLB, 25, Uart> {}
impl<GLB> HasUartSignal<2> for Pad<GLB, 26, Uart> {}
impl<GLB> HasUartSignal<3> for Pad<GLB, 27, Uart> {}
impl<GLB> HasUartSignal<4> for Pad<GLB, 28, Uart> {}
impl<GLB> HasUartSignal<5> for Pad<GLB, 29, Uart> {}
impl<GLB> HasUartSignal<6> for Pad<GLB, 30, Uart> {}
impl<GLB> HasUartSignal<7> for Pad<GLB, 31, Uart> {}
impl<GLB> HasUartSignal<8> for Pad<GLB, 32, Uart> {}
impl<GLB> HasUartSignal<9> for Pad<GLB, 33, Uart> {}
impl<GLB> HasUartSignal<10> for Pad<GLB, 34, Uart> {}
impl<GLB> HasUartSignal<11> for Pad<GLB, 35, Uart> {}
impl<GLB> HasUartSignal<0> for Pad<GLB, 36, Uart> {}
impl<GLB> HasUartSignal<1> for Pad<GLB, 37, Uart> {}
impl<GLB> HasUartSignal<2> for Pad<GLB, 38, Uart> {}
impl<GLB> HasUartSignal<3> for Pad<GLB, 39, Uart> {}
impl<GLB> HasUartSignal<4> for Pad<GLB, 40, Uart> {}
impl<GLB> HasUartSignal<5> for Pad<GLB, 41, Uart> {}
impl<GLB> HasUartSignal<6> for Pad<GLB, 42, Uart> {}
impl<GLB> HasUartSignal<7> for Pad<GLB, 43, Uart> {}
impl<GLB> HasUartSignal<8> for Pad<GLB, 44, Uart> {}
impl<GLB> HasUartSignal<9> for Pad<GLB, 45, Uart> {}

/// Check if an internal multi-media UART signal is connected to target gpio `Pin`.
pub trait HasMmUartSignal {}

impl<GLB, const N: usize> HasMmUartSignal for Pad<GLB, N, MmUart> {}

/// Valid UART pads.
#[diagnostic::on_unimplemented(
    message = "the I/O pad and signal multiplexer group {Self} is not connected to any UART peripherals on hardware"
)]
pub trait Pads<const U: usize> {
    /// Checks if this pin configuration includes Request-to-Send feature.
    const RTS: bool;
    /// Checks if this pin configuration includes Clear-to-Send feature.
    const CTS: bool;
    /// Checks if this pin configuration includes Transmit feature.
    const TXD: bool;
    /// Checks if this pin configuration includes Receive feature.
    const RXD: bool;
    /// Valid split configuration type for current pads and multiplexers.
    type Split<T>;

    fn split<T>(self, uart: T) -> Self::Split<T>;
}

#[inline]
fn from_pads<T, TX, RX>(uart: T, tx: TX, rx: RX) -> (TransmitHalf<T, TX>, ReceiveHalf<T, RX>) {
    (
        TransmitHalf {
            uart: unsafe { core::ptr::read_volatile(&uart) },
            _pads: tx,
        },
        ReceiveHalf { uart, _pads: rx },
    )
}

impl<A1, GLB2, const I: usize, const U: usize, const N: usize> Pads<U>
    for (Pad<A1, N, Uart>, UartMux<GLB2, I, MuxTxd<U>>)
where
    A1: Deref<Target = glb::v2::RegisterBlock>,
    Pad<A1, N, Uart>: HasUartSignal<I>,
{
    const RTS: bool = false;
    const CTS: bool = false;
    const TXD: bool = true;
    const RXD: bool = false;
    type Split<T> = (
        TransmitHalf<T, (Pad<A1, N, Uart>, UartMux<GLB2, I, MuxTxd<U>>)>,
        ReceiveHalf<T, ()>,
    );
    #[inline]
    fn split<T>(self, uart: T) -> Self::Split<T> {
        from_pads(uart, self, ())
    }
}

impl<
        A1,
        GLB2,
        A3,
        GLB4,
        const I1: usize,
        const I2: usize,
        const U: usize,
        const N1: usize,
        const N2: usize,
    > Pads<U>
    for (
        (Pad<A1, N1, Uart>, UartMux<GLB2, I1, MuxTxd<U>>),
        (Pad<A3, N2, Uart>, UartMux<GLB4, I2, MuxRxd<U>>),
    )
where
    A1: Deref<Target = glb::v2::RegisterBlock>,
    A3: Deref<Target = glb::v2::RegisterBlock>,
    Pad<A1, N1, Uart>: HasUartSignal<I1>,
    Pad<A3, N2, Uart>: HasUartSignal<I2>,
{
    const RTS: bool = false;
    const CTS: bool = false;
    const TXD: bool = true;
    const RXD: bool = true;
    type Split<T> = (
        TransmitHalf<T, (Pad<A1, N1, Uart>, UartMux<GLB2, I1, MuxTxd<U>>)>,
        ReceiveHalf<T, (Pad<A3, N2, Uart>, UartMux<GLB4, I2, MuxRxd<U>>)>,
    );
    #[inline]
    fn split<T>(self, uart: T) -> Self::Split<T> {
        from_pads(uart, self.0, self.1)
    }
}

impl<
        A1,
        GLB2,
        A3,
        GLB4,
        const I1: usize,
        const I2: usize,
        const U: usize,
        const N1: usize,
        const N2: usize,
    > Pads<U>
    for (
        (Pad<A1, N1, Uart>, UartMux<GLB2, I1, MuxTxd<U>>),
        (Pad<A3, N2, Uart>, UartMux<GLB4, I2, MuxCts<U>>),
    )
where
    A1: Deref<Target = glb::v2::RegisterBlock>,
    A3: Deref<Target = glb::v2::RegisterBlock>,
    Pad<A1, N1, Uart>: HasUartSignal<I1>,
    Pad<A3, N2, Uart>: HasUartSignal<I2>,
{
    const RTS: bool = false;
    const CTS: bool = true;
    const TXD: bool = true;
    const RXD: bool = false;
    type Split<T> = TransmitHalf<
        T,
        (
            (Pad<A1, N1, Uart>, UartMux<GLB2, I1, MuxTxd<U>>),
            (Pad<A3, N2, Uart>, UartMux<GLB4, I2, MuxCts<U>>),
        ),
    >;
    #[inline]
    fn split<T>(self, uart: T) -> Self::Split<T> {
        TransmitHalf { uart, _pads: self }
    }
}

impl<
        A1,
        GLB2,
        A3,
        GLB4,
        A5,
        GLB6,
        A7,
        GLB8,
        const I1: usize,
        const I2: usize,
        const I3: usize,
        const I4: usize,
        const U: usize,
        const N1: usize,
        const N2: usize,
        const N3: usize,
        const N4: usize,
    > Pads<U>
    for (
        (Pad<A1, N1, Uart>, UartMux<GLB2, I1, MuxTxd<U>>),
        (Pad<A3, N2, Uart>, UartMux<GLB4, I2, MuxRxd<U>>),
        (Pad<A5, N3, Uart>, UartMux<GLB6, I3, MuxRts<U>>),
        (Pad<A7, N4, Uart>, UartMux<GLB8, I4, MuxCts<U>>),
    )
where
    A1: Deref<Target = glb::v2::RegisterBlock>,
    A3: Deref<Target = glb::v2::RegisterBlock>,
    A5: Deref<Target = glb::v2::RegisterBlock>,
    A7: Deref<Target = glb::v2::RegisterBlock>,
    Pad<A1, N1, Uart>: HasUartSignal<I1>,
    Pad<A3, N2, Uart>: HasUartSignal<I2>,
    Pad<A5, N3, Uart>: HasUartSignal<I3>,
    Pad<A7, N4, Uart>: HasUartSignal<I4>,
{
    const RTS: bool = false;
    const CTS: bool = true;
    const TXD: bool = true;
    const RXD: bool = false;
    type Split<T> = (
        TransmitHalf<
            T,
            (
                (Pad<A1, N1, Uart>, UartMux<GLB2, I1, MuxTxd<U>>),
                (Pad<A7, N4, Uart>, UartMux<GLB8, I4, MuxCts<U>>),
            ),
        >,
        ReceiveHalf<
            T,
            (
                (Pad<A3, N2, Uart>, UartMux<GLB4, I2, MuxRxd<U>>),
                (Pad<A5, N3, Uart>, UartMux<GLB6, I3, MuxRts<U>>),
            ),
        >,
    );
    #[inline]
    fn split<T>(self, uart: T) -> Self::Split<T> {
        from_pads(uart, (self.0, self.3), (self.1, self.2))
    }
}

// TODO: support split for MmUart pads.

impl<A1, const U: usize, const N: usize> Pads<U> for Pad<A1, N, MmUart>
where
    A1: Deref<Target = glb::v2::RegisterBlock>,
    Pad<A1, N, MmUart>: HasMmUartSignal,
{
    const RTS: bool = { N % 4 == 2 };
    const CTS: bool = { N % 4 == 3 };
    const TXD: bool = { N % 4 == 0 };
    const RXD: bool = { N % 4 == 1 };
    type Split<T> = ();
    #[inline]
    fn split<T>(self, uart: T) -> Self::Split<T> {
        let _ = uart;
        ()
    }
}

impl<A1, A2, const U: usize, const N1: usize, const N2: usize> Pads<U>
    for (Pad<A1, N1, MmUart>, Pad<A2, N2, MmUart>)
where
    A1: Deref<Target = glb::v2::RegisterBlock>,
    A2: Deref<Target = glb::v2::RegisterBlock>,
    Pad<A1, N1, MmUart>: HasMmUartSignal,
    Pad<A2, N2, MmUart>: HasMmUartSignal,
{
    const RTS: bool = { N1 % 4 == 2 || N2 % 4 == 2 };
    const CTS: bool = { N1 % 4 == 3 || N2 % 4 == 3 };
    const TXD: bool = { N1 % 4 == 0 || N2 % 4 == 0 };
    const RXD: bool = { N1 % 4 == 1 || N2 % 4 == 1 };
    type Split<T> = ();
    #[inline]
    fn split<T>(self, uart: T) -> Self::Split<T> {
        let _ = uart;
        ()
    }
}

impl<A1, A2, A3, const U: usize, const N1: usize, const N2: usize, const N3: usize> Pads<U>
    for (
        Pad<A1, N1, MmUart>,
        Pad<A2, N2, MmUart>,
        Pad<A3, N3, MmUart>,
    )
where
    A1: Deref<Target = glb::v2::RegisterBlock>,
    A2: Deref<Target = glb::v2::RegisterBlock>,
    A3: Deref<Target = glb::v2::RegisterBlock>,
    Pad<A1, N1, MmUart>: HasMmUartSignal,
    Pad<A2, N2, MmUart>: HasMmUartSignal,
    Pad<A3, N3, MmUart>: HasMmUartSignal,
{
    const RTS: bool = { N1 % 4 == 2 || N2 % 4 == 2 || N3 % 4 == 2 };
    const CTS: bool = { N1 % 4 == 3 || N2 % 4 == 3 || N3 % 4 == 3 };
    const TXD: bool = { N1 % 4 == 0 || N2 % 4 == 0 || N3 % 4 == 0 };
    const RXD: bool = { N1 % 4 == 1 || N2 % 4 == 1 || N3 % 4 == 1 };
    type Split<T> = ();
    #[inline]
    fn split<T>(self, uart: T) -> Self::Split<T> {
        let _ = uart;
        ()
    }
}

impl<
        A1,
        A2,
        A3,
        A4,
        const U: usize,
        const N1: usize,
        const N2: usize,
        const N3: usize,
        const N4: usize,
    > Pads<U>
    for (
        Pad<A1, N1, MmUart>,
        Pad<A2, N2, MmUart>,
        Pad<A3, N3, MmUart>,
        Pad<A4, N4, MmUart>,
    )
where
    A1: Deref<Target = glb::v2::RegisterBlock>,
    A2: Deref<Target = glb::v2::RegisterBlock>,
    A3: Deref<Target = glb::v2::RegisterBlock>,
    A4: Deref<Target = glb::v2::RegisterBlock>,
    Pad<A1, N1, MmUart>: HasMmUartSignal,
    Pad<A2, N2, MmUart>: HasMmUartSignal,
    Pad<A3, N3, MmUart>: HasMmUartSignal,
    Pad<A4, N4, MmUart>: HasMmUartSignal,
{
    const RTS: bool = { N1 % 4 == 2 || N2 % 4 == 2 || N3 % 4 == 2 || N4 % 4 == 2 };
    const CTS: bool = { N1 % 4 == 3 || N2 % 4 == 3 || N3 % 4 == 3 || N4 % 4 == 3 };
    const TXD: bool = { N1 % 4 == 0 || N2 % 4 == 0 || N3 % 4 == 0 || N4 % 4 == 0 };
    const RXD: bool = { N1 % 4 == 1 || N2 % 4 == 1 || N3 % 4 == 1 || N4 % 4 == 1 };
    type Split<T> = ();
    #[inline]
    fn split<T>(self, uart: T) -> Self::Split<T> {
        let _ = uart;
        ()
    }
}

/// Managed serial peripheral.
pub struct Serial<UART, PADS> {
    uart: UART,
    pads: PADS,
}

impl<UART: Deref<Target = RegisterBlock>, PADS> Serial<UART, PADS> {
    /// Creates a polling serial instance, without interrupt or DMA configurations.
    #[inline]
    pub fn freerun<const I: usize>(uart: UART, config: Config, pads: PADS, clocks: &Clocks) -> Self
    where
        PADS: Pads<I>,
    {
        // Calculate transmit interval.
        let uart_clock = clocks.uart_clock::<I>().expect("a valid UART clock source");
        let transmit_interval = uart_clock.0 / config.transmit_baudrate.0;
        let receive_interval = uart_clock.0 / config.receive_baudrate.0;
        if !(1..=65535).contains(&transmit_interval) {
            panic!("Impossible transmit baudrate!");
        }
        if !(1..=65535).contains(&receive_interval) {
            panic!("Impossible receive baudrate!");
        }
        let val = BitPeriod::default()
            .set_transmit_time_interval(transmit_interval as u16)
            .set_receive_time_interval(receive_interval as u16);
        unsafe { uart.bit_period.write(val) };

        // Write the bit-order.
        let val = DataConfig::default().set_bit_order(config.bit_order);
        unsafe { uart.data_config.write(val) };

        // Configure transmit feature.
        let mut val = TransmitConfig::default()
            .enable_freerun()
            .set_parity(config.parity)
            .set_stop_bits(config.stop_bits)
            .set_word_length(config.word_length);
        if PADS::TXD {
            val = val.enable_txd();
        }
        if PADS::CTS {
            val = val.enable_cts();
        }
        unsafe { uart.transmit_config.write(val) };

        // Configure receive feature.
        let mut val = ReceiveConfig::default()
            .set_parity(config.parity)
            .set_word_length(config.word_length);
        if PADS::RXD {
            val = val.enable_rxd();
        }
        unsafe { uart.receive_config.write(val) };

        Self { uart, pads }
    }

    /// Release serial instance and return its peripheral and pads.
    #[inline]
    pub fn free(self) -> (UART, PADS) {
        (self.uart, self.pads)
    }

    /// Split serial instance into transmit and receive halves.
    #[inline]
    pub fn split<const I: usize>(self) -> <PADS as Pads<I>>::Split<UART>
    where
        PADS: Pads<I>,
    {
        self.pads.split(self.uart)
    }
}

#[inline]
fn uart_write(uart: &RegisterBlock, buf: &[u8]) -> Result<usize, Error> {
    while uart.fifo_config_1.read().transmit_available_bytes() == 0 {
        core::hint::spin_loop();
    }
    let len = core::cmp::min(
        uart.fifo_config_1.read().transmit_available_bytes() as usize,
        buf.len(),
    );
    buf.iter()
        .take(len)
        .for_each(|&word| unsafe { uart.fifo_write.write(word) });
    Ok(len)
}

#[inline]
fn uart_write_nb(uart: &RegisterBlock, word: u8) -> nb::Result<(), Error> {
    if uart.fifo_config_1.read().transmit_available_bytes() == 0 {
        return Err(nb::Error::WouldBlock);
    }
    unsafe { uart.fifo_write.write(word) };
    Ok(())
}

#[inline]
fn uart_flush(uart: &RegisterBlock) -> Result<(), Error> {
    // There are maximum 32 bytes in transmit FIFO queue, wait until all bytes are available,
    // meaning that all data in queue has been sent into UART bus.
    while uart.fifo_config_1.read().transmit_available_bytes() != 32 {
        core::hint::spin_loop();
    }
    Ok(())
}

#[inline]
fn uart_flush_nb(uart: &RegisterBlock) -> nb::Result<(), Error> {
    if uart.fifo_config_1.read().transmit_available_bytes() != 32 {
        return Err(nb::Error::WouldBlock);
    }
    Ok(())
}

#[inline]
fn uart_read(uart: &RegisterBlock, buf: &mut [u8]) -> Result<usize, Error> {
    while uart.fifo_config_1.read().receive_available_bytes() == 0 {
        core::hint::spin_loop();
    }
    let len = core::cmp::min(
        uart.fifo_config_1.read().receive_available_bytes() as usize,
        buf.len(),
    );
    buf.iter_mut()
        .take(len)
        .for_each(|slot| *slot = uart.fifo_read.read());
    Ok(len)
}

#[inline]
fn uart_read_nb(uart: &RegisterBlock) -> nb::Result<u8, Error> {
    if uart.fifo_config_1.read().receive_available_bytes() == 0 {
        return Err(nb::Error::WouldBlock);
    }
    let ans = uart.fifo_read.read();
    Ok(ans)
}

/// Transmit half from splitted serial structure.
pub struct TransmitHalf<UART, PADS> {
    uart: UART,
    _pads: PADS,
}

/// Receive half from splitted serial structure.
pub struct ReceiveHalf<UART, PADS> {
    uart: UART,
    _pads: PADS,
}

/// Extend constructor to owned UART register blocks.
pub trait UartExt<PADS>: Sized {
    /// Creates a polling serial instance, without interrupt or DMA configurations.
    fn freerun<const I: usize>(
        self,
        config: Config,
        pads: PADS,
        clocks: &Clocks,
    ) -> Serial<Self, PADS>
    where
        PADS: Pads<I>;
}

impl<UART: Deref<Target = RegisterBlock>, PADS> UartExt<PADS> for UART {
    #[inline]
    fn freerun<const I: usize>(
        self,
        config: Config,
        pads: PADS,
        clocks: &Clocks,
    ) -> Serial<Self, PADS>
    where
        PADS: Pads<I>,
    {
        Serial::freerun(self, config, pads, clocks)
    }
}

impl embedded_io::Error for Error {
    #[inline(always)]
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl embedded_hal_nb::serial::Error for Error {
    #[inline(always)]
    fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
        match self {
            Error::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
            Error::Noise => embedded_hal_nb::serial::ErrorKind::Noise,
            Error::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
            Error::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
        }
    }
}

impl<UART, PADS> embedded_io::ErrorType for Serial<UART, PADS> {
    type Error = Error;
}

impl<UART, PADS> embedded_hal_nb::serial::ErrorType for Serial<UART, PADS> {
    type Error = Error;
}

impl<UART, PADS> embedded_io::ErrorType for TransmitHalf<UART, PADS> {
    type Error = Error;
}

impl<UART, PADS> embedded_hal_nb::serial::ErrorType for TransmitHalf<UART, PADS> {
    type Error = Error;
}

impl<UART, PADS> embedded_io::ErrorType for ReceiveHalf<UART, PADS> {
    type Error = Error;
}

impl<UART, PADS> embedded_hal_nb::serial::ErrorType for ReceiveHalf<UART, PADS> {
    type Error = Error;
}

impl<UART: Deref<Target = RegisterBlock>, PADS> embedded_io::Write for Serial<UART, PADS> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        uart_write(&self.uart, buf)
    }
    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        uart_flush(&self.uart)
    }
}

impl<UART: Deref<Target = RegisterBlock>, PADS> embedded_hal_nb::serial::Write
    for Serial<UART, PADS>
{
    #[inline]
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        uart_write_nb(&self.uart, word)
    }
    #[inline]
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        uart_flush_nb(&self.uart)
    }
}

impl<UART: Deref<Target = RegisterBlock>, PADS> embedded_io::Read for Serial<UART, PADS> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        uart_read(&self.uart, buf)
    }
}

impl<UART: Deref<Target = RegisterBlock>, PADS> embedded_hal_nb::serial::Read
    for Serial<UART, PADS>
{
    #[inline]
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        uart_read_nb(&self.uart)
    }
}

impl<UART: Deref<Target = RegisterBlock>, PADS> embedded_io::Write for TransmitHalf<UART, PADS> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        uart_write(&self.uart, buf)
    }
    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        uart_flush(&self.uart)
    }
}

impl<UART: Deref<Target = RegisterBlock>, PADS> embedded_hal_nb::serial::Write
    for TransmitHalf<UART, PADS>
{
    #[inline]
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        uart_write_nb(&self.uart, word)
    }
    #[inline]
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        uart_flush_nb(&self.uart)
    }
}

impl<UART: Deref<Target = RegisterBlock>, PADS> embedded_io::Read for ReceiveHalf<UART, PADS> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        uart_read(&self.uart, buf)
    }
}

impl<UART: Deref<Target = RegisterBlock>, PADS> embedded_hal_nb::serial::Read
    for ReceiveHalf<UART, PADS>
{
    #[inline]
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        uart_read_nb(&self.uart)
    }
}

/// Serial configuration.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Config {
    /// Baudrate on the transmit half.
    pub transmit_baudrate: Baud,
    /// Baudrate on the receive half.
    pub receive_baudrate: Baud,
    /// Data bit order.
    pub bit_order: BitOrder,
    /// Parity settings.
    pub parity: Parity,
    /// Serial stop bits.
    pub stop_bits: StopBits,
    /// Data word length.
    pub word_length: WordLength,
}

impl Config {
    /// Set baudrate for both the transmit and receive halves.
    ///
    /// This function sets the same baudrate for the transmit and receive halves.
    #[inline]
    pub const fn set_baudrate(self, baudrate: Baud) -> Self {
        Self {
            transmit_baudrate: baudrate,
            receive_baudrate: baudrate,
            ..self
        }
    }
}

impl Default for Config {
    /// Serial configuration defaults to 8-bit word, no parity check, 1 stop bit, LSB first.
    #[inline]
    fn default() -> Self {
        Config {
            transmit_baudrate: 115_200.Bd(),
            receive_baudrate: 115_200.Bd(),
            bit_order: BitOrder::LsbFirst,
            parity: Parity::None,
            stop_bits: StopBits::One,
            word_length: WordLength::Eight,
        }
    }
}

/// Order of the bits transmitted and received on the wire.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BitOrder {
    /// Each byte is sent out LSB-first.
    LsbFirst,
    /// Each byte is sent out MSB-first.
    MsbFirst,
}

/// Parity check.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Parity {
    /// No parity check.
    None,
    /// Even parity bit.
    Even,
    /// Odd parity bit.
    Odd,
}

/// Stop bits.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StopBits {
    /// 0.5 stop bits.
    ZeroPointFive,
    /// 1 stop bit.
    One,
    /// 1.5 stop bits.
    OnePointFive,
    /// 2 stop bits.
    Two,
}

/// Word length.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WordLength {
    /// Five bits per word.
    Five,
    /// Six bits per word.
    Six,
    /// Seven bits per word.
    Seven,
    /// Eight bits per word.
    Eight,
}

/// Serial error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Framing error.
    Framing,
    /// Noise error.
    Noise,
    /// RX buffer overrun.
    Overrun,
    /// Parity check error.
    Parity,
}

#[cfg(test)]
mod tests {
    use crate::uart::{StopBits, WordLength};

    use super::{BitPeriod, Parity, ReceiveConfig, RegisterBlock, TransmitConfig};
    use memoffset::offset_of;

    #[test]
    fn struct_register_block_offset() {
        assert_eq!(offset_of!(RegisterBlock, transmit_config), 0x0);
        assert_eq!(offset_of!(RegisterBlock, receive_config), 0x4);
        assert_eq!(offset_of!(RegisterBlock, bit_period), 0x08);
        assert_eq!(offset_of!(RegisterBlock, data_config), 0x0c);
        assert_eq!(offset_of!(RegisterBlock, interrupt_state), 0x20);
        assert_eq!(offset_of!(RegisterBlock, interrupt_mask), 0x24);
        assert_eq!(offset_of!(RegisterBlock, interrupt_clear), 0x28);
        assert_eq!(offset_of!(RegisterBlock, interrupt_enable), 0x2c);
        assert_eq!(offset_of!(RegisterBlock, bus_state), 0x30);
        assert_eq!(offset_of!(RegisterBlock, fifo_config_0), 0x80);
        assert_eq!(offset_of!(RegisterBlock, fifo_config_1), 0x84);
        assert_eq!(offset_of!(RegisterBlock, fifo_write), 0x88);
        assert_eq!(offset_of!(RegisterBlock, fifo_read), 0x8c);
    }

    #[test]
    fn struct_transmit_config_functions() {
        let mut val: TransmitConfig = TransmitConfig(0x0);

        val = val.enable_txd();
        assert_eq!(val.0, 0x00000001);
        assert!(val.is_txd_enabled());
        val = val.disable_txd();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_txd_enabled());

        val = val.enable_cts();
        assert_eq!(val.0, 0x00000002);
        assert!(val.is_cts_enabled());
        val = val.disable_cts();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_cts_enabled());

        val = val.enable_freerun();
        assert_eq!(val.0, 0x00000004);
        assert!(val.is_freerun_enabled());
        val = val.disable_freerun();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_freerun_enabled());

        val = val.enable_lin_transmit();
        assert_eq!(val.0, 0x00000008);
        assert!(val.is_lin_transmit_enabled());
        val = val.disable_lin_transmit();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_lin_transmit_enabled());

        val = val.set_parity(Parity::Even);
        assert_eq!(val.0, 0x00000010);
        assert_eq!(val.parity(), Parity::Even);
        val = val.set_parity(Parity::Odd);
        assert_eq!(val.0, 0x00000030);
        assert_eq!(val.parity(), Parity::Odd);
        val = val.set_parity(Parity::None);
        assert_eq!(val.0 & 0x00000010, 0x00000000);
        assert_eq!(val.parity(), Parity::None);

        val = TransmitConfig(0x0);

        val = val.enable_ir_transmit();
        assert_eq!(val.0, 0x00000040);
        assert!(val.is_ir_transmit_enabled());
        val = val.disable_ir_transmit();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_ir_transmit_enabled());

        val = val.enable_ir_inverse();
        assert_eq!(val.0, 0x00000080);
        assert!(val.is_ir_inverse_enabled());
        val = val.disable_ir_inverse();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_ir_inverse_enabled());

        val = val.set_word_length(WordLength::Five);
        assert_eq!(val.0, 0x00000400);
        assert_eq!(val.word_length(), WordLength::Five);
        val = val.set_word_length(WordLength::Six);
        assert_eq!(val.0, 0x00000500);
        assert_eq!(val.word_length(), WordLength::Six);
        val = val.set_word_length(WordLength::Seven);
        assert_eq!(val.0, 0x00000600);
        assert_eq!(val.word_length(), WordLength::Seven);
        val = val.set_word_length(WordLength::Eight);
        assert_eq!(val.0, 0x00000700);
        assert_eq!(val.word_length(), WordLength::Eight);

        val = TransmitConfig(0x0);

        val = val.set_stop_bits(StopBits::Two);
        assert_eq!(val.0, 0x00001800);
        assert_eq!(val.stop_bits(), StopBits::Two);
        val = val.set_stop_bits(StopBits::OnePointFive);
        assert_eq!(val.0, 0x00001000);
        assert_eq!(val.stop_bits(), StopBits::OnePointFive);
        val = val.set_stop_bits(StopBits::One);
        assert_eq!(val.0, 0x00000800);
        assert_eq!(val.stop_bits(), StopBits::One);
        val = val.set_stop_bits(StopBits::ZeroPointFive);
        assert_eq!(val.0, 0x00000000);
        assert_eq!(val.stop_bits(), StopBits::ZeroPointFive);

        for num in 0..=7 {
            val = val.set_lin_break_bits(num);
            assert_eq!(val.0, (num as u32) << 13);
            assert_eq!(val.lin_break_bits(), num);
        }

        val = TransmitConfig(0x0);

        for length in [0x0000, 0x1234, 0xabcd, 0xffff] {
            val = val.set_transfer_length(length);
            assert_eq!(val.0, (length as u32) << 16);
            assert_eq!(val.transfer_length(), length);
        }

        let default = TransmitConfig::default();
        assert_eq!(default.transfer_length(), 0);
        assert_eq!(default.lin_break_bits(), 4);
        assert_eq!(default.stop_bits(), StopBits::One);
        assert_eq!(default.word_length(), WordLength::Eight);
        assert!(!default.is_ir_inverse_enabled());
        assert!(!default.is_ir_transmit_enabled());
        assert_eq!(default.parity(), Parity::None);
        assert!(!default.is_lin_transmit_enabled());
        assert!(!default.is_freerun_enabled());
        assert!(!default.is_cts_enabled());
        assert!(!default.is_txd_enabled());
    }

    #[test]
    fn struct_bit_period_functions() {
        let mut val: BitPeriod = BitPeriod(0x0);

        for trans in [0x0000, 0x1037, 0xabcd, 0xffff] {
            val = val.set_transmit_time_interval(trans);
            assert_eq!(val.0, trans as u32);
            assert_eq!(val.transmit_time_interval(), trans);
        }

        val = BitPeriod(0x0);

        for recv in [0x0000, 0x1037, 0xabcd, 0xffff] {
            val = val.set_receive_time_interval(recv);
            assert_eq!(val.0, (recv as u32) << 16);
            assert_eq!(val.receive_time_interval(), recv);
        }

        // TODO: use getter functions to check default value for BitPeriod
    }

    #[test]
    fn struct_receive_config_functions() {
        let mut val: ReceiveConfig = ReceiveConfig(0x0);

        val = val.enable_rxd();
        assert_eq!(val.0, 0x00000001);
        assert!(val.is_rxd_enabled());
        val = val.disable_rxd();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_rxd_enabled());

        val = val.enable_auto_baudrate();
        assert_eq!(val.0, 0x00000002);
        assert!(val.is_auto_baudrate_enabled());
        val = val.disable_auto_baudrate();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_auto_baudrate_enabled());

        val = val.enable_lin_receive();
        assert_eq!(val.0, 0x00000008);
        assert!(val.is_lin_receive_enabled());
        val = val.disable_lin_receive();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_lin_receive_enabled());

        val = val.set_parity(Parity::Even);
        assert_eq!(val.0, 0x00000010);
        assert_eq!(val.parity(), Parity::Even);
        val = val.set_parity(Parity::Odd);
        assert_eq!(val.0, 0x00000030);
        assert_eq!(val.parity(), Parity::Odd);
        val = val.set_parity(Parity::None);
        assert_eq!(val.0 & 0x00000010, 0x00000000);
        assert_eq!(val.parity(), Parity::None);

        val = ReceiveConfig(0x0);

        val = val.enable_ir_receive();
        assert_eq!(val.0, 0x00000040);
        assert!(val.is_ir_receive_enabled());
        val = val.disable_ir_receive();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_ir_receive_enabled());

        val = val.enable_ir_inverse();
        assert_eq!(val.0, 0x00000080);
        assert!(val.is_ir_inverse_enabled());
        val = val.disable_ir_inverse();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_ir_inverse_enabled());

        val = val.set_word_length(WordLength::Five);
        assert_eq!(val.0, 0x00000400);
        assert_eq!(val.word_length(), WordLength::Five);
        val = val.set_word_length(WordLength::Six);
        assert_eq!(val.0, 0x00000500);
        assert_eq!(val.word_length(), WordLength::Six);
        val = val.set_word_length(WordLength::Seven);
        assert_eq!(val.0, 0x00000600);
        assert_eq!(val.word_length(), WordLength::Seven);
        val = val.set_word_length(WordLength::Eight);
        assert_eq!(val.0, 0x00000700);
        assert_eq!(val.word_length(), WordLength::Eight);

        val = ReceiveConfig(0x0);

        val = val.enable_deglitch();
        assert_eq!(val.0, 0x00000800);
        assert!(val.is_deglitch_enabled());
        val = val.disable_deglitch();
        assert_eq!(val.0, 0x00000000);
        assert!(!val.is_deglitch_enabled());

        for num in 0..=7 {
            val = val.set_deglitch_cycles(num);
            assert_eq!(val.0, (num as u32) << 12);
            assert_eq!(val.deglitch_cycles(), num);
        }

        val = ReceiveConfig(0x0);

        for length in [0x0000, 0x1234, 0xabcd, 0xffff] {
            val = val.set_transfer_length(length);
            assert_eq!(val.0, (length as u32) << 16);
            assert_eq!(val.transfer_length(), length);
        }
    }

    // TODO: use getter functions to check default value for ReceiveConfig
}
