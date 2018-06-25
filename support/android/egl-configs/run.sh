#!/bin/sh

path="$1"
base="$(basename $1)"
remote_path="/data/local/tmp/$base"
shift

adb -e wait-for-device
adb -e push "$path" "$remote_path"
adb -e shell "$remote_path" "$@"
