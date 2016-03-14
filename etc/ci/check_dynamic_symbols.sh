#!/bin/bash

ACTUAL_SYMBOLS=$(arm-linux-androideabi-objdump -T target/arm-linux-androideabi/debug/libservo.so | grep "D " | grep "UND" | awk '{ print $NF; }' | tr '\n' ';')
ALLOWED_SYMBOLS="unshare;malloc_usable_size;"

echo "Dynamic symbols in binary: $ACTUAL_SYMBOLS" | tr ';' ' '

if [[ "$ACTUAL_SYMBOLS" != "$ALLOWED_SYMBOLS" ]]
then
	echo "There are unexpected dynamic symbols in binary: $ACTUAL_SYMBOLS" | tr ';' ' '
	exit -1
fi
