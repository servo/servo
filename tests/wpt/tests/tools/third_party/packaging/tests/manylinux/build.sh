#!/bin/bash

set -x
set -e

if [ $# -eq 0 ]; then
	docker run --rm -v $(pwd):/home/hello-world arm32v5/debian /home/hello-world/manylinux/build.sh incontainer 52
	docker run --rm -v $(pwd):/home/hello-world arm32v7/debian /home/hello-world/manylinux/build.sh incontainer 52
	docker run --rm -v $(pwd):/home/hello-world i386/debian /home/hello-world/manylinux/build.sh incontainer 52
	docker run --rm -v $(pwd):/home/hello-world s390x/debian /home/hello-world/manylinux/build.sh incontainer 64
	docker run --rm -v $(pwd):/home/hello-world debian /home/hello-world/manylinux/build.sh incontainer 64
	docker run --rm -v $(pwd):/home/hello-world debian /home/hello-world/manylinux/build.sh x32 52
	cp -f manylinux/hello-world-x86_64-i386 manylinux/hello-world-invalid-magic
	printf "\x00" | dd of=manylinux/hello-world-invalid-magic bs=1 seek=0x00 count=1 conv=notrunc
	cp -f manylinux/hello-world-x86_64-i386 manylinux/hello-world-invalid-class
	printf "\x00" | dd of=manylinux/hello-world-invalid-class bs=1 seek=0x04 count=1 conv=notrunc
	cp -f manylinux/hello-world-x86_64-i386 manylinux/hello-world-invalid-data
	printf "\x00" | dd of=manylinux/hello-world-invalid-data bs=1 seek=0x05 count=1 conv=notrunc
	head -c 40 manylinux/hello-world-x86_64-i386 > manylinux/hello-world-too-short
	exit 0
fi

export DEBIAN_FRONTEND=noninteractive
cd /home/hello-world/
apt-get update
apt-get install -y --no-install-recommends gcc libc6-dev
if [ "$1" == "incontainer" ]; then
	ARCH=$(dpkg --print-architecture)
	CFLAGS=""
else
	ARCH=$1
	dpkg --add-architecture ${ARCH}
	apt-get install -y --no-install-recommends gcc-multilib libc6-dev-${ARCH}
	CFLAGS="-mx32"
fi
NAME=hello-world-$(uname -m)-${ARCH}
gcc -Os -s ${CFLAGS} -o ${NAME}-full hello-world.c
head -c $2 ${NAME}-full > ${NAME}
rm -f ${NAME}-full
