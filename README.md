# rust on bare metal rv32i

## setup
```sh
rustup target add riscv32i-unknown-none-elf
```

## build and emulate
initialize for Tang Nano 9K
```sh
./configure.py 9k
```
or Tang Nano 20K
```sh
./configure.py 20k
```
then run
```sh
./run.sh
```

## note
* see https://github.com/calint/tang-nano-9k--riscv--cache-psram for FPGA implementation of the RISC-V RV32I for Tang Nano 9K that runs the application
* see https://github.com/calint/tang-nano-20k--riscv--cache-sdram for Tang Nano 20K version
* committed code has been tested in emulator
* tagged versions have been tested in emulator and on hardware

## tools
* cargo 1.89.0 (c24e10642 2025-06-23)
* rustc 1.89.0 (29483883e 2025-08-04)