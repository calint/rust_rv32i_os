#!/bin/bash
#
# note: when script fails `cat` process might be active reading from TTY 
#  do `ps aux | grep cat` and terminate the process
#
set -e
cd $(dirname "$0")

TTY=/dev/ttyUSB1
BAUD=115200
SLP=0.5

# capture ctrl+c and kill cat
trap 'kill $(jobs -p); exit 130' INT

stty -F $TTY $BAUD cs8 -cstopb -parenb -crtscts -ixon -ixoff -ignbrk -brkint -icrnl -opost -isig -icanon -iexten -echo -echoe -echok -echoctl -echoke
#    -crtscts disables hardware flow control
#    -ixon -ixoff disables software flow control
#    -ignbrk -brkint ignores break conditions
#    -icrnl ensures that carriage return characters are not translated to newlines
#    -opost disables output processing (output will be sent as-is)
#    -isig -icanon -iexten disables terminal signal handling and canonical input processing
#    -echo -echoe -echok -echoctl -echoke disables terminal echoing

cat $TTY | tee test.out &

read -rsp $'program or reset FPGA then press "enter" to continue\n\n'

# read commands from test.in and send them to TTY
while IFS= read -r line; do
    printf "%s\r" "$line" > $TTY
    sleep $SLP
done < ../test.in

# send SIGTERM (termination signal) to 'cat'
kill -SIGTERM %1

# wait for 'cat' to exit
wait %1 || true

if cmp -s ../test.diff test.out; then
    echo
    echo
    echo "test: OK"
    rm test.out
else
    echo
    echo
    echo "test: FAILED, check 'diff ../test.diff test.out'"
    exit 1
fi