#!/bin/sh
set -e
cd $(dirname "$0")

TTY=/dev/ttyUSB1
BAUD=115200
SLP=0.1

stty --file $TTY $BAUD cs8 -cstopb -parenb -crtscts raw -echo
#   cs8: 8 data bits per character
#   -cstopb: 1 stop bit (the - disables 2 stop bits)
#   -parenb: no parity bit
#   -echo: disables echoing of received input characters back to the sender

# stream serial port to terminal and log file
tee test.out <$TTY &
LOG_PID=$!

# ensure background logger is killed on ANY exit (normal, error, or Ctrl+C)
trap 'kill $LOG_PID 2>/dev/null || true' EXIT

echo "Assuming the FPGA opens $TTY at $BAUD baud, 8 data bits, 1 stop bit, no parity"
read -rsp $'Program or reset FPGA then press "Enter" to continue\n\n'

# read commands from test.in and send them to TTY
while IFS= read -r line; do
    printf "%s\r" "$line" >$TTY
    sleep $SLP
done <../test.in

# stop logger
kill $LOG_PID

if cmp --silent ../test.diff test.out; then
    echo
    echo
    echo "test: OK"
    rm test.out
else
    echo
    echo
    echo "test: FAILED, check 'diff --text qa/test.diff qa/fpga/test.out'"
    exit 1
fi
