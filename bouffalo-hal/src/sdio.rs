//! Secure Digital Input/Output peripheral.

mod config;
mod dma_sdh;
mod nodma_sdh;
mod ops;
mod pad;
mod register;
pub use config::*;
pub use dma_sdh::*;
pub use pad::*;
pub use register::*;

/// SDH peripheral type without system dma.
pub type NonSysDmaSdh<SDH, PADS> = nodma_sdh::Sdh<SDH, PADS>;

use crate::glb;

/// Extend constructor to SDH ownership structures.
pub trait SdhExt<'a> {
    /// Create a new instance of the SDH peripheral with a DMA channel.
    fn with_dma<PADS, CH>(
        self,
        pads: PADS,
        dma_channel: CH,
        config: Config,
        glb: &glb::v2::RegisterBlock,
    ) -> dma_sdh::Sdh<'a, PADS, CH>
    where
        PADS: pad::Pads,
        CH: core::ops::Deref<Target = crate::dma::UntypedChannel<'a>>;
}
