.PHONY: all
all: openssl
	@:  # No-op to silence the "make: Nothing to be done for 'all'." message.

# From http://wiki.openssl.org/index.php/Android
.PHONY: openssl
openssl: openssl-1.0.1t/libssl.so

openssl-1.0.1t/libssl.so: openssl-1.0.1t/Configure
	./openssl.sh ${ANDROID_NDK}

openssl-1.0.1t/Configure:
	wget https://www.openssl.org/source/old/1.0.1/openssl-1.0.1t.tar.gz
	tar -zxf openssl-1.0.1t.tar.gz
