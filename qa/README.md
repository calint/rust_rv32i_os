# Quality Assurance

* `test.in` input to emulator or FPGA
* `test.diff` expected output from emulator or FPGA
* `/emulator/test.sh` script to run test using emulator
* `/fpga/test.sh` script to run test on the FPGA
  * _note: assumes FPGA opens tty on `/dev/ttyUSB1`_
