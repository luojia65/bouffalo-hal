Build:

```
rustup target install riscv32imac-unknown-none-elf
cargo build --target riscv32imac-unknown-none-elf --release -p bl702-dualuart
```

Ref: https://github.com/sipeed/bl_mcu_sdk/tree/master/bsp/board/bl702/bl702_debugger_dualuart
