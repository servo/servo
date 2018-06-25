#!/bin/sh

# Any
#target=""

# Specific device
#target="-s something"

# Emulator
target="-e"

# USB
#target="-d"

path="$1"
base="$(basename $1)"
remote_path="/data/local/tmp/$base"
shift

adb $target "wait-for-device"
adb $target push "$path" "$remote_path" >&2
adb $target shell "$remote_path" "$@"
