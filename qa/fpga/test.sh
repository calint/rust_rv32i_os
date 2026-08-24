#!/bin/bash
#
# note: when script fails `cat` process might be active reading from TTY
#  do `ps aux | grep cat` and terminate the process
#
set -e
cd $(dirname "$0")

TTY=/dev/ttyUSB1
BAUD=115200
SLP=0.1

# capture ctrl+c and kill cat
trap 'kill $(jobs -p); exit 130' INT

stty --file $TTY $BAUD cs8 -cstopb -parenb -crtscts raw -echo
#   cs8: 8 data bits per character
#   -cstopb: 1 stop bit (the - disables 2 stop bits)
#   -parenb: no parity bit
#   -echo: disables echoing of received input characters back to the sender

cat $TTY | tee test.out &

read -rsp $'program or reset FPGA then press "enter" to continue\n\n'

# read commands from test.in and send them to TTY
while IFS= read -r line; do
    printf "%s\r" "$line" >$TTY
    sleep $SLP
done <../test.in

# send SIGTERM (termination signal) to 'cat'
kill -SIGTERM %1

# wait for 'cat' to exit
wait %1 || true

if cmp --silent ../test.diff test.out; then
    echo
    echo
    echo "test: OK"
    rm test.out
else
    echo
    echo
    echo "test: FAILED, check 'diff a qa/test.diff qa/fpga/test.out'"
    exit 1
fi
