#!/bin/bash
arm-linux-androideabi-g++ $@ $LDFLAGS -pie -lGLESv2 -lsupc++  -L$GONKDIR/prebuilts/ndk/9/sources/cxx-stl/gnu-libstdc++/4.6/libs/armeabi/ -L$GONKDIR/backup-flame/system/lib/
