.PHONY: all
all: openssl

# From http://wiki.openssl.org/index.php/Android
.PHONY: openssl
openssl: openssl-1.0.1j/libssl.so

openssl-1.0.1j/libssl.so: openssl-1.0.1j/Configure
	./openssl.sh ${ANDROID_NDK}

openssl-1.0.1j/Configure:
	wget https://www.openssl.org/source/openssl-1.0.1j.tar.gz
	tar -zxf openssl-1.0.1j.tar.gz
