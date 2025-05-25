#![no_std]
#![no_main]

use bouffalo_hal::prelude::*;
use bouffalo_rt::{Clocks, Peripherals, entry};
use panic_halt as _;

#[entry]
fn main(p: Peripherals, _c: Clocks) -> ! {
    let rx = p.gpio.io0.into_uart();
    
    todo!()
}
