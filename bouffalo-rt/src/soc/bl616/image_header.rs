use crate::{HalBasicConfig, HalFlashConfig, HalPatchCfg};

/// Clock configuration at boot-time.
#[cfg(any(doc, feature = "bl616"))]
#[unsafe(link_section = ".head.clock")]
pub static CLOCK_CONFIG: HalPllConfig = HalPllConfig::new(HalSysClkConfig {
    xtal_type: 0x07,
    mcu_clk: 0x05,
    mcu_clk_div: 0x00,
    mcu_bclk_div: 0x00,

    mcu_pbclk_div: 0x03,
    emi_clk: 0x02,

    emi_clk_div: 0x01,
    flash_clk_type: 0x01,
    flash_clk_div: 0x00,
    wifipll_pu: 0x01,

    aupll_pu: 0x01,
    rsvd0: 0x00,
});

/// Miscellaneous image flags.
#[cfg(any(doc, feature = "bl616"))]
#[unsafe(link_section = ".head.base.flag")]
pub static BASIC_CONFIG_FLAGS: u32 = 0x654c0100;

/// Processor core configuration.
#[cfg(any(doc, feature = "bl616"))]
#[unsafe(link_section = ".head.cpu")]
pub static CPU_CONFIG: [HalCpuCfg; 1] = [HalCpuCfg {
    config_enable: 1,
    halt_cpu: 0,
    cache_flags: 0,
    _rsvd: 0,
    image_address_offset: 0,
    _rsvd1: 0xA0000000,
    msp_val: 0,
}];

/// Code patches on flash reading.
#[cfg(any(doc, feature = "bl616"))]
#[unsafe(link_section = ".head.patch.on-read")]
pub static PATCH_ON_READ: [HalPatchCfg; 3] = [
    HalPatchCfg {
        addr: 0x20000548,
        value: 0x1000000,
    },
    HalPatchCfg { addr: 0, value: 0 },
    HalPatchCfg { addr: 0, value: 0 },
];

/// Code patches on jump and run stage.
#[cfg(any(doc, feature = "bl616"))]
#[unsafe(link_section = ".head.patch.on-jump")]
pub static PATCH_ON_JUMP: [HalPatchCfg; 3] = [
    HalPatchCfg { addr: 0, value: 0 },
    HalPatchCfg { addr: 0, value: 0 },
    HalPatchCfg { addr: 0, value: 0 },
];

/// Full ROM bootloading header.
#[repr(C)]
pub struct HalBootheader {
    magic: u32,
    revision: u32,
    flash_cfg: HalFlashConfig,
    clk_cfg: HalPllConfig,
    basic_cfg: HalBasicConfig,
    cpu_cfg: HalCpuCfg,
    /// Address of partition table 0.
    boot2_pt_table_0: u32,
    /// Address of partition table 1.
    boot2_pt_table_1: u32,
    /// Address of flashcfg table list.
    flash_cfg_table_addr: u32,
    /// Flashcfg table list len.
    flash_cfg_table_len: u32,
    /// Do patch when read flash.
    patch_on_read: [HalPatchCfg; 3],
    /// Do patch when jump.
    patch_on_jump: [HalPatchCfg; 3],
    _reserved: [u32; 1],
    crc32: u32,
}

/// Hardware system clock configuration.
#[repr(C)]
pub struct HalSysClkConfig {
    xtal_type: u8,
    mcu_clk: u8,
    mcu_clk_div: u8,
    mcu_bclk_div: u8,

    mcu_pbclk_div: u8,
    emi_clk: u8,
    emi_clk_div: u8,
    flash_clk_type: u8,
    flash_clk_div: u8,
    wifipll_pu: u8,

    aupll_pu: u8,
    rsvd0: u8,
}

