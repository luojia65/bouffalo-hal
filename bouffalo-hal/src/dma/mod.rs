//! Direct Memory Access peripheral.

mod channel;
mod config;
mod register;

pub use channel::*;
pub use config::*;
pub use register::*;

use crate::glb;

/// DMA peripheral data register address definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DmaAddr {
    Uart0Tx = 0x2000A000 + 0x88,
    Uart0Rx = 0x2000A000 + 0x8C,
    Uart1Tx = 0x2000A100 + 0x88,
    Uart1Rx = 0x2000A100 + 0x8C,
    Uart2Tx = 0x2000AA00 + 0x88,
    Uart2Rx = 0x2000AA00 + 0x8C,
    Uart3Tx = 0x30002000 + 0x88,
    Uart3Rx = 0x30002000 + 0x8C,
    I2c0Tx = 0x2000A300 + 0x88,
    I2c0Rx = 0x2000A300 + 0x8C,
    I2c1Tx = 0x2000A900 + 0x88,
    I2c1Rx = 0x2000A900 + 0x8C,
    I2c2Tx = 0x30003000 + 0x88,
    I2c2Rx = 0x30003000 + 0x8C,
    I2c3Tx = 0x30004000 + 0x88,
    I2c3Rx = 0x30004000 + 0x8C,
    Spi0Tx = 0x2000A200 + 0x88,
    Spi0Rx = 0x2000A200 + 0x8C,
    Spi1Tx = 0x30008000 + 0x88,
    Spi1Rx = 0x30008000 + 0x8C,
    I2sTx = 0x2000AB00 + 0x88,
    I2sRx = 0x2000AB00 + 0x8C,
    AdcRx = 0x20002000 + 0x04,
    DacTx = 0x20002000 + 0x48,
    IrTx = 0x2000A600 + 0x88,
    WoTx = 0x20000000 + 0xB04,
}

/// Extend constructor to DMA ownership structures.
pub trait DmaExt {
    type Group<'a>
    where
        Self: 'a;
    fn split(&self, glb: &glb::v2::RegisterBlock) -> Self::Group<'_>;
}
