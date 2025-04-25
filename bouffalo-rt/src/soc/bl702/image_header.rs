use crate::HalFlashConfig;

#[cfg(any(doc, feature = "bl702"))]
#[unsafe(link_section = ".head.clock")]
#[used]
pub static CLOCK_CONFIG: HalPllConfig = HalPllConfig::new(HalSysClkConfig {
    xtal_type: 0x1,
    pll_clk: 0x4,
    hclk_div: 0,
    bclk_div: 0x1,

    flash_clk_type: 0x1,
    flash_clk_div: 0,
    _reserved: [0, 0],
});

/// Miscellaneous image flags.
#[cfg(any(doc, feature = "bl702"))]
#[unsafe(link_section = ".head.base.flag")]
pub static BASIC_CONFIG_FLAGS: u32 = 0x00000310;

/// Full ROM bootloading header.
#[repr(C)]
pub struct HalBootheader {
    magic: u32,
    revision: u32,
    flash_cfg: HalFlashConfig,
    clk_cfg: HalPllConfig,
    basic_cfg: HalBasicConfig,
    _reserved: [u32; 2],
    crc32: u32,
}

/// Hardware system clock configuration.
#[repr(C)]
pub struct HalSysClkConfig {
    xtal_type: u8,
    pll_clk: u8,
    hclk_div: u8,
    bclk_div: u8,

    flash_clk_type: u8,
    flash_clk_div: u8,
    _reserved: [u8; 2],
}

impl HalSysClkConfig {
    #[inline]
    pub const fn crc32(&self) -> u32 {
        let mut buf = [0u8; 8];

        buf[0] = self.xtal_type;
        buf[1] = self.pll_clk;
        buf[2] = self.hclk_div;
        buf[3] = self.bclk_div;

        buf[4] = self.flash_clk_type;
        buf[5] = self.flash_clk_div;
        buf[6] = self._reserved[0];
        buf[7] = self._reserved[1];

        crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC).checksum(&buf)
    }
}

/// Clock configuration in ROM header.
#[repr(C)]
pub struct HalPllConfig {
    magic: u32,
    cfg: HalSysClkConfig,
    crc32: u32,
}

impl HalPllConfig {
    /// Create this structure with magic number and CRC32 filled in compile time.
    #[inline]
    pub const fn new(cfg: HalSysClkConfig) -> Self {
        let crc32 = cfg.crc32();
        HalPllConfig {
            magic: 0x47464350,
            cfg,
            crc32,
        }
    }
}

#[repr(C)]
struct HalBasicConfig {
    /// Flags 4bytes
    ///
    /// 2bits  for sign
    /// 2bits  for encrypt
    /// 2bits  for key slot
    /// 2bits  for rsvd
    /// 1bit   for no segment info
    /// 1bit   for cache enable
    /// 1bit   for notload in bootrom
    /// 1bit   for aes region lock
    /// 4bits  for cache way disable
    /// 1bit   for ignore crc
    /// 1bit   for hash ignore
    /// 1bit   for halt cpu1
    /// 13bits for rsvd
    flag: u32,
    /// Image length or segment count.
    img_len_cnt: u32,
    /// Entry point of the image.
    boot_entry: u32,
    /// Ram address or flash offset of the image.
    img_start: u32,
    /// Hash of the image.
    hash: [u32; 8],
}

#[cfg(test)]
mod tests {
    use super::{HalBasicConfig, HalBootheader, HalPllConfig, HalSysClkConfig};
    use core::mem::offset_of;

    #[test]
    fn struct_lengths() {
        use core::mem::size_of;
        assert_eq!(size_of::<HalPllConfig>(), 0x10);
        assert_eq!(size_of::<HalBootheader>(), 0xB0);
        assert_eq!(size_of::<HalBasicConfig>(), 0x30);
    }

    #[test]
    fn struct_hal_bootheader_offset() {
        assert_eq!(offset_of!(HalBootheader, magic), 0x00);
        assert_eq!(offset_of!(HalBootheader, revision), 0x04);
        assert_eq!(offset_of!(HalBootheader, flash_cfg), 0x08);
        assert_eq!(offset_of!(HalBootheader, clk_cfg), 0x64);
        assert_eq!(offset_of!(HalBootheader, basic_cfg), 0x74);
        assert_eq!(offset_of!(HalBootheader, crc32), 0xac);
    }

    #[test]
    fn struct_hal_sys_clk_config_offset() {
        assert_eq!(offset_of!(HalSysClkConfig, xtal_type), 0x00);
        assert_eq!(offset_of!(HalSysClkConfig, pll_clk), 0x01);
        assert_eq!(offset_of!(HalSysClkConfig, hclk_div), 0x02);
        assert_eq!(offset_of!(HalSysClkConfig, bclk_div), 0x03);
        assert_eq!(offset_of!(HalSysClkConfig, flash_clk_type), 0x04);
        assert_eq!(offset_of!(HalSysClkConfig, flash_clk_div), 0x05);
        assert_eq!(offset_of!(HalSysClkConfig, _reserved), 0x06);
    }

    #[test]
    fn struct_hal_pll_config_offset() {
        assert_eq!(offset_of!(HalPllConfig, magic), 0x00);
        assert_eq!(offset_of!(HalPllConfig, cfg), 0x04);
        assert_eq!(offset_of!(HalPllConfig, crc32), 0x0c);
    }

    #[test]
    fn magic_crc32_hal_pll_config() {
        let test_sys_clk_config = HalSysClkConfig {
            xtal_type: 0x1,
            pll_clk: 0x4,
            hclk_div: 0,
            bclk_div: 0x1,
            flash_clk_type: 0x1,
            flash_clk_div: 0,
            _reserved: [0, 0],
        };
        let test_config = HalPllConfig::new(test_sys_clk_config);
        assert_eq!(test_config.magic, 0x47464350);
        assert_eq!(test_config.crc32, 0xD81BB531);
    }
}
