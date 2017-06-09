.PHONY: all
all: openssl
	@:  # No-op to silence the "make: Nothing to be done for 'all'." message.

# From http://wiki.openssl.org/index.php/Android
.PHONY: openssl
openssl: openssl/libssl.so

openssl/libssl.so: openssl/Configure
	./openssl.sh ${ANDROID_NDK}

openssl/Configure:
    VERSION=1.0.2k; \
	URL=https://s3.amazonaws.com/rust-lang-ci/rust-ci-mirror/openssl-$$VERSION.tar.gz; \
	curl $$URL | tar xzf -
