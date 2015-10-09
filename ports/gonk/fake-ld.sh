#!/bin/bash

# Add the -pie (position-independent executable) flag.
PIE_FLAG="-pie"
while getopts :o: OPT; do
  case $OPT in
    o)
      case $OPTARG in
        *.so)
          # Not an executable
          PIE_FLAG=""
          ;;
      esac
      ;;
  esac
done
arm-linux-androideabi-g++ $@ $LDFLAGS $PIE_FLAG -lGLESv2 -L$GONKDIR/backup-flame/system/lib/
