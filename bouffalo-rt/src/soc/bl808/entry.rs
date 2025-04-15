#[cfg(all(feature = "bl808-mcu", target_arch = "riscv32"))]
#[naked]
#[unsafe(link_section = ".text.entry")]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn start() -> ! {
    unsafe {
        use crate::arch::rvi::Stack;
        const LEN_STACK_MCU: usize = 1 * 1024;
        #[unsafe(link_section = ".bss.uninit")]
        static mut STACK: Stack<LEN_STACK_MCU> = Stack([0; LEN_STACK_MCU]);
        core::arch::naked_asm!(
            "   la      sp, {stack}
            li      t0, {hart_stack_size}
            add     sp, sp, t0",
            "   la      t1, sbss
            la      t2, ebss
        1:  bgeu    t1, t2, 1f
            sw      zero, 0(t1)
            addi    t1, t1, 4
            j       1b
        1:",
            "   la      t3, sidata
            la      t4, sdata
            la      t5, edata
        1:  bgeu    t4, t5, 1f
            lw      t6, 0(t3)
            sw      t6, 0(t4)
            addi    t3, t3, 4
            addi    t4, t4, 4
            j       1b
        1:",
            "   la      t0, {trap_entry}
            ori     t0, t0, {trap_mode}
            csrw    mtvec, t0",
            "   li      t1, {stack_protect_pmp_address_begin}
            csrw    pmpaddr0, t1
            li      t1, {stack_protect_pmp_address_end}
            csrw    pmpaddr1, t1
            li      t2, {stack_protect_pmp_flags}
            csrw    pmpcfg0, t2",
            "   call  {main}",
            stack = sym STACK,
            hart_stack_size = const LEN_STACK_MCU,
            trap_entry = sym trap_vectored,
            trap_mode = const 1, // RISC-V standard vectored trap
            // Set PMP entry to block U/S-mode stack access (TOR, no R/W/X permissions)
            stack_protect_pmp_address_begin = const {0x62030000 >> 2},
            stack_protect_pmp_address_end = const {(0x62030000 + 160 * 1024) >> 2},
            stack_protect_pmp_flags = const 0b00001000 << 8,
            main = sym main,
        )
    }
}

#[cfg(all(feature = "bl808-dsp", target_arch = "riscv64"))]
#[naked]
#[unsafe(link_section = ".text.entry")]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn start() -> ! {
    unsafe {
        use crate::arch::rvi::Stack;
        const LEN_STACK_DSP: usize = 4 * 1024;
        #[unsafe(link_section = ".bss.uninit")]
        static mut STACK: Stack<LEN_STACK_DSP> = Stack([0; LEN_STACK_DSP]);
        core::arch::naked_asm!(
            "   la      sp, {stack}
            li      t0, {hart_stack_size}
            add     sp, sp, t0",
            "   la      t1, sbss
            la      t2, ebss
        1:  bgeu    t1, t2, 1f
            sd      zero, 0(t1)
            addi    t1, t1, 8 
            j       1b
        1:",
            "   la      t3, sidata
            la      t4, sdata
            la      t5, edata
        1:  bgeu    t4, t5, 1f
            ld      t6, 0(t3)
            sd      t6, 0(t4)
            addi    t3, t3, 8
            addi    t4, t4, 8
            j       1b
        1:",
            "   la      t0, {trap_entry}
            ori     t0, t0, {trap_mode}
            csrw    mtvec, t0",
            "   li      t1, {stack_protect_pmp_address_begin}
            csrw    pmpaddr0, t1
            li      t1, {stack_protect_pmp_address_end}
            csrw    pmpaddr1, t1
            li      t2, {stack_protect_pmp_flags}
            csrw    pmpcfg0, t2",
            "   call    {main}",
            stack = sym STACK,
            hart_stack_size = const LEN_STACK_DSP,
            trap_entry = sym trap_vectored,
            trap_mode = const 1, // RISC-V standard vectored trap
            // Set PMP entry to block U/S-mode stack access (TOR, no R/W/X permissions)
            stack_protect_pmp_address_begin = const {0x3F000000 >> 2},
            stack_protect_pmp_address_end = const {(0x3F000000 + 32 * 1024) >> 2},
            stack_protect_pmp_flags = const 0b00001000 << 8,
            main = sym main,
        )
    }
}

#[cfg(all(feature = "bl808-lp", target_arch = "riscv32"))]
#[naked]
#[unsafe(link_section = ".text.entry")]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn start() -> ! {
    unsafe {
        use crate::arch::rve::Stack;
        const LEN_STACK_LP: usize = 1 * 1024;
        #[unsafe(link_section = ".bss.uninit")]
        static mut STACK: Stack<LEN_STACK_LP> = Stack([0; LEN_STACK_LP]);
        core::arch::naked_asm!(
            "   la      sp, {stack}
            li      t0, {hart_stack_size}
            add     sp, sp, t0",
            "   la      t1, sbss
            la      t2, ebss
        1:  bgeu    t1, t2, 1f
            sw      zero, 0(t1)
            addi    t1, t1, 4
            j       1b
        1:",
            "   la      t3, sidata
            la      t4, sdata
            la      t5, edata
        1:  bgeu    t4, t5, 1f
            lw      t6, 0(t3)
            sw      t6, 0(t4)
            addi    t3, t3, 4
            addi    t4, t4, 4
            j       1b
        1:",
            // TODO trap support
            // TODO pmp support
            "   call  {main}",
            stack = sym STACK,
            hart_stack_size = const LEN_STACK_LP,
            main = sym main,
        )
    }
}

#[cfg(any(
    all(feature = "bl808-mcu", target_arch = "riscv32"),
    all(feature = "bl808-lp", target_arch = "riscv32"),
    all(feature = "bl808-dsp", target_arch = "riscv64")
))]
unsafe extern "Rust" {
    // This symbol is generated by `#[entry]` macro
    fn main() -> !;
}
