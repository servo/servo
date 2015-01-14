.PHONY: all
all: openssl

# From http://wiki.openssl.org/index.php/Android
.PHONY: openssl
openssl: openssl-1.0.1k/libssl.so

openssl-1.0.1k/libssl.so: openssl-1.0.1k/Configure
	./openssl.sh ${ANDROID_NDK}

openssl-1.0.1k/Configure:
	wget https://www.openssl.org/source/openssl-1.0.1k.tar.gz
	tar -zxf openssl-1.0.1k.tar.gz
