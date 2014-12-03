CARGO_OPTS ?=
RUST_ROOT ?= /usr/local

.PHONY: all
all: glut_app
	NDK_DEBUG=1 $(ANDROID_NDK)/ndk-build -B
	find glut_app/target ! \( -type d -name dist -prune \) -name libmozjs.so | \
		xargs -I {} cp -f {} libs/armeabi
	find glut_app/target ! \( -type d -name dist -prune \) -name 'libglut_app-*.so' | \
		xargs -I {} cp -f {} libs/armeabi/libservo.so
	find $(RUST_ROOT)/lib/rustlib/arm-linux-androideabi/lib \
		-name '*.so' -type f -size +1c | \
		xargs -I {} cp -f {} libs/armeabi
	$(ANDROID_SDK)/tools/android update project \
		--name ServoAndroid \
		--target "android-18" \
		--path .
	ant debug

.PHONY: glut_app
glut_app: openssl
	cd glut_app; \
	OPENSSL_PATH=`pwd`/../openssl-1.0.1j ../../../mach cargo build --target=arm-linux-androideabi $(CARGO_OPTS)

.PHONY: install
install:
	$(ANDROID_SDK)/platform-tools/adb install -r bin/ServoAndroid-debug.apk

# From http://wiki.openssl.org/index.php/Android
.PHONY: openssl
openssl: openssl-1.0.1j/libssl.so

openssl-1.0.1j/libssl.so: openssl-1.0.1j/Configure
	bash openssl.sh ${ANDROID_NDK}

openssl-1.0.1j/Configure:
	wget https://www.openssl.org/source/openssl-1.0.1j.tar.gz
	tar -zxvf openssl-1.0.1j.tar.gz