impl HalSysClkConfig {
    #[inline]
    pub const fn crc32(&self) -> u32 {
        let mut buf = [0u8; 12];

        buf[0] = self.xtal_type;
        buf[1] = self.mcu_clk;
        buf[2] = self.mcu_clk_div;
        buf[3] = self.mcu_bclk_div;

        buf[4] = self.mcu_pbclk_div;
        buf[5] = self.emi_clk;
        buf[6] = self.emi_clk_div;
        buf[7] = self.flash_clk_type;
        buf[8] = self.flash_clk_div;
        buf[9] = self.wifipll_pu;

        buf[10] = self.aupll_pu;
        buf[11] = self.rsvd0;

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

/// Processor core configuration in ROM header.
#[repr(C)]
pub struct HalCpuCfg {
    /// Config this cpu.
    config_enable: u8,
    /// Halt this cpu.
    halt_cpu: u8,
    /// Cache setting.
    cache_flags: u8,
    _rsvd: u8,
    /// Image address on flash.
    image_address_offset: u32,
    _rsvd1: u32,
    /// Msp value.
    msp_val: u32,
}
#[cfg(test)]
mod tests {
    use super::{HalBootheader, HalPllConfig, HalSysClkConfig};
    use core::mem::offset_of;

    #[test]
    fn struct_lengths() {
        use core::mem::size_of;
        assert_eq!(size_of::<HalPllConfig>(), 0x14);
        assert_eq!(size_of::<HalBootheader>(), 0x100);
    }

    #[test]
    fn struct_hal_bootheader_offset() {
        assert_eq!(offset_of!(HalBootheader, magic), 0x00);
        assert_eq!(offset_of!(HalBootheader, revision), 0x04);
        assert_eq!(offset_of!(HalBootheader, flash_cfg), 0x08);
        assert_eq!(offset_of!(HalBootheader, clk_cfg), 0x64);
        assert_eq!(offset_of!(HalBootheader, basic_cfg), 0x78);
        assert_eq!(offset_of!(HalBootheader, cpu_cfg), 0xa8);
        assert_eq!(offset_of!(HalBootheader, boot2_pt_table_0), 0xb8);
        assert_eq!(offset_of!(HalBootheader, boot2_pt_table_1), 0xbc);
        assert_eq!(offset_of!(HalBootheader, flash_cfg_table_addr), 0xc0);
        assert_eq!(offset_of!(HalBootheader, flash_cfg_table_len), 0xc4);
        assert_eq!(offset_of!(HalBootheader, patch_on_read), 0xc8);
        assert_eq!(offset_of!(HalBootheader, patch_on_jump), 0xe0);
        assert_eq!(offset_of!(HalBootheader, crc32), 0xfc);
    }

    #[test]
    fn struct_hal_sys_clk_config_offset() {
        assert_eq!(offset_of!(HalSysClkConfig, xtal_type), 0x00);
        assert_eq!(offset_of!(HalSysClkConfig, mcu_clk), 0x01);
        assert_eq!(offset_of!(HalSysClkConfig, mcu_clk_div), 0x02);
        assert_eq!(offset_of!(HalSysClkConfig, mcu_bclk_div), 0x03);

        assert_eq!(offset_of!(HalSysClkConfig, mcu_pbclk_div), 0x04);
        assert_eq!(offset_of!(HalSysClkConfig, emi_clk), 0x05);
        assert_eq!(offset_of!(HalSysClkConfig, emi_clk_div), 0x06);
        assert_eq!(offset_of!(HalSysClkConfig, flash_clk_type), 0x07);
        assert_eq!(offset_of!(HalSysClkConfig, flash_clk_div), 0x08);
        assert_eq!(offset_of!(HalSysClkConfig, wifipll_pu), 0x09);

        assert_eq!(offset_of!(HalSysClkConfig, aupll_pu), 0x0a);
        assert_eq!(offset_of!(HalSysClkConfig, rsvd0), 0x0b);
    }

    #[test]
    fn struct_hal_pll_config_offset() {
        assert_eq!(offset_of!(HalPllConfig, magic), 0x00);
        assert_eq!(offset_of!(HalPllConfig, cfg), 0x04);
        assert_eq!(offset_of!(HalPllConfig, crc32), 0x10);
    }

    #[test]
    fn magic_crc32_hal_pll_config() {
        let test_sys_clk_config = HalSysClkConfig {
            xtal_type: 7,
            mcu_clk: 5,
            mcu_clk_div: 0,
            mcu_bclk_div: 0,
            mcu_pbclk_div: 3,
            emi_clk: 2,
            emi_clk_div: 1,
            flash_clk_type: 1,
            flash_clk_div: 0,
            wifipll_pu: 1,
            aupll_pu: 1,
            rsvd0: 0,
        };
        let test_config = HalPllConfig::new(test_sys_clk_config);
        assert_eq!(test_config.magic, 0x47464350);
        assert_eq!(test_config.crc32, 0x89EF340B);
    }
}
