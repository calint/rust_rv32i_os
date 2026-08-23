# rust on bare metal rv32i

## setup

```sh
rustup target add riscv32i-unknown-none-elf
```

## build, emulate and flash

initialize for Tang Nano 9K

```sh
./configure.py 9k
```

or Tang Nano 20K

```sh
./configure.py 20k
```

then run in the emulator

```sh
./run.sh
```

to flash the firmware to the FPGA use

```sh
scripts/firmware-build-and-flash-9k.sh
```

or

```sh
scripts/firmware-build-and-flash-20k.sh
```

## note

* see <https://github.com/calint/tang-nano-9k--riscv--cache-psram> for FPGA
  implementation of the RISC-V RV32I for Tang Nano 9K that runs the application
* see <https://github.com/calint/tang-nano-20k--riscv--cache-sdram> for Tang
  Nano 20K version
* committed code has been tested in emulator
* tagged versions have been tested in emulator and on hardware

## tools

* cargo 1.98.0 (797e8a9bc 2026-08-05)
* rustc 1.98.0 (88d9e12ae 2026-08-18)
