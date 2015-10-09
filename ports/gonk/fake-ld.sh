#!/bin/bash

# Add the position-independent executable flag if not building a shared lib.
if echo $@ | grep -qv " -shared "
then
        PIE_FLAG="-pie"
fi
echo $PIE_FLAG
echo arm-linux-androideabi-g++ $@ $LDFLAGS $PIE_FLAG -lGLESv2 -L$GONKDIR/backup-flame/system/lib/ >linker.log
