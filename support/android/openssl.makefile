.PHONY: all
all: openssl
	@:  # No-op to silence the "make: Nothing to be done for 'all'." message.

# From http://wiki.openssl.org/index.php/Android
.PHONY: openssl
VERSION = 1.0.2k
openssl: openssl-${VERSION}/libssl.so

openssl-${VERSION}/libssl.so: openssl-${VERSION}/Configure
	./openssl.sh ${ANDROID_NDK}

openssl-${VERSION}/Configure:
	URL=https://s3.amazonaws.com/rust-lang-ci/rust-ci-mirror/openssl-${VERSION}.tar.gz; \
	curl $$URL | tar xzf -
