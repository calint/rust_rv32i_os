# Quality Assurance

* `test.in` - input file for the emulator or FPGA
* `test.diff` - expected output from the emulator or FPGA
* `emulator/test.sh` - script to run the test using the emulator
* `fpga/test.sh` - script to run the test on the FPGA
  * _note: assumes the FPGA opens a TTY on `/dev/ttyUSB1`_
