#![no_std]
#![no_main]

use bouffalo_hal::{prelude::*, spi::Spi, uart::Config};
use bouffalo_rt::{entry, Clocks, Peripherals};
use embedded_hal::spi::MODE_3;
use embedded_sdmmc::{SdCard, VolumeManager};
use embedded_time::rate::*;
use panic_halt as _;

struct MyTimeSource {}

impl embedded_sdmmc::TimeSource for MyTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        // TODO
        embedded_sdmmc::Timestamp::from_calendar(2023, 1, 1, 0, 0, 0).unwrap()
    }
}

#[entry]
fn main(p: Peripherals, c: Clocks) -> ! {
    let tx = p.gpio.io14.into_uart();
    let rx = p.gpio.io15.into_uart();
    let sig2 = p.uart_muxes.sig2.into_transmit::<0>();
    let sig3 = p.uart_muxes.sig3.into_receive::<0>();
    let pads = ((tx, sig2), (rx, sig3));

    let config = Config::default().set_baudrate(2000000.Bd());
    let mut serial = p.uart0.freerun(config, pads, &c).unwrap();
    writeln!(serial, "Hello world!").ok();

    let mut led = p.gpio.io8.into_floating_output();
    let mut led_state = PinState::High;

    let spi_clk = p.gpio.io3.into_spi::<1>();
    let spi_mosi = p.gpio.io1.into_spi::<1>();
    let spi_miso = p.gpio.io2.into_spi::<1>();
    let spi_cs = p.gpio.io0.into_spi::<1>();
    let spi_sd = Spi::new(
        p.spi1,
        (spi_clk, spi_mosi, spi_miso, spi_cs),
        MODE_3,
        &p.glb,
    );

    let delay = riscv::delay::McycleDelay::new(40_000_000);
    let sdcard = SdCard::new(spi_sd, delay);
    while sdcard.get_card_type().is_none() {
        core::hint::spin_loop();
    }

    let time_source = MyTimeSource {};
    let mut volume_mgr = VolumeManager::new(sdcard, time_source);
    let volume0 = volume_mgr
        .open_raw_volume(embedded_sdmmc::VolumeIdx(0))
        .unwrap();
    let root_dir = volume_mgr.open_root_dir(volume0).unwrap();

    volume_mgr
        .iterate_dir(root_dir, |entry| {
            writeln!(serial, "Entry: {:?}", entry).ok();
        })
        .unwrap();

    volume_mgr.close_dir(root_dir).unwrap();

    loop {
        led.set_state(led_state).ok();
        led_state = !led_state;
        riscv::asm::delay(100_000);
    }
}
