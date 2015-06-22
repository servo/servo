#!/bin/bash
arm-linux-androideabi-g++ $@ $LDFLAGS -pie -lGLESv2 -L$GONKDIR/backup-flame/system/lib/
