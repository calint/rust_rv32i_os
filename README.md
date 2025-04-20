# rust on bare metal rv32i

## setup
```sh
rustup target add riscv32i-unknown-none-elf
```

## build and emulate
```sh
./run.sh
```

## note
* see https://github.com/calint/tang-nano-9k--riscv--cache-psram for FPGA implementation of the RISC-V RV32I that runs the application
*  committed code has been tested in emulator
*  tagged versions have been tested in emulator and on hardware
