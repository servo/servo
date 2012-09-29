#!/bin/bash
# This script creates a skeleton implementation for a C++ class which
# implements a Web IDL interface.

# This script is released into the public domain.

if [ -z "$1" ]; then
  echo usage: ./generate.sh ClassName
  exit 1
fi

expression="s/Skeleton/$1/g"

sed "$expression" < Skeleton.h > "$1.h"
sed "$expression" < Skeleton.cpp > "$1.cpp"

