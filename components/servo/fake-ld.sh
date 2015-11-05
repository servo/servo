#!/bin/bash
TARGET_DIR=$OUT_DIR/../../..
arm-linux-androideabi-gcc $@ $LDFLAGS -lc -o $TARGET_DIR/libservo.so -shared && touch $TARGET_DIR/servo
