.PHONY: all
all: openssl
	@:  # No-op to silence the "make: Nothing to be done for 'all'." message.

# From http://wiki.openssl.org/index.php/Android
.PHONY: openssl
openssl: openssl-1.1.0f/libssl.so

openssl-1.1.0f/libssl.so: openssl-1.1.0f/Configure
	./openssl.sh ${ANDROID_NDK}

openssl-1.1.0f/Configure:
	wget https://www.openssl.org/source/openssl-1.1.0f.tar.gz
	tar -zxf openssl-1.1.0f.tar.gz
